---
id: S-0409
title: Shroud and sight (explored and seen cells, sight radius and shape, height blocking, gap generators, re-covering, reveal on fire)
topic: visibility
revision: 13
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-15
measurements: []
requests: [Q-0110, Q-0111, Q-0460, Q-0649]
supersedes: []
---

# S-0409: Shroud and sight (explored and seen cells, sight radius and shape, height blocking, gap generators, re-covering, reveal on fire)

## Summary

When a cell becomes explored for a house and when it is covered again: the sight of objects (radius,
centre, shape, the cells that high ground hides), when sight is applied, the whole-map reveals,
gap generators, the periodic re-cover, spy satellites, the reveal that some weapons give the house
they fire at, and fog of war (always off in a skirmish). S-0110 draws the result and this spec owns
the states it reads (skirmish only: network games and campaigns are out of scope).

## Terms

Shared terms used here are defined in the glossary: *cell*, *legal cell*, *usable cell*, *house*,
*AI house*, *neutral house*, *special house*, *local player*, *country*, *object*, *type*, *field*,
*building*, *foundation*, *top corner*, *aircraft*, *infantry*, *vehicle*, *height level*, *world
height*, *height lift*, *lepton*, *tick*, *tick number*, *game minute*, *shroud*, *fog of war*,
*weapon*, *reference point* (S-0204, claim 4), *being erased*, *low power*, *radar building*, *bystander computer player*, *veterancy rank*, *departure*, *importer warning*, *quirk switch*, *round-toward-zero
mode*, *measurement setup*, *recorder*, *scripted input*, *override section*, *game start*,
*lobby*, *in play*, *on the map*, *turn* (in S-0203's sense: an object's share of one tick, as in
"its own turn") and *turn pass*. S-0205's notation applies: trunc, ⌊ ⌋, RZ₅₃ and *double*. The
simulation lift *L(z)* is defined here in F2 for sight centres; it is not the drawing lift of
S-0101 claim 3. In addition:

- **Player house.** A house of a player, human or AI (glossary *AI house*). The neutral house and
  the special house are not player houses.
- **Explored, unexplored.** A per-house state of every legal cell. Shroud covers the cells that are
  unexplored for the local player's house (S-0110). In the original only the local player's house
  has this state; see claim 1 for our engine.
- **Seen, unseen.** A second per-house state of every legal cell. It changes what is drawn only
  with fog of war on (claim 19), which a skirmish never has, so in a skirmish it is bookkeeping
  that only the uncounted sight (Edge cases, `quirks.uncounted_sight`) reads. Sights, area
  reveals and the spy satellite's whole-map reveal make the cells they explore seen as well; a
  plain reveal (claim 16) and the whole-map reveal of claim 3 and of defeat (claim 15) make cells
  explored without making them seen.
- **Allied.** As in S-0314 (Terms): a house is allied with itself and with the houses its alliance
  state names.
- **Ally reveal.** A game-wide rules flag (Import mapping, set when absent). With it on, an
  object's sight counts for every player house that its owner counts as allied (claim 2).
- **Sight field.** A type's integer sight field, in cells (Import mapping).
- **Sight radius.** The radius in cells that an object's sight uses at a given moment (claim 4).
- **Sight centre.** The cell on which a sight or an area reveal is centred (claim 5).
- **Observer level.** For a sight or reveal from a point at world height *Z* leptons: trunc(*Z* /
  104), in height levels (claim 7).
- **Sight shape.** The set of cell displacements (*dx*, *dy*) that a radius *r* covers (claim 6,
  table T1). *dx* runs along the map x axis, *dy* along y.
- **A sight.** The coverage that one object's sight currently holds for one house: the cells of
  its sight shape around its sight centre, minus the cells claim 7 hides, recorded when it was last
  applied (claim 8). An object holds at most one sight. A sight is *added* when it is applied and
  *removed* when it is taken back.
- **Plain reveal.** A reveal that makes cells explored without adding a sight: the reveal on fire
  of claim 16. Area reveals of other specs (claim 17) are not plain reveals.
- **Gap generator, gap field.** A *gap generator* is an object whose type has the gap-generator flag
  (Import mapping). While it generates, it holds one *gap field*: the cells of claim 11 around its
  sight centre. A gap field is *hostile* to house H when its owner is not allied with H and H has
  not been defeated (claim 11). S-0110 (Terms) names the same stock building.
- **Cover count, hostile gap count, pending re-cover.** This spec states the original's behaviour
  through three per-house quantities of every legal cell: an integer *cover count k*, which starts
  at 1, an integer *hostile gap count G* (the number of hostile gap fields covering the cell),
  which starts at 0, and a yes/no *pending re-cover* mark. Claim 12 gives how they change. Only the
  resulting explored, unexplored, seen and unseen cells are observable. An implementation may hold
  any state that gives the same results.
- **Surrounded mark.** A per-house yes/no mark of every legal cell, used in the same way as the
  explored state is. A sight (S1 row 1) and a plain reveal set it whenever they explore a cell, whatever
  its neighbours. The explored-only whole-map reveals of claims 3 and 15 also set it where
  all eight map neighbours are explored, reading non-legal neighbours through claim 14's shared
  state. Border cells are not categorically excluded. These reveals do not clear an existing mark.
  It is cleared whenever the cell becomes unexplored, and on the map neighbours of a cell that
  regrowth re-covers (claim 18). It matters only for regrowth and for S1 row 1.
- **Discovered by H.** An object's discovery mark for house *H*, distinct from ownership and
  from its cell's exploration. Claim 20 gives placement and reveal rules. Claim 16 and S-0410
  read the mark.
- **Re-cover sweep.** The part of a tick, in every tick whose tick number is a multiple of 120, in
  which every cell with the pending re-cover mark becomes unexplored and unseen (claim 13).
- **Spy satellite.** A building type flag (Import mapping). A house with an active spy satellite
  has the whole map explored (claim 15).
- **Reveal-on-fire flag.** A weapon flag (Import mapping, claim 16).
- **Whole-map-revealed mark.** A per-house yes/no mark that the whole map has been revealed for
  the house (claims 3 and 15); any gap field that starts or stops clears it (claim 11). Nothing a player
  can see differs with the mark alone; it matters through the conditions that read it (claim 15, and
  S-0011 claim 26, which tests it).
- **Electromagnetic pulse, electromagnetic lock.** The temporary switched-off state that a pulse
  weapon puts a building or vehicle in. Its weapons and duration belong to the combat and
  special-ability specs; this spec only says where the state is not tested (claim 15).
- **Border cells of the map's diamond.** The legal cells of which at least one of the eight map
  neighbours is not a legal cell (S1 row 10).
- **Arrival in a cell.** The movement arrival update, not merely crossing a cell boundary.
  For ordinary walking infantry it is the tick the unit snaps to its chosen standing position
  (S-0208, claims 6 and 23), including intermediate arrivals while the walk continues and a
  same-cell arrival (S-0208, claim 21). Driving arrivals follow S-0207 (scenario D1).

## Behaviour

1. **One state per player house (our engine).** In the original the explored and seen states, the
   cover counts and the pending marks exist once, for the local player's house. Every reveal in
   this spec is applied only when the house it would count for is the local player's house (after
   the ally rule of claim 2), and every gap field is applied only against the local player's
   house; the computer player keeps no shroud and acts as if it saw the whole map. Our engine keeps
   all of these states for every player house, and applies every rule below to each player house
   *H* in place of "the local player's house". This follows from the AI-houses rule of the
   glossary and from the rule that game state never depends on the local player: any rule that reads
   the explored state (for example S-0214's order rules for moving into shroud) reads the state
   of the house concerned. What is drawn
   uses the local player's house's states (S-0110). The neutral and special houses keep no state; whether their objects reveal for anyone depends
   only on their countries' passive flag (claim 2). For AI designers: the original's computer
   player ignored the shroud; an AI of our engine sees only what its house's states show, unless
   its strategy is given more by design.
   - Backing: `unmeasured` for the original's single state (the computer player's view cannot be
     observed); the rule for our engine is a design rule, not a claim about the original
   - Confidence: high

2. **Whose sight counts for H.** A sight and the area reveals of claim 17 count for house *H* when
   the house they are made for is *H*, or when that house is allied with *H* and ally reveal is on.
   A reveal on fire counts only for the house that owns the target (claim 16), never for its
   allies. An object whose owner's country is marked passive for multiplayer (Import mapping) reveals
   nothing for anyone, and this takes precedence over the reveal-to-all flag of claim 8: a
   reveal-to-all building of a passive country reveals nothing.
   - Backing: `measurable` (scenario V3 for the owner, the allied rule, ally reveal off and the
     passive flag; scenario V7, part E for the precedence over reveal-to-all; V15 part B for
     alliance being tested from the object owner's side)
   - Confidence: high for the owner, the allied rule and the flag; medium for the alliance side
     and for the precedence

3. **Start of a game.** At game start every cell is unexplored and unseen for every player house,
   every cover count is 1, every hostile gap count is 0 and no pending mark is set. When the rules'
   lobby shroud setting is off (Import mapping), the whole map is revealed for every player house
   at game start and its whole-map-revealed mark is set; the original does this for the local
   player's house. This reveal makes every legal cell explored but changes no cover count and
   does not make cells seen. It sets the surrounded mark wherever all eight map neighbours are
   explored, without clearing any existing surrounded mark (S1 row 10). A non-legal neighbour
   uses claim 14's shared explored state: when that state is explored, border cells gain the mark
   too. When it is unexplored, a border cell gains no mark from this reveal alone. The same rule
   applies to the defeat reveal of claim 15. This distinction affects regrowth and hence which
   cells remain explored (claim 18); it is not merely a drawing difference.
   The skirmish lobby has no control of its own for this setting, so the rules value decides.
   A consequence for reveal on fire in such a game is in claim 16.
   - Backing: `measurable` for initial exploration and the whole-map reveal (scenario V1,
     part A; part B with a named test copy of the rules); the surrounded-mark consequence is
     `measurable` by V11 D for the initially unexplored shared state and V19 for both shared
     states at defeat, which uses the same reveal. Initial seen states and counts are
     `unmeasured` as separate quantities: V1 reads exploration only, and fog is off. Their
     observable consequences are tested by the sight, gap and regrowth scenarios below.
   - Confidence: high for the initial state and the reveal; medium that the skirmish lobby offers
     no control

4. **Sight radius.** An object's sight radius *r* is computed from its type's sight field *S*, its
   world height *Z* and its veterancy rank (Formulas F1):
   - a height bonus of *b* = 10 · trunc(*Z* / *P*) per cent, where *P* is the rules' leptons per
     sight increase (Import mapping), increases the field, and the result is truncated toward zero,
   - the sight ability of S-0311 (claim 5, Formulas F4) then multiplies it by the veteran sight
     factor,
   - a radius of 0 reveals nothing, and a radius above 10 counts as 10 (claim 6). Our engine
     also reveals nothing for a negative radius; the original's negative-radius result is
     undefined (Edge cases).

   The height bonus uses the absolute world height, not the height above the terrain, so a unit on
   high ground gets it as an aircraft does. In the original *b* wraps into the range −128 … 127
   (reduced modulo 256), so a *b* above 127 becomes negative (Edge cases, `quirks.sight_bonus_wrap`). A building's area reveal when it is
   placed (claim 8) uses the plain sight field, without the bonus and the veterancy factor.
   - Backing: `measurable` (scenario V2 for the cap and the field; V4 for the height bonus and for
     the rounding of F1, V4 runs 4 and 5 for the wrap of *b*; the veteran factor is measurable by S-0311's scenarios); `unmeasured` for
     the placement reveal of a building using the plain field, because the same building adds its
     full sight at the same moment over overlapping cells and
     no scenario here isolates that one-time reveal from the simultaneous full sight; in the
     ordinary non-wrapped case the plain field reveals no additional cells
   - Confidence: high for the formula and the cap; medium for the wrap of *b* (derived, not
     yet observed) and for its use at every object kind (the
     aircraft case of claim 8 is derived, not yet observed) and for the placement reveal

5. **Sight centre.** A sight or area reveal from the world point (*X*, *Y*, *Z*) is centred on the
   cell (trunc(*X*′ / 256), trunc(*Y*′ / 256)) with *X*′ = *X* − 256·*k*, *Y*′ = *Y* − 256·*k* and
   *k* = trunc(*L*(*Z*) / 30) (Formulas F2). That is, for positive height the point is shifted towards map north-west onto the
   height-0 grid, with the simulation lift of F2 counted in whole steps of 30 pixels.
   This retains the original's floating-point lift because the centre determines explored cells,
   not just drawing. S-0101 claim 3's exact drawing lift does not change this game state.
   For example at *Z* = 2080 leptons (level 20), *L* = 299 pixels and *k* = 9 cells,
   not the 10 cells an exact 300-pixel drawing lift would give.
   The point of an object is its position: for a building the centre of the
   top-left cell of its foundation, not the centre of its foundation (Edge cases,
   `quirks.building_sight_top_left`). When the sight centre is not a legal cell, the sight or
   reveal reveals nothing at all (Edge cases, `quirks.sight_centre_off_map`).
   - Backing: `measurable` (scenario V5; part D for the off-map case, part E for the
     simulation lift remaining distinct from the exact drawing lift)
   - Confidence: high for the formula; medium for the off-map case. Truncation, not floor,
     applies in both coordinates: −256 < *X*′ < 0 gives coordinate 0; such a centre is not legal

6. **Sight shape.** For a radius *r* from 1 to 10, the sight covers the legal cells (*cx* + *dx*,
   *cy* + *dy*) around the sight centre (*cx*, *cy*) with |*dy*| ≤ *r* and |*dx*| ≤ *w*ᵣ(|*dy*|),
   where *w*ᵣ is the half-width of table T1. The shape is symmetric in *dx* and *dy*. For *r* = 1,
   2, 3, 5, 6 and 7 it is exactly the cells with *dx*² + *dy*² ≤ *r*² + 1; T1 shows the
   differences at 4, 8, 9 and 10. Only legal cells are covered. The order in which the
   cells are processed has no observable effect.
   - Backing: `measurable` (scenario V2)
   - Confidence: high

7. **High ground hides cells.** With the rules' reveal-by-height flag on (Import mapping; set when
   absent), a cell *T* of the shape is left out of the sight (it is neither explored nor covered by it) when its
   *blocking cell* has a height level greater than the observer level + 3. The blocking cell of *T*
   is *T* + *s* + (2, 2) + (*c*ₓ − *cx*, *c*ᵧ − *cy*), where *s* is the step direction of table T2
   for *T*'s displacement, (*c*ₓ, *c*ᵧ) is the cell of the unmoved point (trunc(*X* / 256),
   trunc(*Y* / 256)) and (*cx*, *cy*) is the sight centre. For an observer at height 0 that is the
   cell two steps map south-east of the neighbour of *T* that lies one step towards the centre.
   Only that one cell is tested, not the cells along the line. The displacement (2, 2) and the
   single-cell test are quirks (Edge cases, `quirks.height_block_cell`). A blocking cell that is not
   a legal cell has height level 0.
   - Backing: `measurable` (scenario V6); the height level 0 of a blocking cell that is not legal is
     `unmeasured`, because no such cell can be given another height
   - Confidence: medium (the formula is exact; whether it was meant this way is unknown, so the
     scenario is designed to tell it from the plausible alternative)

8. **When sight is applied.** An object holds a sight only while it is on the map and inside the
   usable area. For each house *H* that it counts for by claim 2:
   - it adds its sight when it enters the map (it is placed, built, unloaded or created),
   - a walking or driving unit removes its sight and adds it again at the position reached on each
     arrival in a cell (Terms), including a walking infantry unit's same-cell arrival,
   - every moving object (any locomotor) also removes and adds its sight, in its turn, on each tick
     in which it is moving and at least 15 ticks have passed since its last such refresh (the first
     one comes on the first tick it moves, or 15 ticks after the previous refresh if it stopped and
     started again sooner), which is how aircraft and other units that arrive in no cell keep their
     sight current; flying and jumping locomotors also refresh on some events of their own movement
     (taking off and landing), which their specs own,
   - it removes its sight when it leaves the map (it dies, enters a transport or a building, or is
     otherwise taken off the map),
   - when its owner changes, it removes its sight for the old owner and adds it for the new one.

   Removing a sight leaves its cells explored unless claim 12 re-covers them. A building also
   makes a one-time area reveal with its plain sight field (claim 4) when it is placed, which
   explores the cells without adding a sight. A building of a type with the reveal-to-all flag
   (Import mapping) holds its sight for every player house, allied or not, unless its owner's
   country is passive (claim 2). All of this happens in
   the object's own turn, except the placement of a building, which happens when the building is
   placed (S-0203).
   - Backing: `measurable` (scenario V7: part A arrival, part B the sustained-flight 15-tick
     period, part E reveal-to-all, part F a change of owner; leaving the map by death
     in scenario V8 part B2); `unmeasured` for distinct factory-release and unloading sight
     events (the listed scenarios do not isolate these from other sights), and for the
     short-stop timer rule independently of take-off or landing refreshes (V7 B's restart
     run may include those other refreshes). A same-cell walking arrival's refresh is
     `unmeasured` independently here: on V7 A's flat ground its sight centre and radius
     are unchanged, so the explored footprint does not distinguish a refresh from none.
     Also `unmeasured` for entering a transport or
     a building and for the flying and jumping events: the removal shows only through a pending mark inside a hostile
     field, where the transport's own sight covers the same cells, and the jumping locomotor's
     events belong to its own spec
   - Confidence: high for entering the map, arrival in a cell, the 15-tick period and leaving the
     map; medium for the other ways of entering and leaving the map, for reveal-to-all, for the
     change of owner and for the restart after a short stop, and low for the flying and jumping events (the events are known to
     refresh sight, but their exact conditions were not worked out)

9. **Explored stays explored.** Without fog of war, a cell that has become explored for *H* stays
   explored, whatever happens to the sights over it, except through a hostile gap field (claims 11
   and 12), the re-cover after a plain reveal (claim 16), the loss of a spy satellite or a radar
   infiltration (claim 15) and the optional regrowth (claim 18).
   - Backing: `measurable` (scenario V7 part D; the exceptions by V8 to V11)
   - Confidence: high

10. **No sight while outside the playfield.** An object that is on the map but has not yet entered
    the usable area (for example a unit that a map places outside the local area, or one arriving
    from beyond the edge) holds no sight until it enters it.
    - Backing: `measurable` (scenario V7, part C)
    - Confidence: medium

11. **Gap fields.** A gap generator generates while it is on the map and in operation (S-0405,
    claims 8 and 9). So its field stops when the generator leaves operation (for example when its
    house has low power, or its power plant is destroyed) and starts again when it is back in
    operation. An accepted power-off order stops the field immediately, even if the building remains in operation because it is charged; an accepted power-on order lets the field start again in the next building turn that finds it in operation (S-0405, claim 10). The field also stops when the generator is removed from the map or destroyed. A charged building can restart its field in its next turn even while still switched off (S-0405, claim 9). Its gap field covers the cells (*cx* + *dx*, *cy* + *dy*)
    with *dx*² + *dy*² < (*R* + 1)² around its sight centre (claim 5), where *R* is its gap radius
    in cells (the type's gap radius field, wrapped into −128 … 127 by `quirks.gap_radius_wrap`, so
    a field value from 128 up reads as negative: Edge cases). This is a different shape from the sight shape; a gap
    radius of 0 covers the centre cell only, and a negative one covers nothing. The field is
    hostile to *H* when its owner is not allied with *H* and *H* has not been defeated (alliance
    tested from the generator owner's side). Gap fields of *H* and of its allies cover nothing for
    *H*. When a hostile field starts, each of its cells that no sight of *H* covers becomes
    unexplored for *H* at once (its seen state does not change); cells that a sight of *H* covers stay explored while that sight
    lasts. Every gap field that starts or stops, hostile to *H* or not, clears *H*'s
    whole-map-revealed mark (nothing observable differs for a field that is not hostile). When it stops, its cells stay unexplored, unless *H* has an active
    spy satellite (claim 15), which brings them back (explored, with the seen state unchanged) (Edge cases,
    `quirks.gap_end_keeps_cover`). The gap generator's own super-gap mode belongs to the
    S-0405 claim 16.
    - Backing: `measurable` (scenario V8, with part E for fields of allies, V15 for the alliance
      side and changes, part F for a defeated
      house and part G for the wrap of the gap radius; V16 for generation while switched off
      but charged). Accepted switch-off's immediate cessation, independently of the next
      building turn, is `unmeasured`: input applies before that turn and no drawn frame isolates
      the interval, especially when a charged building restores the field in the same tick. The original ends the game at the human's
      defeat (S-0414 claim 6), but play continues during its end delay: part F measures within
      that delay, lengthened by a named test rules file (S-0414 claim 8). In our engine the
      defeated person keeps watching (S-0414 claim 6); that departure is not the measurement
      oracle. `unmeasured` for the clearing of the mark by a field that is not hostile, because
      no cell changes and the mark alone is not visible (Terms)
    - Confidence: high for the accepted switch-off exception; medium for the next-turn
      restoration while charged; high for the shape, the immediate cover, the sight exception and the end without a
      spy satellite; medium for the wrap of the radius from 128 to 255 (low above 255), for the coupling to being in operation, for allied fields, for a
      defeated house and for the mark

12. **The cover count.** Per house *H* and legal cell, the cover count *k*, the hostile gap count
    *G* and the pending mark change as table S1 gives. In words: a sight of *H* arriving makes the
    cell explored and marked surrounded, and cancels a pending re-cover on a cell that was already
    explored and marked surrounded (so on a cell that a re-cover has just reset, only a second
    sight cancels it: claim 15); when the last sight of *H* leaves a cell that a
    hostile field covers, the cell is marked pending and the next re-cover sweep makes it
    unexplored and unseen; without a hostile field nothing happens. A plain reveal of a cell whose
    count is above 0 also marks it pending, so it lasts only until the next sweep (claim 16).
    - Backing: `measurable` for explored-state transitions (V8 for sights and fields, V10
      for plain reveals, V14 for area reveals, V13 B and B2 for re-cover). Cancellation of a
      pending mark by sight arrival over an already explored, surrounded cell is tested by
      V13 B2's repeat without satellite loss. Seen-state transitions
      are `unmeasured` directly because fog is off; V13 A tests their proposed indirect rim effect.
    - Confidence: high for the transitions of S1 rows 1 to 4 and 6; medium for rows 5, 7, 8 and 10
    - The one-off count after row 5 with a spy satellite (State and ordering) and the interleaving
      behind row 9 are `unmeasured`: they change a count by one and show only as one missed or one
      surplus pending mark, which needs an instrumented count or an interleaving of five events

13. **The re-cover sweep.** In every tick whose tick number is a multiple of 120 (tick 0
    included), before the turn pass of that tick (S-0203), every cell that is pending for *H*
    becomes unexplored and unseen for *H*, and its pending mark is cleared. A pending cell that
    a sight of *H* reaches again before that tick stays explored only if it was already explored
    and marked surrounded when the sight arrived (S1 row 1); claim 15 gives the exception after
    a whole-map re-cover. So a cell left by
    sight inside a hostile gap field stays visible for 1 to 120 ticks when regrowth is off,
    depending on when it was left. When regrowth is due in the same tick, claim 18's combined
    result applies, including at tick 0: the sweep does not remove a pending cell from that
    regrowth step's affected set.
    - Backing: `measurable` (scenario V8, the recorder logs the tick numbers; the place of the
      sweep before the turn pass by scenario V13 part C, which needs a sight added on a sweep tick,
      the only case where the order shows: see `quirks.recover_keeps_pending`)
    - Confidence: high for the period and the phase; medium for the place before the turn pass

14. **Gap fields and the map edge.** A gap field is applied even when its centre is not a legal
    cell, unlike a sight (claim 5), and its cells are not tested for legality. A cell of the field
    with both coordinates in 0 … 511 that is not a legal cell changes no legal cell. A cell with a
    coordinate outside 0 … 511 is read as another cell, as in S-0110's point test (Edge cases,
    `quirks.shroud_point_cell_wrap` of S-0110, which this spec shares): (*x*, *y*) with *x* < 0 is
    read as (*x* + 512, *y* − 1) and with *x* > 511 as (*x* − 512, *y* + 1), when that row lies in
    0 … 511. This can reach a legal cell only on maps whose width plus height (S-0009's map width
    and height) is close to 512 (S-0009, claim 2).
    A field cell that is not a legal cell also has an effect of its own (Edge cases,
    `quirks.offmap_cells_shared`): all cells that are not legal share one explored state, and the
    end of a hostile field that covers such cells, while *H* has an active spy satellite, makes
    that shared state explored, provided no other hostile field still covers a non-legal
    cell. Later gap changes can cover it again. Whole-map operations do not reset it, but their
    removal and restoration of gap fields do change it (claim 21). S-0110's border pieces then change (scenario V8, part H). This is an explicit exception
    to the usual non-legal-cell rule: S-0110 claim 1, its covered-neighbour definition and
    claim 8's point test already carry this exception. Its persistence is subject to claim 21,
    not immunity to later gap changes.
    - Backing: `measurable` (scenario V8, part D for the wrapped cells; part H for the shared state
      of the non-legal cells)
    - Confidence: medium for the wrapped cells; low for the shared state

15. **Whole-map reveal and re-cover.**
    - *Spy satellite.* A building of *H* qualifies when its type is a spy satellite and it is on the
      map, alive and not being sold (neither its current nor its queued behaviour is *selling*). When
      the first qualifying building, in the order in which *H* came to own its buildings, is not
      being erased (glossary), *H*'s spy satellite is active and the whole map is revealed for *H*
      (below), once, when it becomes active. When that first building is being erased, or no
      building qualifies, the satellite is inactive, even if a later spy-satellite building works (Edge
      cases, `quirks.spy_satellite_first_building_decides`), and if it was active, *H*'s whole map
      is re-covered as below. Power and the electromagnetic lock are not tested (Edge cases): a building under a pulse or
      in a house with low power still holds the satellite active. The house checks this whenever its
      radar state would be rechecked after an event of S-0410 claim 2, in the houses' part of
      the tick (S-0203). This spy-satellite test continues after defeat, at those same recheck
      moments, although radar availability itself is no longer re-evaluated (S-0410 claim 2).
      Thus loss of an active satellite at defeat can still cause a whole-map re-cover at the
      next recheck (scenario V9 part F3).
    - *Spy satellite's whole-map reveal of H.* If *H*'s whole-map-revealed mark is clear, every
      legal cell becomes explored and seen for *H*; its cover count becomes 0 and its hostile gap
      count becomes 0, and the mark is set. If the mark is already set, this operation changes
      nothing. In the resulting state, hostile gap fields leave their cells unexplored unless a
      sight of *H* covers those cells (S1 rows 1 and 3): a spy satellite does not show what a
      hostile gap generator hides. Any gap field on the map leaves the whole-map-revealed mark
      clear again (claims 8 and 11).
    - *Non-legal cells.* Neither operation resets the shared state directly. The observable result
      for non-legal cells depends on the gap-field changes associated with the operation and on
      whether the spy satellite is active at those changes. On satellite activation, the field
      changes occur while the satellite is inactive; on satellite loss, they occur while it is
      active. Claim 21 gives the resulting shared-state consequences.
    - *Whole-map re-cover of H.* Every legal cell becomes unexplored and unseen for *H*, every
      cover count becomes 1 and every hostile gap count 0, the re-applied sights and the gap
      fields come back (claims 8 and 11) and the whole-map-revealed mark is cleared. So only what
      those sights currently cover stays explored.
    - *Resulting cell states.* After either operation, each legal cell has the exploration, seen,
      cover-count, hostile-gap-count and pending-mark results specified in S1 rows 7 and 8. For
      exploration, a cell is explored after a reveal unless a hostile field covers it and no
      qualifying sight covers it; after a re-cover, it is explored only where a qualifying sight
      covers it. Pending marks are not reset by either operation. A sight clears a pending mark
      only when the cell is explored and marked surrounded when that sight arrives (S1 row 1).
      Let *n* be the number of sights included in the resulting coverage of the cell.
      - A mark held before the operation remains unless the resulting sight changes clear it as
        described below (`quirks.recover_keeps_pending`).
      - A re-cover can leave a new pending mark where none existed before. V13 part D tests
        this low-confidence observable prediction: under exactly one stationary own unit's
        sight and one hostile gap field, the cell becomes unexplored at the next sweep if the
        generator was created after the unit, but stays explored if the generator was created
        before it. This creation-order relation is a hypothesis, not a required internal
        traversal or field-application sequence. New-mark outcomes with several sights or
        generators are not established here (see Backing).
      - After a re-cover, a previously pending cell with *n* = 1 is explored immediately but
        becomes unexplored at the next sweep even if the unit stands still. With *n* ≥ 2 its
        pending mark is cleared and it stays explored; with *n* = 0 it remains unexplored
        (scenario V13, parts B and B2).
      - After a reveal, a previously pending cell with *n* = 0 retains its mark. If no hostile
        field covers it, it is explored immediately and becomes unexplored at the next sweep; a
        hostile field leaves it unexplored. With *n* ≥ 2 the mark is cleared. For *n* = 1,
        survival of the mark is an unmeasured hypothesis: it may depend on creation order
        relative to the hostile generators. No general creation-order rule for this reveal case
        is established here.
      A cell covered only by a standing allied unit is not included in the resulting sight
      coverage (below).
    - *Re-applied sights of H.* The sights of *H*'s own objects, the sights of buildings of houses
      allied with *H* (with ally reveal on) and the sights of other houses' buildings of a
      reveal-to-all type. The sights of allied units that are not buildings are neither removed
      nor added again: they count for *H* again only from their next refresh (claim 8), so a
      standing allied unit's surroundings stay covered after a re-cover until it moves (Edge cases,
      `quirks.recover_skips_allied_units`). The same removal and re-application happens when two
      houses ally or stop being allied (Edge cases).
    - *Radar infiltration.* When a spy of another house infiltrates a radar building of *H*,
      *H*'s whole map is re-covered, unless *H* has an active spy satellite (S-0510 claim 3,
      row E1). Alliance is not tested for the effect: this includes an allied spy if it reaches
      entry. Entry conditions and the single-effect precedence belong to S-0510 claims 1 and 3.
    - *Defeat.* When a player house is defeated, and its whole-map-revealed mark is clear, the whole
      map becomes explored for it as in claim 3 (explored, not seen; cover counts, hostile gap counts
      and pending marks unchanged, S1 row 10) and the mark is set. Border cells gain surrounded
      marks when the shared non-legal explored state is explored, just as at game start (V19).
      Existing surrounded marks are retained. The pending marks the house holds
      at that moment are kept, so the next re-cover sweep makes those cells unexplored again.
      With no later whole-map operation, nothing explores them afterwards, because a defeated
      house has no sights and no gap field is hostile to it: the defeated player sees the whole
      map except for the few cells that were pending at the moment of defeat (Edge cases,
      `quirks.defeat_reveal_keeps_pending`). This does not prevent the satellite-loss re-cover
      at a later recheck described above.
    - Backing: `measurable` (scenario V9; conditional allied infiltration and effect precedence
      by S-0510 scenarios I5 and I2; the surviving marks by scenario V13 parts B and B2, the
      single-sight, single-generator creation-order prediction by part D); `unmeasured`
      for queued selling and the ownership-order rule for acquired satellite buildings (V9
      does not isolate these conditions), for new-mark outcomes with several sights or
      generators, and for mark survival after a
      reveal: V13 tests re-cover, not reveal, and no scenario here isolates the required pending
      cell under sight before satellite activation. Clearing a pre-existing mark specifically
      by a second re-applied sight is also `unmeasured`: V13 B2's second sight clears it before
      the whole-map operation, so B2 tests the final explored result but cannot isolate that
      clearing event. Immediate seen-state changes and the
      whole-map-revealed mark are also `unmeasured` as state in their own right: fog is off
      and the mark alone is not drawn; the explored consequences are the scenario targets. The
      defeat clause is `measurable` (V9 parts F and F2; V19 for border regrowth), as is the satellite test after defeat
      (part F3), within the original's end delay lengthened by a named test rules file
      (S-0414 claim 8). The original ends the game at the human's defeat (S-0414 claim 6).
      Our engine instead lets the defeated person keep watching, but that departure is not
      the measurement oracle
    - Confidence: high for the reveal, the re-cover, the gap fields covering again after the
      reveal and ordinary radar infiltration; medium for conditional allied infiltration,
      for the list of conditions, for power not being tested,
      for the allied-unit gap, for the surviving marks and for the defeat reveal; low for the
      creation-order prediction for new marks, for the survival of a mark
      after a reveal and for the pending marks of the defeat reveal; high for a second re-applied
      sight clearing a mark after a re-cover (it is the counter rule of S1 row 1)

16. **Reveal on fire.** When an object fires a shot from a weapon with the reveal-on-fire flag at a
    target object owned by house *H*, the cells of the sight shape of radius 3 (claims 5 to 7)
    around the firer's *reference point* (glossary), not its position, become explored for *H*
    by a plain reveal; they are not made seen. For a building whose foundation is larger than one
    cell the centre therefore differs from its sight centre (claim 5). Whether the reveal happens
    depends on whether the firer's reference point is shrouded for *H* (S-0110, claim 8) and on
    the firer:

    | Firer's reference point for *H* | Firer | Reveal |
    |---|---|---|
    | not shrouded | owned by *H*, or discovered by *H* (Terms) | no |
    | not shrouded | neither owned nor discovered by *H* | yes |
    | shrouded | an aircraft owned by *H* | no |
    | shrouded | any other firer, including one owned by *H* that is not an aircraft | yes |

    A plain reveal adds no sight, so by claim 12 the newly
    explored cells are marked pending (their count is 1 if no sight of *H* has ever covered them)
    and become unexplored again at the next re-cover sweep: with regrowth off, the attacker is
    glimpsed for 1 to 120 ticks (Edge cases, `quirks.fire_reveal_expires`). Regrowth can consume
    the pending mark earlier (claim 18). With the lobby shroud setting off (claim 3)
    every cell is explored with count 1. An undiscovered firer could therefore mark its reveal
    pending and produce shroud in a game that started without it. However, an ordinary ground
    firer created after game start is already discovered on placement (claim 20), even if no
    cover piece changes. That case produces no reveal and no later shroud. The undiscovered
    condition cannot be assumed merely from the absence of a scout.
    - Backing: `measurable` (scenario V10: part A the reveal, part B the reference point, part C the
      skip for a discovered firer, part D discovery on arrival in explored ground,
      part E that allies of the target's owner get nothing, part F for no shroud-off expiry
      from an ordinarily placed ground firer),
      `unmeasured` for the hypothetical undiscovered-firer shroud-off consequence (no setup
      here isolates an undiscovered firer after claim 20's placement rule), and for the aircraft clause,
      which needs an aircraft of *H* firing at an object of *H* (a scripted attack on its own
      house's object, not available in a skirmish)
    - Confidence: high for the radius, the owner of the target, the flag and the centre
      point; medium for the skip conditions (the discovered rule of Terms is derived) and for the
      expiry (it follows from S1, which is derived, not yet observed); low for the aircraft clause
      and for the hypothetical undiscovered-firer shroud-off consequence

17. **Area reveals of other specs.** A reveal around a point that another spec orders (the waypoint
    reveal of S-0011, claim 26, the drop-zone reveal of S-0011, claim 36, and superweapon
    reveals) uses claims 5 to 7 with the radius it gives (counted as 10 above 10, nothing at 0) and
    the observer level of the point. The psychic reveal is an exception: it uses claims 5 and 6
    but skips claim 7's high-ground test, so high ground hides none of its legal shape cells,
    whatever the reveal-by-height flag says. S-0610 claim 14 gives the houses it is made for.
    It explores and sees the cells and adds no lasting sight
    (S1 row 6). Outside hostile gap fields the cells stay explored. A cell inside a gap field
    hostile to the house is marked pending and re-covered at the next sweep, unless a sight of
    that house covers it. Which houses it is made for is the ordering spec's. In the original, S-0011 claims
    26 and 36 reveal only for the local player. Our engine applies those reveals to every
    player house, human or AI, observers included, as their documented departure under claim 1.
    S-0011 claim 26 skips each house whose whole-map-revealed mark is set, whereas its
    claim 36's drop-zone reveal does not. This exception is tested separately for each house,
    not against the local player's mark for all houses.
    The all-player-house scope is a design rule, not an observation of the original.
    - Backing: `measurable` (scenario V14 for the expiry inside a hostile field and the permanence
      outside; the area and the centre are S-0011's scenarios T16 and T21, which observe the disc
      and its centre but not the expiry; S-0610 scenario P7 tests the psychic reveal's exemption
      from height blocking and its recipients)
    - Confidence: medium; high for the psychic reveal's exemption from height blocking

18. **Optional regrowth.** With the rules' shroud-regrowth flag on (off when absent) and a
    regrowth rate other than 0, a regrowth step runs every trunc(rate × 900) ticks (F5; the first
    one at tick 0, and on every tick when the period is 0 or less). For house *H*, the affected
    cells are exactly those that, at the start of the step, are explored and not marked
    surrounded, or hold a pending re-cover mark. In the resulting state every affected cell
    has its pending mark cleared. An affected cell is unexplored unless it lies under one of
    *H*'s own objects' sights or an allied building's sight that counts for *H*. Every map
    neighbour of an affected cell that was explored at the start loses its surrounded mark,
    except where those sights restore it. Clearing a neighbour's mark does not make it affected
    in the same step: if it remains explored and unmarked, the next step affects it. Thus a
    ring cannot disappear in the same step merely because the adjacent ring disappeared.
    Seen states are unchanged except that the sights make their cells seen. The sights' resulting
    count changes are those of uncounted sight (Edge cases, `quirks.uncounted_sight`), not an
    extra lasting counted sight. No internal ordering of cell updates is required.

    When a regrowth step and a re-cover sweep are due in the same tick, including tick 0,
    regrowth has its effect before the sweep. A cell pending before that regrowth step is
    therefore affected even on a sweep tick: if explored, it loses exploration and clears its
    neighbours' surrounded marks as above, except where the restoring sights protect them.
    The sweep does not undo that neighbour effect. With no restoring sight, the pending cell
    is unexplored after the tick, and its neighbours that remain explored with their marks
    cleared become eligible for the next regrowth step. Running the sweep first would instead
    consume the pending mark without this erosion of the neighbours; that is not the required
    result. If regrowth
    restores exploration under a qualifying sight and clears the pending mark, the same-tick
    sweep does not re-cover that cell merely because it was pending before regrowth.
    Consequences. A cell explored by a sight or a plain reveal is always marked surrounded,
    whatever its neighbours, so a step affects no cell while no cell holds a pending mark and
    no cell was explored by a reveal that leaves the mark clear (some trigger reveals outside
    S-0011's subset, and the reveals of claim 3 and of defeat where a cell has an unexplored
    neighbour: S1 row 10). Those whole-map reveals leave a previously unmarked border cell
    unmarked only when the shared non-legal explored state is unexplored. When that state is
    explored, every border cell gains the surrounded mark, so the border alone starts no erosion.
    The reveal does not clear existing surrounded marks in either case. Once an affected explored cell loses exploration,
    its explored neighbours outside the restoring sights become unmarked and lose exploration
    at the next step, with their neighbours at the following step. The explored area outside
    those sights therefore erodes by one ring of eight-neighbour distance per step, through
    the connected explored cells. Cells under the restoring sights remain explored. In a skirmish with no map reveals of that kind, the start is a pending
    cell that the step reaches before the sweep does (a cell a sight left inside a hostile gap
    field, or a plain reveal, less than the step period before). With the lobby shroud setting off
    the whole map is explored without a sight. With the shared non-legal state still initially
    unexplored, its border cells are not marked surrounded, and the first step (at tick 0)
    leaves them unexplored, after which the shroud erodes
    inwards by one ring per step except under sights; this follows from the rules above and is
    not observed (scenario V11, part D).
    - Backing: `measurable` (scenario V11, with a named test copy of the rules: parts A and B that
      sight-explored cells do not regrow, part C that pending cells go at the regrowth step and
      that the ring follows one step later each; part D the shroud-off consequence; part E the
      zero-rate and non-positive-period cases; part F for the combined result on a sweep tick,
      and V19 for the border consequence with either shared state at defeat).
      A pending-cell collision specifically at tick 0 is `unmeasured`: no scenario here
      establishes a pending cell before tick 0's periodic work; its result follows the same
      coincident-tick rule. Restoration under sight on a coincident tick is also `unmeasured`
      here: part F excludes restoring sights, so it tests the neighbour erosion only.
      The re-application of the sights as uncounted sights is `unmeasured` (its effect is a
      count one lower on the rim cells, seen only through the interleaving of V13 part A,
      which does not use regrowth)
    - Confidence: low for the regrowth-state consequences; high for regrowth preceding a
      coincident sweep

19. **Fog of war is off in a skirmish.** In a skirmish fog of war is always off (S-0412, claim 1):
    starting a skirmish from the lobby always turns the fog setting off, whatever it was before,
    as one of the values a skirmish fixes. The rules' lobby fog default, which gives the setting
    its value only until the skirmish is started, the map's own fog flag and any other source have
    no effect, and no cell is ever drawn as fogged. What remains of the seen and unseen states in a skirmish is
    their bookkeeping (Terms).
    *For later modes only (out of scope here; the original's rule when fog is on):* as always
    (S1), sights and area reveals make cells seen and the re-cover sweep makes them unseen; in
    addition, when the fog rate *f* (game minutes, Import mapping) is not 0, every trunc(*f* × 900) ticks (F5) the seen cells
    of *H* that no sight of *H* covers at that moment become unseen again; for that test only the
    sights of *H*'s own objects and of allied buildings count, as in the re-application of claim 15
    (`quirks.recover_skips_allied_units`). What is hidden under fog (remembered images) belongs to
    S-0102 and S-0110.
    - Backing: `measurable` (scenario V12, with a named test copy of the rules, for the always-off
      rule); `unmeasured` for the fog-on rule, which no skirmish reaches
    - Confidence: high for fog being off in a skirmish (the original turns it off at every
      skirmish start, with no condition); low for the fog-on rule (timing and which cells become
      unseen)

20. **Discovery in a skirmish (Q-0110).** A newly created infantry, vehicle, aircraft or building
    starts with its discovery mark clear; ownership alone does not set it. However, placing an object in a cell's ground or bridge-deck layer marks that
    object discovered by *H*, regardless of ownership, shroud or fog. In the original this is
    the local player's house; our engine applies the same rule to every player house under
    claim 1. Thus an own vehicle placed by a map or trigger in unexplored usable ground is
    already discovered, as is an own vehicle or infantry produced by a factory. Placement in
    explored ground likewise needs no later cover-piece change. Placement outside the usable
    area clears the mark when the object's placement finishes. Discovery is not exploration:
    marking an enemy object discovered does not reveal its cell or bypass S-0410's enemy-object
    shroud tests.

    A reveal whose cover piece changes also marks one object per eligible cell discovered.
    For a height-0 reveal cell, the eligible cells are that cell plus (*n*, *n*) for integer
    *n* from 0 to 6, when the candidate cell's height level *h* satisfies
    2*n* − 1 ≤ *h* ≤ 2*n* + 1. Only infantry, vehicles, aircraft and buildings attached to the
    candidate's ground layer compete. Terrain objects, overlays, animations and projectiles
    do not compete; objects on the bridge-deck layer are not selected by this reveal test.
    The selected object has the smallest integer distance of F6, measured from the cell's
    top corner, not its centre. Equal distances favour the non-building most recently placed
    in that cell layer; any non-building precedes a tied building, and tied buildings favour
    the earliest placement in that layer. These are placement priorities, not object-number
    priorities. This selection never removes another object's existing discovery mark.
    - Backing: `measurable` for usable-ground vehicle placement discovery (V17, also the
      corrected S-0410 R9 part A). Placement discovery for infantry, aircraft, buildings,
      bridge-deck placements and outside-area clearing is `unmeasured` here: V17 tests only
      vehicles on usable ground, and no listed part isolates those other kinds or locations.
      The unset mark before map placement is `unmeasured` independently: an off-map newly
      created object is not listed on radar, so its mark cannot be distinguished there.
      The reveal-selection geometry, eligible kinds and tie priority are `unmeasured`: ordinary
      skirmish cell placement already marks the competing objects discovered, so selecting one
      again has no separately observable effect. A tree cannot hide a vehicle's radar mark.
    - Confidence: high for discovery on cell placement and the distinction from ownership,
      medium for outside-area clearing and the reveal-selection rules

21. **Shared off-map state in whole-map operations (Q-0111).** With
    `quirks.offmap_cells_shared` on, non-legal cells covered by gap fields affect one shared
    state; they are not ignored during claim 15's whole-map operations. The shared hostile gap
    count counts field-cell contributions, not merely generators. Wrapped coordinates that
    resolve to a legal cell affect that legal cell instead (claim 14). A previously untouched
    shared state is unexplored and unseen, with cover count 1 and hostile gap count 0. Neither
    whole-map reset nor whole-map reveal directly changes these shared quantities.

    The shared state's final result depends on the satellite state when the associated
    gap-field changes take effect. On gaining a satellite, the field changes occur while the
    satellite is inactive; the shared cover count is not lowered and the shared state does not
    become explored. On losing a satellite, the field changes occur while it is active; the
    shared state can become explored if the last hostile contribution ends. If hostile field
    coverage is then present again, a shared cover count of 0 becomes 1 and the shared state is
    unexplored, whereas a negative count remains negative and the shared state remains explored.
    These are final-state requirements, not requirements to expose an intermediate picture or
    to apply state-table rows in an internal order. With one running border field and an initially
    untouched shared state, neither gaining nor losing the satellite leaves the shared state
    explored after the operation. Stopping that field while the satellite is active does leave
    it explored (claim 14); losing the satellite afterwards, with no border field left, does not
    reset that exploration.
    - Backing: `measurable` for the final explored-state consequences (V18); `unmeasured` for
      the satellite state during removal considered separately, since no drawn frame separates
      the whole-map operation from its final field restoration, and for the exact shared counts
      and their negative-count outcomes: V18 isolates exploration from an untouched state,
      not separately observable counts or a setup that makes the shared count negative
    - Confidence: high for participation and the satellite-state timing; medium for the derived
      shared-state consequences, pending measurement

## Formulas

**F1. Sight radius** (claim 4). *S* is the sight field (integer), *Z* the world height in leptons
(integer), *P* the leptons per sight increase (integer), *v* the veteran sight factor (double).

> *q* = trunc(*Z* / *P*), integer division truncated toward zero.
>
> *b* = 10 · *q*, an integer that wraps: *b*′ = *b* reduced modulo 256 into −128 … 127
> (`quirks.sight_bonus_wrap`; off: *b*′ = *b*).
>
> *r*₀ = trunc( RZ₅₃( *S* × RZ₅₃( RZ₅₃( *b*′ × 0.01 ) + 1 ) ) ), where 0.01 is the double nearest
> to one hundredth and the three operations are done in double precision under the
> round-toward-zero mode of S-0205.
>
> *r* = S-0311 F4 applied to *r*₀ when the sight ability applies, otherwise *r*₀; then the radius
> used by claim 6 is min(*r*, 10), and a radius of 0 or less reveals nothing (Edge cases).

Example: *S* = 10 and *b*′ = 30 give 10 × 1.2999999999999998… = 12.999999999999998, so *r*₀ = 12,
not 13. (Under the cap of 10 both draw the radius-10 shape, so this example cannot be measured; the
measurable case is *S* = 5 and *b*′ = 20: 5 × 1.19999999999999995… = 5.9999999999999991, so *r*₀ = 5
under F1 where exact arithmetic gives 6, scenario V4.)

**F2. Sight centre and simulation lift** (claim 5). *X*, *Y*, *Z* are signed integer leptons,
*L* is an integer pixel lift and *k* an integer number of cells. Let *a* be the
binary64 constant nearest to 0.14350360082660085 pixels per lepton, and let *c(Z)* be 1 pixel
when *Z* ≥ 728 leptons, otherwise 0 pixels.

> *L(Z)* = trunc( RZ₅₃( RZ₅₃( RZ₅₃(*Z* · *a*) + *c(Z)* ) + 0.5 ) )
>
> *k* = trunc(*L(Z)* / 30)
>
> centre = ( trunc((*X* − 256*k*) / 256), trunc((*Y* − 256*k*) / 256) )

The lift product and both sums use floating point with a 53-bit significand, rounded toward zero
under S-0205's round-toward-zero mode. The grouping above matters: the correction is added to the
rounded product before the half pixel is added. The final conversion and both integer divisions
truncate toward zero, including for negative heights and coordinates. With
`quirks.chop_rounding` off, the three floating-point operations follow S-0205's alternative mode,
while the final truncation remains. No drawing-lift switch changes this formula.

Check values in pixels: *L*(0) = 0, *L*(104) = 15, *L*(727) = 104, *L*(728) = 105,
*L*(1976) = 285, *L*(2080) = 299 and *L*(−104) = −14.
For whole non-negative height levels on flat ground, *L*(104*h*) = 15*h* for 0 ≤ *h* ≤ 19,
so *k* = trunc(*h* / 2) there. This is a property of this simulation lift only:
at level 20, *k* = 9 instead of 10; the shortcut must not replace F2 at higher or negative levels.
S-0101 claim 3 defines the distinct exact lift used for drawing.

**F3. Height block** (claim 7). All integers. *T* = (*cx* + *dx*, *cy* + *dy*); *s* from table T2,
*u* = trunc(*Z* / 104).

> blocking cell = *T* + *s* + (2 + *c*ₓ − *cx*, 2 + *c*ᵧ − *cy*); *T* is hidden when the blocking
> cell's height level > *u* + 3.

**F4. Gap field** (claim 11). Integers. Cell (*cx* + *dx*, *cy* + *dy*) is covered when
*dx*² + *dy*² < (*R* + 1)².

**F5. Periods.** Regrowth period = trunc(RZ₅₃(*g* × 900)) ticks, fog period = trunc(RZ₅₃(*f* ×
900)) ticks, where *g* and *f* are the rates in game minutes (doubles).

**F6. Discovery selection distance** (claim 20). For an object's world position (*X*, *Y*),
let *a* = *X* mod 256 and *b* = *Y* mod 256, both integer leptons in 0 … 255. Its selection
distance is ⌊*r*(*a*² + *b*²)⌋ leptons, where *r* is S-0204 F1's table square root, under
`quirks.table_root_distance`. Squaring and addition are exact here. The original evaluates
these small integer squares and their sum in double precision, exactly, and truncates the
root toward zero to an integer. F6 ignores world height.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Largest sight radius | 10 | cells | claims 4 and 6 |
| Height bonus step | 10 | per cent per whole *P* leptons | claim 4, F1 |
| Height-bonus range (wraps) | −128 … 127 | per cent | F1, `quirks.sight_bonus_wrap` |
| Centre shift step | 30 | pixels of simulation lift per cell | claim 5, F2 |
| Simulation lift scale *a* | 0.14350360082660085 | pixels per lepton | Binary64 constant, F2 |
| Simulation lift correction threshold | 728 | leptons | F2 adds 1 pixel at or above this height |
| Simulation lift rounding addition | 0.5 | pixels | Before final truncation, F2 |
| Height-block threshold | 3 | height levels above the observer level | claim 7 |
| Height-block displacement | (2, 2) | cells | claim 7 |
| Moving sight refresh | 15 | ticks | claim 8 |
| Re-cover sweep period | 120 | ticks | claim 13 |
| Reveal-on-fire radius | 3 | cells | claim 16 |
| Initial cover count | 1 | count | claim 3, S1 |
| Height level | 104 | leptons | glossary |
| Ticks per game minute | 900 | ticks | glossary; F5 |

**T1. Half-widths *w*ᵣ(|*dy*|) of the sight shape** (claim 6). Row *r*, columns |*dy*| = 0, 1, 2, …,
a cell (*dx*, *dy*) is covered when |*dy*| ≤ *r* and |*dx*| ≤ the entry.

| *r* | half-widths for \|*dy*\| = 0 … *r* | Differs from *dx*² + *dy*² ≤ *r*² + 1 |
|---|---|---|
| 1 | 1, 1 | no |
| 2 | 2, 2, 1 | no |
| 3 | 3, 3, 2, 1 | no |
| 4 | 4, 4, 3, 3, 1 | adds (±3, ±3) |
| 5 | 5, 5, 4, 4, 3, 1 | no |
| 6 | 6, 6, 5, 5, 4, 3, 1 | no |
| 7 | 7, 7, 6, 6, 5, 5, 3, 1 | no |
| 8 | 8, 8, 7, 7, 6, 6, 5, 3, 1 | lacks (±7, ±4) and (±4, ±7) |
| 9 | 9, 9, 8, 8, 7, 7, 6, 5, 3, 1 | lacks (±8, ±4) and (±4, ±8) |
| 10 | 10, 10, 9, 9, 8, 8, 7, 7, 5, 3, 1 | lacks (±9, ±4), (±4, ±9), (±8, ±6) and (±6, ±8) |

**T2. Step towards the centre** (claim 7). For a displacement (*dx*, *dy*) other than (0, 0), *s* is
the step direction (glossary) whose direction is nearest in angle, in the map plane, to the
direction from *T* back to the centre, (−*dx*, −*dy*); no displacement of T1 lies exactly halfway
between two step directions. Nine displacements take another step instead (`quirks.height_block_cell`):

| (*dx*, *dy*) | *s* | Nearest step |
|---|---|---|
| (5, −2) | (1, 0) | (−1, 0) |
| (−3, −7) | (0, 1) | (1, 1) |
| (3, −7) | (0, 1) | (−1, 1) |
| (−7, −3) | (1, 0) | (1, 1) |
| (7, −3) | (−1, 0) | (−1, 1) |
| (−7, 3) | (1, 0) | (1, −1) |
| (7, 3) | (−1, 0) | (−1, −1) |
| (−3, 7) | (0, −1) | (1, −1) |
| (3, 7) | (0, −1) | (−1, −1) |

The displacement (0, 0) has *s* = (0, 0).

## State and ordering

**S1. Cover count of one cell for house H** (claim 12). *k* is the cover count, *G* the hostile gap
count, *m* the pending mark (not the leptons per sight increase of F1); *k*′ and *G*′ are the values after the event. The seen state changes
only where a row says so (rows 1, 4, 6, 7, 8, 9): a cell made explored or unexplored by a row that
does not mention it keeps its seen state.

| # | Event | Change of *k* and *G* | Result for the cell |
|---|---|---|---|
| 1 | A sight of *H* is added over the cell | *k*′ = −1 if *k* = 1, otherwise *k*′ = *k* − 1 | If now *k*′ ≤ 0: explored, seen and marked surrounded, and *m* cleared if the cell was already explored and marked surrounded before the event (so the first sight over a cell that is not explored or not marked surrounded keeps *m*) |
| 2 | A sight of *H* is removed from the cell | *k*⁺ = 1 if *k* = −1, otherwise *k*⁺ = *k* + 1; *k*′ = min(*k*⁺, *G*) | If *k* went from 0 or less to above 0: *m* set |
| 3 | A gap field hostile to *H* starts over the cell | *k*′ = *k* + 1 if *k* = 0 or *k* ≥ 2, otherwise *k*′ = *k*; *G*′ = *G* + 1 | If now *k*′ ≥ 1: unexplored at once; the seen state is unchanged |
| 4 | Re-cover sweep, cell has *m* | none | unexplored and unseen; *m* cleared |
| 5 | A gap field hostile to *H* stops over the cell | *G*′ = *G* − 1; *k*′ = *k* − 1 if *H*'s spy satellite is active and *G*′ ≤ 0, otherwise *k*′ = *k* | If that made *k*′ ≤ 0: explored and marked surrounded; the seen state is unchanged |
| 6 | Plain reveal or area reveal | Plain reveal: *k*′ = *k*, *G*′ = *G*. Area reveal: *k*₁ = −1 when *k* = 1, and *k* − 1 otherwise; *k*₂ = 1 when *k*₁ = −1, and *k*₁ + 1 otherwise; *k*′ = min(*k*₂, *G*), *G*′ = *G* | Area reveal: explored, seen and marked surrounded; plain reveal: explored and marked surrounded but not made seen, and with *k* > 0: *m* set |
| 7 | Whole-map re-cover of *H* (claim 15) | afterwards *k*′ = −*n* when *n* ≥ 1 re-applied sights of *H* cover the cell, otherwise *k*′ = 1; *G*′ = the number of gap fields hostile to *H* that cover it | explored and seen when *n* ≥ 1, otherwise unexplored and unseen; *m* kept, and set where claim 15 creates a mark; the first sight over the cell does not clear *m* (the cell is unexplored when it arrives) and a second one does, so *m* survives only where *n* ≤ 1 |
| 8 | Spy satellite's whole-map reveal of *H* (claim 15) | as row 7, except that a cell that no re-applied sight covers has *k*′ = 1 when *G*′ ≥ 1 and *k*′ = 0 when *G*′ = 0 | explored, seen and marked surrounded, except a cell with *k*′ = 1, which is unexplored but still seen (row 3 leaves the seen state unchanged); a pre-existing *m* survives with *n* = 0 and is cleared with *n* ≥ 2; survival with *n* = 1 and creation of new marks during a reveal are not established (claim 15) |
| 9 | Uncounted sight of *H* over the cell (Edge cases) | only when the cell is not yet explored, seen and surrounded with all neighbours seen: as row 1 | as row 1 |
| 10 | Explored-only whole-map reveal of *H* (claim 3 at game start; defeat, claim 15) | none | explored, not made seen; existing surrounded marks kept, and surrounded marks set where all eight map neighbours are explored. Non-legal neighbours read claim 14's shared explored state under `quirks.offmap_cells_shared`: when explored, border cells gain the mark too; otherwise previously unmarked border cells remain unmarked. *m* kept (at game start no cell has a pending mark) |

Consequences: with no hostile field (*G* = 0), removing a sight brings *k* back to at most 0, so the
cell stays explored. Several sights over one cell count down (*k* = −2 for two); the cell becomes
pending only when the last one leaves. Row 5 with an active spy satellite on a cell that a sight
covers makes *k* more negative than the number of sights, so that cell then needs one more removal
before it can become pending; this lasts until the next whole-map re-cover or spy-satellite reveal (Edge cases).

**Ordering.** The re-cover sweep comes before the turn pass of its tick (claim 13). On a tick
when regrowth is also due, including tick 0, claim 18's regrowth result precedes that sweep:
previously pending explored cells can start neighbour erosion rather than having their marks
consumed before regrowth can affect them (V11 F). Sights are added
and removed in the turn of the object concerned (claim 8). A gap field starts or stops when its
generator enters or leaves operation or enters or leaves the map (claim 11); for buildings the
in-operation test is S-0405's (claims 8 and 9). An accepted power-off order also stops the field
immediately, and a charged building can start it again on its next turn without an operation
transition (claim 11; S-0405 claim 10). Within one tick, everything a reveal changes is visible to every later rule of that tick.
No rule depends on the order in which the cells of one sight are processed.

## Edge cases and quirks

Entries labelled fixed rules below are not optional corrections. Only entries naming a
`quirks.<name>` setting offer an alternative, always with original behaviour as the default.

- **`quirks.gap_radius_wrap`** (default on, the original). A gap radius field value is read
  reduced modulo 256 into −128 … 127 cells (128 reads as −128, 200 as −56, 260 as 4),
  so a gap generator type with a radius from 128 to 255 has a field that covers nothing
  (claim 11). Off: the radius is used as given, with no wrap. The importer warns in either case
  when the value lies outside −128 … 127. Distinct from `quirks.sight_bonus_wrap`, which wraps
  the height bonus only.
- **`quirks.sight_bonus_wrap`** (default on, the original). The height bonus *b* wraps into
  −128 … 127 (F1), so 10 · trunc(*Z* / *P*) above 127 wraps: at *q* = 13 it reads −126, which
  makes the factor negative. Off: *b* is used without the wrap. With the absent-key default of 50 leptons a unit at 650 leptons
  or higher does wrap. Where the wrapped factor makes *r*₀ negative, the original's result is
  undefined: our engine reveals nothing for that sight, and the
  importer warns when the rules' leptons per sight increase is below 160, which allows a wrap
  below 2080 leptons (a *departure* for bad data). A value of 0 is also bad data (a division by
  zero): our engine uses the absent-key default 50 and warns.
- **`quirks.building_sight_top_left`** (default on). A building's sight, area reveal and gap field
  are centred on its top-left foundation cell's centre, so a large building sees farther towards
  map north-west than towards south-east. Its reveal on fire (claim 16) uses its reference point
  instead. Off: they are all centred on its reference point (S-0204, claim 4).
- **`quirks.sight_centre_off_map`** (default on). A sight or area reveal whose centre (after the
  height shift of claim 5) is not a legal cell reveals nothing, even where part of its shape lies on
  legal cells; this can happen to a unit high in the air near the map's top edges. Off: the legal
  cells of the shape are revealed as usual.
- **`quirks.height_block_cell`** (default on). The height block tests one cell, displaced by (2, 2)
  plus the centre shift from the neighbour towards the centre, and nine displacements of table T2
  use a step that does not point towards the centre (one, (5, −2), points away from it). Off: the
  blocking cell is *T* + *s* with *s* the nearest step direction towards the centre for every
  displacement (no displacement, no exceptions). The block applies unless the rules clear the
  flag, which is set when absent.
- **`quirks.gap_end_keeps_cover`** (default on). When a hostile gap field stops, its cells stay
  unexplored for *H* until a sight of *H* reaches them, unless *H* has an active spy satellite.
  Off: each cell that was explored when the field started, and that the field covered, becomes
  explored again when the field stops. Related: with a spy satellite active, row 5 of S1 can leave
  a cell's count one step too low (State and ordering); the next whole-map re-cover resets it.
- **`quirks.fire_reveal_expires`** (default on). The reveal on fire is a plain reveal, and plain
  reveals of cells never covered by a sight of *H* are undone by the next re-cover sweep, so the
  attacker's area is shown for 1 to 120 ticks with regrowth off, depending on the tick number.
  Regrowth may re-cover it sooner (claim 18). Off: the reveal on
  fire behaves like an area reveal (S1 row 6, area reveal) and is permanent outside hostile gap fields.
  Inside them it follows claim 17.
- **Fixed spy-satellite rule (not an optional quirk).** Claim 15 deliberately does not use
  S-0405's in-operation test: low power and electromagnetic locks do not stop the satellite.
  This is ordinary compatibility behaviour with no switch or alternative; it is not classified
  as a defect to correct. Backing: `measurable` (V9 C2 and C3); confidence: medium.
- **Fixed passive-country rule (not an optional quirk).** Objects whose owners' countries are
  marked passive reveal nothing, by claim 2, including neutral and special houses with that
  flag. This is imported game behaviour, not a defect or an ownership-role exception; it has
  no switch or alternative. Backing: `measurable` (V3 and V7 E); confidence: high for the
  passive flag, medium for precedence over reveal-to-all
- **Ally reveal and alliance changes.** When two houses ally or stop being allied, the new
  alliance takes effect for buildings and gap fields on that tick: for the houses concerned, the
  re-applied sights (claim 15) and the gap fields count as taken back under the old alliance (S1
  rows 2 and 5) and applied again under the new one (S1 rows 1 and 3). No cell is reset here: a
  mark that the taking back sets is cleared again where a sight comes back (the cell is still
  explored and surrounded: S1 row 1), so the change leaves no cell pending that a sight covers. Allied units that are not buildings
  follow only at their next refresh (`quirks.recover_skips_allied_units`). This paragraph is
  `measurable` for buildings and gap fields by V15; `unmeasured` for standing allied units'
  transition after an alliance change, since V15 excludes their sights and no scenario here
  isolates that transition (confidence: low). S-0414 claims 12 to 14 allow an alliance change during
  skirmish when the two rules flags permit it; lobby alliances are not the only possible ones.
- **`quirks.recover_skips_allied_units`** (default on). The removal and re-application of claim 15
  (whole-map reveal and re-cover, alliance changes) and, with fog on, the fog test of claim 19 take in the sights
  of *H*'s own objects and of allied and reveal-to-all buildings, but not those of allied units
  that are not buildings. After a whole-map re-cover the cells that only a standing allied unit
  sees stay unexplored until that unit next refreshes its sight (claim 8, usually when it moves).
  Off: every sight that counts for *H* by claim 2 is removed and re-applied.
- **`quirks.recover_keeps_pending`** (default on). The whole-map re-cover and the spy satellite's
  whole-map reveal leave pending re-cover marks in place, and a sight that comes back clears one
  only on a cell that is explored and marked surrounded when it arrives (claim 15 gives the
  result per number of sights; the creation-order prediction for newly pending cells is low
  confidence and does not prescribe an internal event sequence). After a re-cover a cell holding a
  mark under exactly one re-applied sight is explored by it with the mark kept, so the next
  re-cover sweep makes the cell unexplored although a sight of *H* covers it; it becomes explored
  again only when a sight is next added over it. A cell under two or more re-applied sights loses
  its mark. After a reveal, marks under at least two re-applied sights are cleared; without a sight
  they remain. Survival under exactly one sight is not established (claim 15). A mark held before needs a cell that was
  pending at that moment (a sight left it inside a hostile gap field, or a reveal on fire touched
  it, less than 120 ticks before). Off: the re-cover and the reveal clear every pending mark of
  the house once the sights and fields have come back, so neither the earlier marks nor the
  sequence of the objects matters.
- **`quirks.uncounted_sight`** (default on, low confidence). Besides the sights of claim 8, the
  original applies an *uncounted sight* at a few moments: to the local player's units that are not
  buildings when the local area is set up at the start of play, to a unit that comes out of the
  ground (tunnel), and to *H*'s own objects and allied buildings after each regrowth step (claim
  18). It reveals the same cells as a sight (claims 4 to 7), but it is never removed, and it counts
  down the cover count (S1 row 9) only on cells that are not already explored, seen and surrounded
  with every neighbour seen, which are the cells at the rim of the area *H* sees. Each such cell
  keeps a count one lower for good, so it stays explored when the sights over it leave, even
  inside a hostile gap field (S1 rows 2 and 3), until the next whole-map re-cover or spy-satellite reveal (both reset the count). Whether the
  start-of-play case finds any unit already on the map was not established (Open questions). Off:
  no uncounted sight is applied; the start of play and the tunnel exit rely on the sights of claim
  8, and regrowth re-applies the sights as counted sights.
- **Fixed sight-boundary rule (not an optional quirk).** Unlike S-0110's point test and gap
  fields (claim 14), a sight or area reveal never reaches a cell across the 512-cell coordinate
  range: every covered cell must be legal, and no cell outside 0 … 511 is legal. This is
  ordinary boundary behaviour, not a defect to correct, and has no switch or alternative.
  Backing: `measurable` (V8 D2, falsified if the sight explores a cell at the opposite end of
  the map); confidence: high
- **`quirks.offmap_cells_shared`** (default on, low confidence). In the original every cell that
  is not legal has the same explored state for *H*: a change to that state at one non-legal cell
  is a change at all of them. A gap field whose cells include non-legal cells changes that common
  state: when such a hostile field stops while *H* has an active spy satellite and no
  other hostile field still covers a non-legal cell (S1 row 5), the shared state becomes explored.
  Every non-legal cell then reads as explored for *H*, until another gap change covers the common
  state again. Whole-map operations reset legal cells only, but their field removals and
  restorations affect the common state as claim 21 gives. S-0110's border pieces and
  point test then treat non-legal cells as explored, which is the one exception to S-0110's
  general statement (claim 1) that a non-legal cell is never explored. Off: non-legal cells are
  never explored.
- **`quirks.spy_satellite_first_building_decides`** (default on). Only the first qualifying
  spy-satellite building is looked at (claim 15): when it is being erased, the whole-map reveal ends
  even if another spy-satellite building is not. Off: the satellite is active while any qualifying
  building is not being erased. S-0410 has the same rule for radar
  (`quirks.radar_first_building_decides`).
- **`quirks.defeat_reveal_keeps_pending`** (default on, low confidence). The defeat reveal (claim 15)
  leaves the defeated house's pending re-cover marks in place, and the re-cover sweep does not skip
  defeated houses, so the cells that were pending at the moment of defeat become unexplored at the
  next sweep and stay so. Off: the defeat reveal clears every pending mark of the house, so the
  whole map stays explored for it.
- **`quirks.shroud_point_cell_wrap`** (S-0110, shared; default on). Gap fields (claim 14) read a
  cell with a coordinate outside 0 … 511 as another cell, exactly as S-0110's point test does; the
  one switch controls both. Off: such a cell is read as not legal and changes nothing. Sights never
  wrap (see No wrap for sights above).
- **Campaign-only behaviour** (out of scope): in a campaign mission the original reveals for objects
  of passive countries, ties some tests to whether an object has been discovered, and keeps fog off
  unless the map sets it.

## Import mapping

| User data | Feeds | Form | Absent |
|---|---|---|---|
| Rules, type section, `Sight` | sight field *S* (claims 4, 8) | integer, cells | 0 |
| Rules, type section, `RevealToAll` | reveal-to-all flag (claim 8) | boolean | no |
| Rules, type section, `GapGenerator` | gap-generator flag (claim 11) | boolean | no |
| Rules, type section, `GapRadiusInCells` | gap radius *R* (claim 11) | integer, cells, wrapped into −128 … 127 by `quirks.gap_radius_wrap` | 0 |
| Rules, building type section, `SuperGapRadiusInCells` | super-gap radius (S-0405 claim 16 owns the import) | integer, cells | 0 |
| No user key | simulation lift scale, correction and rounding for the sight centre (claim 5, F2) | engine constants and arithmetic, not rules data | always F2, independent of drawing lift |
| Map, cell data | terrain height contributing to world height *Z* (claim 5) | signed integer height levels, converted to leptons by S-0204 claim 1 | level 0, as in S-0009 claim 7 |
| No user key | discovery on cell placement and reveal-selection priority (claim 20) | engine rule, not configurable through data | always applies in a skirmish |
| No user key | shared-state participation and satellite-state timing (claim 21) | engine rule under `quirks.offmap_cells_shared` | original behaviour |
| No user key | surrounded marks after explored-only whole-map reveal, including border cells (claims 3, 15 and 18; S1 row 10) | engine rule; non-legal neighbours use the shared explored state under `quirks.offmap_cells_shared` | no independent setting; with that quirk off, non-legal neighbours are unexplored |
| Rules, building type section, `SpySat` | spy-satellite flag (claim 15) | boolean | no |
| Rules, building type section, `Radar` | radar flag, for infiltration (claim 15) and S-0410 | boolean | no |
| Rules, weapon section, `RevealOnFire` | reveal-on-fire flag (claim 16) | boolean | no |
| Rules, `[General]`, `LeptonsPerSightIncrease` | leptons per sight increase *P* (F1) | integer, leptons | 50 |
| Rules, `[General]`, `RevealByHeight` | reveal-by-height flag (claim 7) | boolean | yes |
| Rules, `[General]`, `VeteranSight` | veteran sight factor (S-0311 F4) | decimal | 1.0 |
| Rules, `[AudioVisual]`, `AllyReveal` | ally reveal (claim 2) | boolean | yes |
| Rules, `[AudioVisual]`, `ShroudGrow` | regrowth flag (claim 18) | boolean | no |
| Rules, `[AudioVisual]`, `ShroudRate` | regrowth rate *g* (F5) | decimal, game minutes | 4.0 |
| Rules, `[AudioVisual]`, `FogRate` | fog rate *f* (F5); read only with fog on, which a skirmish never has (claim 19) | decimal, game minutes | 0.05 |
| Rules, `[MultiplayerDialogSettings]`, `Shroud` | lobby shroud setting (claim 3) | boolean | yes |
| Rules, `[MultiplayerDialogSettings]`, `FogOfWar` | no effect in a skirmish: starting a skirmish turns the setting off (claim 19) | boolean | no |
| Map, special-flags section, `FogOfWar` | ignored in a skirmish (claim 19) | boolean | no |
| Rules, country section, `MultiplayPassive` | passive country (claim 2) | boolean | no |

The parse forms and absent-key defaults are `unmeasured` here (confidence: high): the
scenarios override selected fields, but none systematically omits each key and rules layer to
isolate its fallback. Observable consequences of imported fields are backed by the numbered
claims and their scenarios. Bad-data importer warnings and quirk-off alternatives are design
rules, not measurements of the original.

All rules keys follow the layering of S-0006. The absent-key defaults of this table (50, 4.0, 0.05, 1.0 and the yes/no values) are built into the program, not read from any rules file. Other visibility keys of the rules (the fog blending
and aircraft fog reveal keys, the attacking-aircraft sight range, the reveal radius of the reveal
trigger and the drop zone) belong to the specs that use them; the subterranean shroud-move key
belongs to S-0214.

## Measurement scenarios

**Common setup.** The glossary's *measurement setup*, a skirmish with one human player and one
bystander computer player (glossary) on a purpose-made map of one theater, flat ground at level 0 unless stated, fog of
war off, regrowth off unless a part enables it, any game speed. The director and mods are off,
except for each scenario's explicitly named test data. Explored cells are read from screenshots (S-0110 claims 1 and 2; the test
shroud sprite of S-0110's scenarios may be used so that each cell's state can be read from its own
diamond). Tick numbers come from a *recorder* that reads the number of ticks run. The test map's
override section gives the test types their fields (named test data). One valid run per case
unless stated. Unless a part explicitly leaves its outcome open, any difference between its
recorded explored-cell set or timing and its stated expectation falsifies that part's hypothesis.
Each scenario's observed area is the set of cells it reads, together with its test
objects; the bystander's void rule applies, except to the bystander objects a scenario names (its
gap generators, attackers, spy team and allied units). "Computer ally" and "computer enemy" below
mean an allied and an enemy bystander computer player.

- **V1 (claim 3).** Part A: start a game; screenshot at tick 1 before any unit moves. Expected:
  only the cells of the starting units' sights (and building placements) are explored. Part B:
  named test copy of the rules with the lobby shroud setting off. Expected: the whole map is
  explored at the first drawn frame. Falsified if: part A shows other explored cells, part B shows
  shroud anywhere, or the skirmish lobby shows a shroud control.
- **V2 (claims 4 and 6).** Thirteen test infantry types, made by the override section from one stock
  type, with sight fields 0, 1, 2, …, 12; one unit of each, standing still on flat level-0 ground far
  from the others and from the human player's other units, veteran sight factor 1. Observable: the set of explored
  cells around each, read against the cell grid. Expected: no reveal for 0, the shapes of T1
  for 1 to 10, and the
  shape of 10 for 11 and 12. Falsified if: any cell differs from T1, or 11 or 12 exceed the
  radius-10 shape, or the sight-0 unit reveals any cell.
- **V3 (claim 2).** The human player, one allied bystander computer player (same lobby team) and one enemy
  one. The map places units of the ally in every run, far from the human's, so no run depends on
  starting units (a passive country may get none). A neutral-house vehicle is placed by the map
  far from everyone; the named test rules explicitly mark its country passive. Runs, each
  set by a named test copy of the rules: (1) ally reveal on; (2) ally
  reveal off; (3) ally reveal on and the computer ally's country marked passive for multiplayer.
  Expected: in (1) the ally's units' surroundings are explored for the human player; in (2) and (3)
  they are not; the neutral vehicle's surroundings are explored in none of the runs. Falsified if:
  any differs.
- **V4 (claims 4 and 5, height bonus and rounding).** Named test copy of the rules with leptons per
  sight increase set to 104 (one height level) and veteran sight factor 1. An infantry unit of sight
  field 5 stands (run 1) at level 0; (run 2) on a flat plateau at level 3 (*Z* = 312, *q* = 3, *b* =
  30, centre shift *k* = 1); (run 3) on a flat plateau at level 2 (*Z* = 208, *q* = 2, *b* = 20,
  *k* = 1). The plateaus are wider than the radius-6 shape. Observable: the explored shape around
  the shifted centre. Expected: run 1 radius 5; run 2 radius 6 (5 × 1.3 is 6 under F1 and
  exactly); run 3 radius 5, because under F1 the double steps give 5.9999999999999991 and the result
  truncates to 5, where exact arithmetic would give 6. Falsified if: run 2 shows radius 5 (the bonus
  is not applied), or run 3 shows radius 6 (F1's round-toward-zero arithmetic is not reproduced:
  the engine must then use exact or round-to-nearest arithmetic here). The sight field 10 case of
  F1's example cannot be used: the cap of claim 6 hides it.
  Run 4 (the wrap of *b*, `quirks.sight_bonus_wrap`; named test copy of the rules with leptons per
  sight increase set to 10 and veteran sight factor 1): the infantry unit of sight field 5 stands
  on a flat plateau at level 2 (*Z* = 208, *q* = 20, *b* = 200, *b*′ = −56, centre shift
  *k* = 1) that is wider than the radius-10 shape; everywhere else the map is flat level 0, and no
  aircraft or jumpjet unit is on it, the bystander's included. With this setting an object at a
  world height of 130 to 159 leptons (or 390 to 419) would get a negative *r*₀, which is
  undefined in the original (Edge cases), so a run in which any object reaches such a height is
  void. Expected: radius 2 (5 × RZ₅₃(RZ₅₃(−0.56) + 1) = 2.2000000000000002, truncated to 2).
  Falsified if: radius 10 (no wrap: 5 × 3 = 15, capped), or any other radius, for example the
  radius 10 that a clamp of *b* to 127 gives. Run 5 (a wrap to a positive value): the same copy
  and a plateau at level 3 (*Z* = 312, *q* = 31, *b* = 310, *b*′ = 54, *k* = 1), the same void
  condition. Expected: radius 7 (5 × 1.54 = 7.6999999999999984, truncated to 7). Falsified if:
  radius 10 (no wrap: 5 × 4.1, capped), or another radius.
- **V5 (claim 5).** A plateau 4 levels high (flat top). An infantry unit stands on a flat cell of the
  plateau (*Z* = 416, *L* = 60, *k* = 2). Observable: the centre of its explored area relative to
  its own cell. Expected: centred on the cell two steps map north-west (−2, −2) of its own cell.
  A second run on a 3-level plateau: *k* = 1, centred one step north-west. A third run: a building
  with a 3 × 3 foundation at level 0: centred on its top-left cell. Part D (off-map case): a unit on
  a 4-level plateau whose own cell is legal and lies one cell from the edge of the legal area in the
  map-north-west direction, so that its shifted centre (two cells further) is not legal, with sight
  field 5. Expected: nothing is explored around it, though part of its shape would lie on legal
  cells. Falsified if: centred on the unit's own cell (or on the foundation centre), or shifted by
  another amount, or (part D) any cell is explored.
  Part E (Q-0460, simulation lift): two fresh runs on purpose-made maps with a broad flat plateau,
  at level 19 and level 20 respectively. One map-placed human infantry unit stands on the same
  cell (*u*, *v*) in each run, far enough from all legal and usable boundaries that both candidate
  centres and their radius-1 shapes are legal. Other human objects and allied sights are far away.
  Named test-map overrides set its sight field to 1, leptons per sight increase to 100000,
  veteran sight factor to 1 and reveal-by-height off, isolating the centre from radius bonuses
  and terrain blocking. Keep the unit stationary and record the explored-cell set in the first
  drawn frame, at any game speed, 800×600. Expected in both runs: the radius-1 shape of T1 centred
  on (*u* − 9, *v* − 9), since the lifts are 285 and 299 pixels. At level 20 the exact drawing
  lift would instead centre it on (*u* − 10, *v* − 10). Any other explored set falsifies F2,
  particularly that latter centre at level 20. One valid run per height; if the original rejects
  these map heights, record that and leave this discriminating part unconfirmed.
- **V6 (claim 7).** Flat level-0 ground with one tall column of single cells at level 8 (a
  one-cell-wide wall of high ground) placed so that it is the blocking cell under F3 for some cells
  of an observer's shape but not the cell one step towards the centre, and a second layout in which
  it is the cell one step towards the centre but not the F3 cell. An infantry unit of sight 7 at
  level 0 (observer level 0, threshold 3). Observable: which shape cells stay unexplored. Expected:
  in layout 1 exactly the cells whose F3 blocking cell is a column cell stay unexplored; in layout
  2 none does. A third run with a named test copy of the rules setting the reveal-by-height flag
  off: no cell is hidden. Falsified if: layout 2 hides cells (the "towards the centre" alternative)
  or layout 1 hides none, or cells along the whole line are hidden.
  Part D: use sight radius 10 and, in nine separate fresh runs, place the high column at the
  exceptional blocking cell for one displacement of T2, with its nearest-step alternative at
  level 0. Read the displaced target cell. Expected: it is hidden under F3 with the T2 exception,
  not the nearest-step alternative. Falsified if any sampled target remains explored. Part E:
  repeat layout 1 with the observer at level 5, recalculating its shifted centre and blocking
  cells under F3. Expected: a level-8 blocking cell no longer hides a target (8 is not greater
  than observer level 5 + 3). Falsified if the target remains hidden.
- **V7 (claims 8 to 10).** Part A: a human vehicle drives a straight route of 10 cells; repeat
  with ordinary walking infantry. Capture every drawn frame and record per-tick positions.
  For infantry also record its walking target so the standing-position snaps of S-0208
  claims 6 and 23 can be distinguished from boundary crossings and final walk completion.
  Observable: the tick on which each new cell enters the explored area. Expected: arrival
  refreshes use the position reached on that tick, including intermediate standing-position
  arrivals, not the earlier boundary-crossing position. The explored area is the union of
  the shapes revealed at arrivals and at the separate moving refreshes (the first tick of
  motion and every 15 ticks thereafter, claim 8). Repeat S-0208 W13's same-cell walk:
  its standing-position snap counts as an arrival too, but on flat ground with unchanged
  sight centre and radius no new explored cell is required; that repeat alone cannot
  isolate the refresh. Falsified if the inter-cell run's explored set or timing differs
  from that union after accounting for periodic refreshes. Part B: a human aircraft flies across unexplored ground; sample only sustained flight,
  outside take-off and landing, and use a sufficiently long route for three periodic refreshes.
  Expected: explored area grows in steps 15 ticks apart. A second run of part B: the aircraft is ordered to stop (a
  hover or a landing) and 5 ticks later to move again; expected: the next growth comes 15 ticks
  after the previous periodic refresh, not 5 ticks after the restart, provided the stop and
  restart introduce no separate locomotor refresh. If take-off or landing refreshes intervene,
  record the result but do not use that run to confirm or falsify the short-stop timer rule. Part C: a map places a human unit outside the local area and
  orders it in (scripted input); expected: nothing is explored around it before it enters the
  usable area. Part D: the units leave; expected: the explored cells stay explored. Part E (named test data):
  a computer-owned test building type with the reveal-to-all flag and a large sight field, placed by
  the map far from the human's units. Expected: its surroundings are explored for the human player
  although it is an enemy building; a repeat with the computer's country marked passive (the building is placed by the map in
  both runs): nothing is explored. Part F: a human engineer captures a computer building (stock) whose sight field is larger than the
  engineer's, in unexplored ground. Expected: after the capture the cells between the engineer's
  sight and the building's sight radius become explored. Falsified if: any differs.
- **V8 (claims 11 to 14).** The computer enemy owns a powered stock gap generator next to an area the
  human player has explored. Part A: the generator is built after the human unit has
  explored the area and left (a map trigger creates it, or the computer builds it): expected: the field's
  cells (F4) become unexplored on the tick the generator powers on; cells inside the human unit's
  current sight stay explored. Part B: a human unit walks into the field and out again; the
  recorder logs the tick on which the cells it left become unexplored. Expected: a tick number
  that is a multiple of 120, the first after it left. Part B2: as part B, but the unit is killed
  inside the field (a computer unit placed by the map attacks it); expected: its cells become
  unexplored at the first multiple of 120 after its death. Part C: the generator's power plant is
  destroyed (the generator loses power): expected: the field's cells stay unexplored; repeat with
  the human player owning an active spy satellite: they become explored. Part D: on a map of size 258 × 254 (width plus height = 512, as S-0110 scenario SH8),
  an enemy gap generator stands near the corner cell (1, 258) and the human player has explored the
  area around the legal cell (511, 255) at the opposite end: expected: cells there inside the wrapped
  field (claim 14) become unexplored when the generator powers on. Part D2 (sights do not wrap): on the same map, a
  human unit with sight field 10 stands three cells from the corner cell (1, 258) along the edge,
  so that a wrapping read would reach the area around (511, 255); expected: no cell there is
  explored by it. Part E: as part A, with the
  generator owned by the computer ally (same lobby team) instead. Expected: no cell becomes
  unexplored. Part F: as part A, after the human player is defeated (V9 part F), with no active
  spy satellite and a named test rules file lengthening the end delay to 900 ticks (S-0414
  claim 8). During that delay a map trigger creates the enemy's powered generator. Expected:
  no cell changes when the generator powers on. Part G (named test data): gap generator types whose radius
  field is 200 and 260, plus two controls with fields 0 and −1, each in a fresh run.
  Expected under `quirks.gap_radius_wrap`: the 200 and −1 generators cover no cell, the
  260 generator covers the cells of F4 for radius 4, and the 0 generator covers only its centre.
  Falsified if either control covers a different set. Part H (the shared state of the non-legal cells,
  `quirks.offmap_cells_shared`; low confidence): the human player owns an active spy satellite
  and a power plant, the whole map is explored, and the border cells of the map's diamond show the
  edge pieces of S-0110 claim 1 towards their non-legal neighbours (S-0110's border sub-test). The
  computer enemy's powered stock gap generator stands so that its field (F4) covers non-legal
  cells beside the border and no other hostile field covers any; it is then put out of operation
  by destroying its power plant. Observable: the shroud pieces of the border cells that lie
  outside the field, and S-0110's scenario SH8 part Q repeated afterwards. Expected under the
  quirk: before the generator stops, the border cells show the edge pieces towards the outside,
  and after it stops they show none (every neighbour reads as explored) and they stay so after a
  later whole-map re-cover; in the repeated part Q the non-legal candidate counts as explored.
  Falsified if: the edge pieces towards the outside remain after the generator stops, or they
  vanish without a hostile field having covered non-legal cells, or without the spy satellite,
  or return after the later whole-map re-cover, or the repeated part Q still treats its
  non-legal candidate as unexplored.
  Falsified (for the other parts) if: re-cover happens on a tick
  that is not a multiple of 120, cells inside current sight are covered, or the end of the field
  re-explores cells without a spy satellite, or part D shows no wrapped cover, or part D2 explores a cell, or part E or F
  covers cells, or in part G the 200 generator covers cells (no wrap, or a clamp to 127) or the
  260 generator covers anything but the radius-4 field.
- **V9 (claim 15).** Part A: the human player owns a stock spy satellite building, placed by the
  map, and a power plant. Expected: the whole map is explored from the first tick. The building is
  then destroyed (a computer unit placed next to it by the map attacks it): every cell outside the
  human player's current sights becomes unexplored on the tick of the radar-state recheck that
  follows its death (S-0410, claim 2), and cells under the sights stay explored (the pending-mark
  case is V13 part D). Part B: the victim of a radar
  infiltration must be the local player, so a map trigger creates a computer team whose spy enters
  the human player's radar building (the team actions are handed to other specs by S-0011, claim
  30). Expected: the human player's map is re-covered except the current sights; a repeat with an
  active spy satellite changes nothing. A run of part B in which the spy does not reach the building
  is void. Part C (named test data, as S-0410 scenario R2): the human player owns two spy-satellite
  buildings, and a test erasing weapon (a temporal weapon) is fired at the first one only. Expected: the
  whole map is re-covered while it is being erased; a repeat on the second one only changes nothing. Part C2: the same with an electromagnetic pulse
  weapon on the first building: nothing changes, the whole map stays explored. Part C3: the same
  with the human's house at low power: nothing changes.
  Part D: as part A, with a powered computer gap generator, placed by the map, far from the human
  player's units. Expected: when the satellite becomes active, every cell is explored except the
  gap generator's field (F4), which stays unexplored. Part E: as part B (or part A's destruction),
  with a computer ally (ally reveal on) whose infantry unit stands still in an area only it sees and
  whose building stands in another such area. Expected: after the re-cover the ally's building's
  surroundings are explored again on the same tick, the standing unit's are not until it is ordered
  to move (scripted input on the ally is not possible, so the map's trigger moves it).
  Parts F, F2 and F3 use a named test rules file lengthening the end delay to 900 ticks (S-0414
  claim 8), and record screenshots and tick numbers throughout that delay: the original ends
  the game at the human's defeat, not after a period of watching surviving allies.
  Part F: the human player has no active spy satellite and is defeated (all its objects destroyed
  by map-placed computer units), with no unit of its dying in a gap field and no pending cells.
  Expected: the whole map is explored for the defeated player from the tick of defeat and stays
  so throughout the delay. Part F2: as part F, but the human's last unit died, less than 120
  ticks before the defeat, inside a hostile gap field from which it had just been walking, so that
  cells are pending at the moment of defeat. Expected: the whole map is explored at the tick of
  defeat, and at the next multiple of 120 exactly those pending cells become unexplored and stay
  so for the remainder of the delay (`quirks.defeat_reveal_keeps_pending`).
  Part F3: as part F, but the human has an active spy satellite before defeat, and no allied or
  reveal-to-all building provides sight afterwards. Use the short-game setting and make the
  satellite type not scored in named test data (S-0414 claims 2 and 4), with no counted
  base-unit vehicle or production item left, so loss of the last counted building defeats
  the human while the satellite is still alive. Defeat destroys it
  (S-0414 claim 5). Expected: at the next recheck after its destruction the whole map becomes
  unexplored, although the defeated house's radar stays on (S-0410 claims 1 and 2).
  Falsified if: in part F any cell is covered during the delay, or in part F2 the pending cells
  stay explored (the defeat reveal clears the marks, or the sweep skips a defeated house) or
  other cells become unexplored, or in part F3 the map stays explored after that recheck.
  Otherwise any differs.
- **V10 (claim 16).** A computer enemy unit whose weapon has the reveal-on-fire flag (a stock long-
  range type or a test type) fires from shroud at a human building. Part A: the recorder logs the
  tick of the shot and the tick on which the revealed area disappears. Expected: radius-3 shape
  around the firer explored on the tick of the shot, unexplored again on the next tick whose
  number is a multiple of 120. Part B (named test data): a test building type with a 3 × 3
  foundation and a reveal-on-fire weapon, owned by the computer player, fires at a human unit from
  shroud. Expected: the radius-3 shape is centred on the middle cell of the foundation (its centre
  point), not on its top-left cell. Part C (the skip): a human scout with small sight (field 2)
  first stands next to the computer firer so that the firer's cell is explored and the firer is
  discovered (its cell's cover piece changed), then walks off; the firer is then ordered by a map
  trigger to fire at a human unit. The firer's point is unshrouded, but cells on its far side
  within radius 3, which the scout never reached, are still unexplored. Expected: those cells
  stay unexplored after the shot. Part D: as part C, but the computer firer walks into the
  explored area only after the scout has left, then fires. Its cell placement already marks it
  discovered (claim 20), so absence of a scout does not make it undiscovered.
  Expected: the far cells stay unexplored, just as in part C. Part E: the shot's target belongs to a computer ally of the human
  (ally reveal on) and the firer is a computer enemy in shroud of both. Expected: the cells around
  the firer stay unexplored for the human player. Part F (named test copy of the rules with the lobby shroud setting
  off): the whole map is explored at the start; a computer enemy unit with a reveal-on-fire
  weapon is created on usable ground by a map trigger after the start, away from every human unit,
  and fires at a human building. Expected: its placement marks it discovered (claim 20), so no
  plain reveal is made and no shroud appears at the next multiple of 120. A repeat in which a
  human unit has been near the firer first gives the same result.
  Falsified if: another radius, a permanent
  reveal, a reveal in part C or D, a reveal in part E, a part-B shape centred on
  the top-left cell, or (part F) any cell becomes unexplored.
- **V11 (claim 18).** Named test copy of the rules: regrowth on, rate 0.1 game minutes (90 ticks).
  Part A: a human unit explores an area and leaves it, no gap generators. Part B: an enemy gap field
  re-covers part of the area and stops. Expected: nothing regrows in either part over 900 ticks.
  Part C: as part B, but a human unit leaves the enemy field at a tick chosen so that the next
  multiple of 90 comes before the next multiple of 120 (for example leaving at tick 145: the next
  multiples are 180 and 240). Expected: its cells become unexplored at the next multiple of 90 ticks,
  not at the multiple of 120; at each following multiple of 90 the explored cells outside the
  human player's current sights that touch (eight-neighbour) a cell taken at the step before become
  unexplored too, one ring per step and none earlier, until no explored cell outside the sights is
  connected to them. Falsified if: explored cells become unexplored at multiples of 90 ticks
  in part A or B (that would show the regrowth taking sight-explored cells, the alternative to
  claim 18), if in part C they go at the multiple of 120, if nothing but the pending cells ever
  goes (no ring), or if a ring appears at the same step as the cells it surrounds. Part D (claim 18's shroud-off
  consequence; low confidence): named test copy of the rules with the lobby shroud setting off,
  regrowth on and rate 0.1 game minutes, no gap generators and no pending cells. Expected: at
  tick 0 the border cells of the map's diamond that no sight covers become unexplored, and at each
  following multiple of 90 ticks the explored cells that touch (eight-neighbour) a cell taken at
  the step before, outside the sights, become unexplored too. Falsified if: the explored map stays
  whole for 900 ticks, or the border cells do not go at tick 0.
  Part E: repeat part D with rates 0, −1 and 0.001 game minutes in three named test rules
  copies, keeping the human's survival objects' sight fields at 0. Expected: rate 0 causes no
  regrowth over 900 ticks; −1 and 0.001 give periods of −900 and 0 ticks respectively, so
  the first border ring disappears at tick 0 and another ring at each following tick.
  Falsified if rate 0 erodes the map, either non-positive period waits between rings, or a
  ring disappears sooner than one tick after its predecessor.
  Part F (coincident regrowth and sweep, claims 13 and 18): use the 90-tick regrowth period,
  lobby shroud on, no satellite or whole-map operation. A human scout first explores a flat
  interior patch and stays with its centre and all eight neighbours under its sight. Create
  the scout after tick 200 to exclude start-of-play uncounted sight. A map trigger then
  creates a powered enemy gap generator centred on that cell, with gap radius 0
  in named test data, so only the centre is in the hostile field and the scout prevents its
  immediate covering. After tick 270, scripted input moves the scout away so that the centre
  and all eight neighbours lose its sight before tick 360; aim for departure near tick 330.
  Keep every sight away afterwards, with no further reveals or field changes and no other
  source of regrowth near the patch. Record the departure tick; void the run if the scout
  has not left all nine sampled cells after tick 270 and before tick 360, another hostile
  field reaches the patch, or a sight returns before tick 450.
  Observable: tick-labelled screenshots of the centre and its eight neighbours through tick
  450. Expected: the centre becomes unexplored at tick 360, when both periodic events are
  due; all eight neighbours remain explored then but become unexplored at tick 450, one
  regrowth step later. The sweep at tick 360 must not prevent that next ring. Falsified if
  the centre survives tick 360, any neighbour disappears before tick 450, or any neighbour
  stays explored at tick 450. In a fresh control run with regrowth off, repeat the departure:
  the centre still becomes unexplored at tick 360, but its neighbours stay explored through
  tick 450. One valid run per setting, any game speed. This distinguishes the collision's
  neighbour effect, not an intermediate image within tick 360 or a pending-cell setup at tick 0.
  Early loss of the centre after a valid departure, or a different control outcome, also
  falsifies the prediction rather than voiding the run.
- **V12 (claim 19).** Named test copy of the rules: lobby fog default on, fog rate 0.1 game
  minutes. A human unit explores an area and walks away. Expected: fog of war is off in the
  skirmish: cells left by sight stay drawn as explored ground (no fog pieces of S-0110) however
  long the run. A second run with a map whose fog flag is set: no fog either. Falsified if: any
  fog appears.
- **V13 (Edge cases, `quirks.uncounted_sight` and `quirks.recover_keeps_pending`; claim 15; parts B and B2: the surviving marks).** Part A: a
  human infantry unit stands at game start far from everything; after 200 ticks it walks away out
  of sight range; then a map trigger gives the computer player a powered gap generator whose field
  covers the whole area the unit saw at the start. Expected under the quirk (if the start-of-play
  uncounted sight reached the unit): the rim cells of its start shape stay explored while the inner
  cells become unexplored. Without the quirk (or if no unit existed when the local area was set):
  the whole field becomes unexplored. Either of those results decides the open question
  rather than by itself falsifying a settled start-of-play rule. The proposed rim effect is
  falsified if the surviving explored cells form neither the predicted rim nor the empty set,
  after excluding other sights and reveals. Part B: the human player has an active satellite. A human unit
  leaves a hostile gap field, making sampled cells pending, then walks until its sight covers
  none of them. Before the next multiple of 120, a map trigger creates a second powered enemy
  gap generator covering those cells; they must visibly become unexplored. A second human
  unit then brings exactly one sight over them and stops: because they were unexplored on
  arrival, S1 row 1 restores exploration without clearing their pending mark. Destroy the
  satellite before that sweep, with no other human sight over the sampled cells. Void and
  repeat if a sweep intervenes, either generator stops, or the cells were still explored
  immediately before the second unit's arrival. Expected: the cells become unexplored at
  the next multiple of 120 and stay unexplored while the second unit stands still.
  Part B2: as part B, with a third human unit that reaches and stops with the same cells in
  its sight before the re-cover (two re-applied sights). Expected: the cells stay explored at
  the next multiple of 120. Repeat without destroying the satellite to isolate cancellation
  by ordinary sight arrival, with neither sight refreshed before the sweep: expected, the
  cells remain explored. B2 is not an isolated test of the second re-applied sight clearing
  an existing mark: the third unit already clears that mark on arrival before re-cover.
  That particular mark-clearing event is independently unmeasured.
  Part C (the place of the sweep): no satellite or whole-map operation. A human unit leaves
  sampled cells inside one running hostile field, making them pending. A second human unit
  arrives over a sampled cell on the next tick whose number is a multiple of 120 (the
  recorder's tick log lets the harness repeat with different delays until one run coincides). Expected: the cell is explored
  after that tick and still explored after the next multiple of 120 while the unit stands
  still: the earlier sweep cleared the pending mark before the sight arrived. Part D (creation-order prediction, claim 15; low confidence): the human player owns an
  active spy satellite building and a unit standing still, with the unit's cells inside the field
  of a powered computer gap generator, the generator hostile. Run 1: the generator is created by a
  map trigger after the unit exists (so it comes later in creation order). Run 2: the unit is
  created by a map trigger after the generator exists (so the generator was created earlier). In
  each run the satellite building is then destroyed. Expected: in run 1 the cells under the unit
  become unexplored at the next multiple of 120 although the unit still stands there, and
  stay so; in run 2 they stay explored. Sample only cells under exactly one own sight and one
  hostile field, with no plain or area reveal there during the previous 120 ticks; run 1's
  generator is created while the unit's sight is already present, so its activation cannot
  immediately hide those cells. Exclude the satellite building's sight and all other sights. Falsified if:
  both runs agree or show the opposite outcomes: either falsifies the proposed creation-order
  relation, without identifying an internal processing sequence. Parts B, B2 and C are falsified if: part B's cells stay explored, or part B2's cells become
  unexplored (the predicted two-sight final state is wrong, but B2 alone cannot establish
  whether the mark was cleared before or during re-cover), or
  if B2's repeat without satellite loss becomes unexplored, or in part C
  the cell is unexplored right after the coincident tick (the sweep ran after the turn pass),
  or it becomes unexplored at the following sweep despite no intervening sight or gap change.
- **V14 (claim 17).** A map trigger (S-0011 subset) reveals around a waypoint inside an area that a
  computer enemy's powered gap generator covers, and another around a waypoint far from any field,
  with the human player's units elsewhere. Observable: the recorder's tick log and screenshots.
  Expected: the disc outside the field stays explored; the part of the disc inside the field is
  explored after the reveal and unexplored at the next multiple of 120. Falsified if: the cells
  inside the field stay explored, or the outside disc is covered later.

- **V15 (claim 2, claim 11 and alliance changes).** Use the common setup with a human house
  *H*, an initially mutually allied bystander *B* and another enemy bystander *E*; keep *E*
  alive so that making peace with *B* is not a last-enemy refusal (S-0414 claim 12). Named test
  rules enable both alliance-order flags of S-0414 claim 14 and keep ally reveal on. *B* owns
  a building whose sight covers the sampled cells and a powered gap generator whose field
  includes them. No sight of *H*, reveal-to-all building or other gap field reaches them.
  Record tick numbers and screenshots, and void a run if *B* independently changes alliance.
  Part A: scripted input breaks *H*'s alliance with *B* through the in-game player list, as
  S-0414 V9 does. Expected: the sampled cells are explored before the break and unexplored
  on the break tick, without waiting for a sweep: the building no longer reveals for *H*
  and its generator's field becomes hostile. Falsified if the cells stay explored or change
  only at the next multiple of 120. Part B: scripted input then makes *H* ally *B* one way,
  with *B* still not allied to *H*. Expected: the cells stay unexplored; both the sight and
  hostile-field tests use *B*'s view of *H*, not *H*'s view of *B*. Falsified if either the
  building's surroundings or the sampled gap cells become explored without a sight of *H*.
  One valid run per part, at any game speed. The effect on standing allied non-building units
  after an alliance change is not tested by this scenario.

- **V16 (claim 11, charged generation after switch-off).** Common setup, active radar,
  an own powered gap building with positive drain and two chargers in named test data
  (S-0405 W6's charging setup), all other operation conditions passing. Record tick numbers,
  the accepted off-order and the radar terrain darkening of S-0410 claim 6 over its field.
  Expected: after its next turn the field still darkens the radar although the switch is off.
  Repeat without chargers: darkening ends after the off-order tick and returns after an
  accepted on-order and the next turn that finds the building in operation. One valid run per
  case, any game speed; falsified if the charged building's darkening stays absent while off
  or the uncharged building continues generating while off. This tests the observable
  regenerated field, not the immediate stop within the order-to-turn interval.
- **V17 (claim 20; Q-0110).** Common setup, active radar, flat usable ground, own test vehicle
  with sight field 0 and ordinary radar-visible settings. A map trigger places it in unexplored
  ground after tick 200. Record its radar pixel from the first following turn; expected: shown
  without a scout or a cover-piece change. Repeat with an own map-placed vehicle at game start,
  a trigger vehicle in explored ground and a factory-produced vehicle in explored ground.
  Expected: each is discovered on placement and listed as soon as radar tracking updates.
  One valid run per case; falsified if a vehicle stays absent until a scout reveals it. Repeat
  the trigger case with a tree beside the vehicle in the same cell (named passable terrain test
  data if necessary): the tree must not suppress the vehicle's discovery or radar listing.
  This last run tests the observable consequence, not the redundant reveal-selection tie.
- **V18 (claim 21; Q-0111).** Common setup, a border visible within the usable area, no earlier
  border gap field, and the S-0110 test shroud sprite. The enemy owns one powered gap generator
  whose field includes non-legal cells. Keep it running while a trigger gives the human player
  a satellite, then destroy only the satellite. Record border pieces at sampled legal cells
  outside that field but within one stationary own building's sight, so legal-cell re-cover
  does not conceal the shared-state result. Expected: the outside-neighbour edges remain
  before activation, after activation and after satellite loss. In a fresh run, gain the
  satellite, stop the enemy field while the satellite works, then lose the satellite.
  Expected: outside-neighbour edges vanish when the field stops and stay absent after loss.
  One valid run per case, any game speed. Falsified if activation alone removes the edges,
  the running-field loss leaves them absent, or field-free satellite loss restores them.
  To test that the state is not permanently immune, in a third run restart the stopped
  field before satellite loss: expected outside-neighbour edges return on field restart.

- **V19 (Q-0649; claims 3, 15 and 18; S1 row 10).** Two fresh skirmish runs on a flat
  purpose-made map with a border visible inside the usable area. Use named test rules enabling
  regrowth at a period of 900 ticks and lengthening the end delay to 1800 ticks (S-0414 claim 8).
  Keep lobby shroud on and ally reveal off, and place no allied or reveal-to-all building.
  Director and mods are off apart from this named test data. In run A, give the human an active
  satellite after tick 200. Stop one powered enemy border gap field while the satellite works,
  with no other field touching non-legal cells, as in V8 H. Confirm that outside-neighbour edge
  pieces vanish. Destroy the satellite and wait for its loss recheck, with no running border
  field: claim 21 leaves the shared state explored. In run B, never create a satellite or border
  field, leaving the shared state unexplored. In each run defeat the human between ticks 500
  and 700, with no active satellite and no pending cells. Sample a long border segment and
  its first inward ring, excluding the segment's ends. Keep them and all their legal neighbours
  outside every sight and earlier reveal except run A's satellite reveal.
  In run B they must have remained unexplored until defeat, so none has an earlier surrounded
  mark. No later reveals, satellites or gap-field changes are permitted.
  Record tick-labelled screenshots at defeat and through tick 1800, using S-0110's test shroud
  sprite to read exploration rather than mistaking an edge piece for an unexplored cell.
  Expected: both runs explore every legal cell at defeat. At tick 900, run A keeps every legal
  cell explored: the defeat reveal marked even the border surrounded. In run B the sampled
  border cells become unexplored at tick 900 and the sampled inward ring at tick 1800.
  Any other sampled explored-cell
  set or timing falsifies the prediction, including border erosion in run A or no erosion in
  run B. One valid run per case, any game speed, 800×600; scripted input and a tick recorder
  suffice. This tests the gameplay consequence of the marks, not their unobservable values
  independently. V11 D separately tests the ordinary game-start case with shared state unexplored.

## Open questions

- Whether the pending marks of a defeated house really survive the defeat reveal and are swept
  (`quirks.defeat_reveal_keeps_pending`). Scenario V9, part F2, decides.
- Whether any unit is already on the map when the original sets up the local area at the start
  of play, which decides whether the start-of-play uncounted sight (`quirks.uncounted_sight`) has
  any effect in a skirmish. Scenario V13, part A, decides.
- The exact fog re-cover rule for fog on (not reachable in a skirmish): the original re-fogs a
  cell and, recursively, neighbours whose fog piece would become full; claim 19 states only the
  main effect.
- Whether a cell explored by a trigger reveal that does not mark it as surrounded can keep a pending
  mark while a sight covers it (S1 row 1 clears the mark only on cells already marked surrounded).
  This affects only trigger reveals outside S-0011's subset.
- Whether object creation order predicts newly pending cells after a whole-map re-cover
  (claim 15). V13 part D tests the single-sight, single-generator case only. The reveal case
  and general cases with several sights or generators remain unmeasured; no internal traversal
  order is a requirement of this spec.

## Questions for the project lead

None. The template requires switches for optional corrections, not for every fixed rule
listed under edge cases. The passive-country rule, sight boundaries and power-independent
satellite behaviour are fixed compatibility rules, not optional quirks; their classification
above changes no behaviour or quirk default.

Trigger reveals in the original are local-player-only. S-0011 claims 26 and 36 separately
state our engine's all-player-house departure, including observers; claim 17 here agrees,
with the whole-map-revealed exception tested per house for the waypoint reveal only.
S-0003 claim 14's trigger-name exceptions agree with S-0011 claims 5 and 21 and do not
restrict reveal recipients. At joint sign-off, confirm these exceptions and reveal scope
against the S-0003, S-0011 and S-0409 versions being handed off together. In particular,
S-0011 claims 26 and 36 and S-0409 claim 17 agree on all player
houses, observers included, and the waypoint-only per-house mark exception. Confirm these
paired revisions at hand-off. Animation start effects and random-draw composition remain
owned by the companion specs; they add no restriction on reveal recipients. S-0110's current
draft carries the non-legal-cell exception and its claim-21 persistence qualification.
Confirm that agreement too. No human-only trigger reveal in our engine or ownership-only
discovery rule is intended.

For the S-0414 joint hand-off, claim 15's defeat reveal is the immediate exploration effect,
not a promise of permanent whole-map exploration. Pending re-cover marks may hide cells at
the next sweep, and later satellite loss may re-cover the map (V9 F2 and F3). S-0414 claim 5's
summary must retain these exceptions when describing the defeated person's continuing view.
Its play-to-last-side departure changes how long the person can watch, not these visibility
rules. This confirmation concerns visibility only; the other owners confirm the remaining
S-0414 hand-off dependencies.
