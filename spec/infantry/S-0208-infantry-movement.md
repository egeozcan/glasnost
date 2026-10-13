---
id: S-0208
title: Infantry movement (standing positions, walking step and speed, crawling, scatter destinations)
topic: infantry
revision: 14
sources-class: [decomp]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-13
measurements: []
requests: [Q-0017, Q-0072]
supersedes: []
---

# S-0208: Infantry movement (standing positions, walking step and speed, crawling, scatter destinations)

## Summary

This spec covers where inside a cell an infantry unit stands and walks to, how far and how fast it
walks per tick (crawling included), when a walk ends and the unit loses its destination (also for
an infantry unit that approaches its target: the infantry part of Q-0072, question 3), when it
accepts a scatter request and where it then goes, and which scatter requests damage sends it.

### Scope and cited specs

Which sequence an infantry unit shows while it walks, and when fear makes it lie down, get up or
panic, belong to S-0107 (claims 7, 10, 15 and 16). How many infantry a cell holds, what a blocked
walker does and every entry outcome of a cell belong to S-0209, which owns occupancy outcomes (a
decision under the delegated sign-off, 2026-10-10): where this spec names such an outcome, it
points to S-0209 and S-0209 decides. This spec owns the infantry scatter from damage (claims 11 and 15).
For an infantry unit that approaches its target, S-0215 claim 37 owns when the approach step runs
and whether it keeps, renews or takes a destination, S-0307 owns the approach position and the
outcomes of a failed search, and S-0310 claim 8 owns the firing check's *moving* answer. This spec
owns only when such a walker stops and loses its destination (claims 16 and 17). Infantry of a
jumpjet type move by the jumpjet locomotor, which belongs to other movement specs; claims 16 and
17 do not cover them.

Every spec cited here is approved, and the claim, formula, table and scenario numbers cited are
those of the approved revisions in the spec index: S-0003 (INI value forms), S-0006 (rules
layering), S-0009 (map file semantics), S-0011 (map triggers), S-0105 (voxel rendering, for its table trigonometry and
table square root), S-0106 (slope shapes and terrain surface height), S-0107 (infantry sprite
sequences, revision 4, which drops its damage-scatter passages and points here), S-0201 (game
speed and tick rate), S-0202 (the random streams), S-0204 (coordinates, distance and facing),
S-0205 (number domains), S-0207 (ground vehicle movement), S-0209 (cell occupancy and crowding),
S-0210 (crushing), S-0211 (land types and passability), S-0212 (pathfinding), S-0215 (behaviours)
and S-0311 (veterancy), and for claims 16 and 17 S-0307 (weapon range test), S-0308 (weapon
choice) and S-0310 (turrets and facing for fire, for the firing check's answers and the infantry
speed fraction).

## Terms

Glossary terms are used as defined there, in particular *cell*, *lepton*, *tick*, *facing*,
*infantry*, *scatter*, *scenario random stream*, *human-controlled*, *computer-controlled*, *AI house*, *team member*, *bridge, high bridge, bridge
deck*, *locomotor*, *field*, *local area*, *round-toward-zero mode* and *measurement setup*. Terms
used by this spec:

- **Human-controlled, computer-controlled owner.** As the glossary defines them, and as the original
  tells them apart. This spec uses "computer-controlled owner" for every owner that is not
  human-controlled, which is wider than the glossary entry *computer-controlled*: the original's
  computer player does not play the neutral house or the special house (glossary *house*), but the
  original gives their units the same owner rules as those of its computer player's houses, since
  it tests only whether the owner is human-controlled (claims 11 and 15). Where a rule below names a computer-controlled owner, it states what
  the original does. Our engine departs from it for AI houses only: an AI house counts as
  human-controlled (glossary *AI house*, the project lead's decision), so for units of an AI house
  our engine follows the rule for human-controlled owners. That decision covers player houses only.
  The neutral and special houses, which are neither played by a person nor AI houses, keep the
  original's rules for owners that are not human-controlled (the project lead's decision of
  2026-10-10), so our engine treats them as computer-controlled owners, as the original does. The
  departure for AI houses is stated in claim 11 and recalled in claim 15, the only claims of this
  spec with an owner condition.
- **Position within a cell.** A point (*u*, *v*) with *u* = *x* mod 256 and *v* = *y* mod 256, where
  (*x*, *y*) is the unit's world position in leptons. (128, 128) is the cell centre.
- **Standing position.** One of five fixed positions within a cell where infantry stand (Constants,
  table P): the *centre* position and the *top*, *right*, *left* and *bottom* positions, named after
  where they appear on the screen. The top position is the one with the smallest *x* and *y*.
- **Position class** of a point: the standing position that the original associates with that
  point (claim 2). It is not always the nearest standing position.
- **Side positions.** The right, left and bottom positions. They are the only positions that the
  original assigns to an arriving walker when it takes the cell's other occupants into account
  (claim 3).
- **Open gate.** A gate (S-0209 Terms: a building type that units can pass through while it is
  open) that is open at that moment.
- **Occupancy record.** The per-cell record of which standing positions are taken, owned by S-0209.
  A cell has one record for the ground and one for its bridge deck.
- **Walking target.** The point a walking infantry unit is currently heading for: a standing
  position in the cell it is entering (claims 3 and 4).
- **Step vector.** For a step direction (glossary *step direction*: 0 map north, 2 map east, 4 map
  south, 6 map west), the one-cell step in direction *d* moves a point by 256 leptons along each map
  axis that the direction uses, for example (+256, −256) for direction 1.
- **Speed field.** The type's imported speed value (Import mapping), an integer.
- **Speed multiplier.** A per-unit real number, 1.0 when the unit is created. A speed crate
  multiplies it (the crate rules belong to the crate specs).
- **Country speed factor for infantry.** A real number per country, imported (Import mapping), that
  scales the walking speed of every infantry unit of a house of that country.
- **Faster veterancy ability.** The veterancy ability *faster* of S-0311 (Terms, claims 3 and 4),
  which raises speed. As S-0311 claim 3 gives for every ability, a unit of veteran rank has it when
  its type's veteran ability list names it, and a unit of elite rank has it when its type's veteran
  or elite ability list names it. Its factor is a game-wide imported value, the *veteran speed
  factor*. The *scatter veterancy ability* (claim 11) is S-0311's *scatter*, gained the same way.
- **Prone state, can crawl, cyborg, fraidycat, fear level, deploy kinds, interruptible.** As defined
  in S-0107 Terms.
- **Crippled cyborg.** A unit of a cyborg type that took lethal damage but survived as a crawling
  unit with at least 1 health. When that happens and what it looks like belong to the combat specs
  and S-0107. Here it only matters that it enters the prone state then, whether or not its type can
  crawl.
- **Mission target.** As defined in S-0209 Terms: the object or cell that a unit's current
  behaviour aims to enter, capture, sabotage or otherwise reach, while the unit is in a behaviour
  that has one. Claim 4 uses only a mission target that is an object; a mission target that is a
  cell plays no part there. The master of an enslaved unit is not a mission target (glossary
  *mission target*); claim 4 names it beside it.
- **Enslaved unit, master, master's unloading cell.** As defined in S-0209 Terms.
- **Team member.** As in the glossary and S-0215 Terms: a unit that belongs to a team of
  the original's computer-player script system. A unit of an AI house is never one (glossary *AI
  house*). Other houses could get teams only through that system, for example by the map trigger
  actions that S-0011 claim 30 hands to it. The system is out of scope for glasnost (glossary *team
  member*), so while it stays out of scope no unit of our engine is a team member, and the team
  condition of claim 11 never applies in our engine.
- **The cow.** The hard-coded infantry type that S-0107 claim 14 singles out for its idle scatter
  (`quirks.cow_idle`); S-0107 Import mapping names how the type is recognised.
- **Known attacker.** As defined in S-0107 Terms: damage that comes with the object that caused it.
- **Behaviour.** A unit's current behaviour in the sense of S-0215 (behaviours), for example *move*
  or *guard*.
- **Retaliate property, scatter property.** Two of the behaviour properties of S-0215 Terms: the
  property of a behaviour without which an actor in it never retaliates (S-0215 claim 18), and the
  property of a behaviour that lets an actor in it scatter (S-0215 claim 19). "Its behaviour has the
  scatter property" refers to the unit's current behaviour.
- **May retaliate.** Whether a damaged actor may retaliate against its attacker, by the whole of
  the original's test: the conditions of S-0215 claim 18 other than its distance rule (the
  retaliate property of the current behaviour, and the behaviour list and existing-target
  conditions for a human-controlled owner), together with the retaliation conditions that the
  target-acquisition specs own (glossary *retaliation*), among them that the actor has a weapon it
  can use against the attacker. An actor whose current behaviour lacks the retaliate property may
  not. An actor that may retaliate does retaliate against the hit when S-0215 claim 18 lets it
  override: in the original, an actor of a computer-controlled owner always does, at once, and an
  actor of a human-controlled owner does when its weapon already reaches the attacker or the
  attacker lies within the distance of S-0215 Formulas F7.
- **Target, destination.** As defined in S-0215 Terms: the unit's current target object and its
  current destination (a cell or object it moves to). Either may be absent.
- **Locomotor moving.** "Its locomotor is moving" means that the unit *has movement in progress* in
  the sense of S-0107 Terms. It does not mean S-0107's narrower *is moving now*.
- **Locomotor destination.** The point the unit's locomotor is currently taking it to as the end of
  its walk: while a unit walks under a move, the point that its destination gives (for a cell
  destination, a point in that cell). A unit with no movement in progress has none. It differs from the
  *destination* above, which is the cell or object the unit's orders and behaviour give it. Claim
  16 tests the locomotor destination, not the destination. It is not the walking target (Terms below), which is only the next standing
  position on the way.
- **Scatter reference point.** The point that the scatter centre cell is taken from: the unit's
  walking target (Terms below) while it has one, and otherwise its own position. For a unit
  travelling through a tunnel (S-0212 Terms: the map-defined passage from a tunnel entrance to a
  tunnel exit, not the land type *tunnel* of S-0211), the tunnel's exit cell centre counts as its
  scatter reference point here (Open questions). Claim 15 tests the destination; the scatter
  centre cell uses the scatter reference point.
- **Tick number.** As defined in S-0201 (Terms and claim 1): the number of ticks the game has
  completed, read during the tick in question.
- **Player-scatter rule.** A game-wide imported yes/no rule (Import mapping) that lets units of
  human-controlled houses scatter (claims 11 and 15).
- **Usable area.** As in the glossary entry *usable area*, owned by S-0009: a cell is inside it
  when the test of S-0009 claim 4 puts it inside the local area. Claim 12 uses the test's
  *height-aware* form (S-0009 claim 4), as the nearby-cell search does (S-0009 claim 5a).
- **High-bridge cell.** A cell that carries a high bridge's bridge deck (glossary *bridge, high
  bridge, bridge deck*). Low-bridge cells are not high-bridge cells.
- **Scatter request.** A request to an infantry unit to scatter, with an optional *threat point* (the
  point to move away from) and two yes/no options, *forced* and *move anywhere*, whose effects
  claim 11 gives (the glossary's *scatter request* defines them in general terms). Claim 15 here says which
  requests damage sends, S-0107 claim 15 the request of the flee rule, S-0107 claim 14 the
  requests of the idle scatters (of a frightened fraidycat whose owner is not human-controlled, and
  of the cow), S-0215 claim 28 the scatter off a building (sent to an actor in *capture*,
  *sabotage* or *eaten* that falls back to rest while its position is on a cell that holds a
  building), and S-0209 and S-0210 the requests of crowding and crushing, each with its options. The flee rule, the idle scatters and the scatter
  off a building send forced requests without a threat point that are not move-anywhere.
- **Scatter centre cell.** The cell around which a scattering unit looks for a destination (claims 12
  and 13): the cell of its scatter reference point. So it is the cell of the standing position a
  walker is heading for at that moment (the next cell of its path, or its own cell when it walks
  to another position in the cell it is in), and the unit's own cell when it has no walking target:
  when it stands still, and also on a walker's arrival tick and asking tick (claim 6). It is never
  the cell of a farther destination.
- **Screen-cover check.** A test of a candidate cell *C* at its own height level ℓ (claim 12). *C*ᵢ is the
  cell *i* cells from *C* towards the bottom of the screen along the cell diagonal (*C* moved by +*i*
  cells along both map axes), and its *effective level* *e*(*C*ᵢ), in height levels, is its height
  level, plus 4 when *C*ᵢ is a high-bridge cell. The original adds the 4 only when *C* also has a
  property that was not identified (Open questions); our engine adds it whenever *C*ᵢ is a
  high-bridge cell, as if every *C* had the property. That is
  our engine's provisional choice, kept until the open question is settled, and it has no quirk
  switch because the original's condition is not known. *C* fails the check exactly when *e*(*C*ᵢ) ≥ ℓ + 2*i* − 1 for at least one *i* from
  1 to 6, and passes otherwise. Height levels and *i* are integers, and the test is exact integer
  arithmetic. *C* itself never makes *C* fail. A *C*ᵢ that lies outside the
  map's cells counts as height level 0 and not a high-bridge cell, so on a map whose levels are all 0
  or more it never makes *C* fail. Consequences: on flat ground with no high-bridge cell every cell
  passes, and a cell whose diagonal neighbour towards the bottom of the screen is one level higher fails. The threshold
  2*i* − 1 is what a cell lifted on the screen by 128 leptons along each map axis per height level
  needs to reach *C*, so the check roughly asks whether a raised cell in front hides *C* on the
  screen. The *visibility test* of the nearby-cell search (S-0212 claim 25) is this check, applied
  to a candidate cell at its own height level. This entry owns the check's arithmetic; S-0212
  claim 25 cites it for its visibility test. Reading
  a cell outside the map as level 0 and not a high-bridge cell agrees with how S-0009 claim 44 has
  our engine read every cell outside the map.
- **Approximate square root √̃, table sine sinT, table cosine cosT.** The original's table square
  root (S-0204 claim 5 and Formulas F1, the same function as S-0105's table square root) and table
  trigonometry (S-0105 Formulas F1). See Formulas F3 and F8. S-0209 and S-0210 use √̃ too.
- **Enterable.** A neighbouring cell is enterable for a unit when S-0209's occupancy check gives the
  outcome *clear* for that unit.
- **Tether link.** A link that two objects share while one of them performs a docking, loading or
  unloading manoeuvre with the other. Both hold it from the moment one of them begins loading or
  unloading, gives the other clearance to unload, or docks at a refinery, and both lose it when
  that manoeuvre ends or the link is broken off. A unit holding one is not sent the second case of
  request A (claim 15). Which situations give an infantry unit one was not determined (Open
  questions).

## Behaviour

### Standing positions

1. **Five standing positions.** An infantry unit standing still in a cell stands at one of the five
   standing positions of table P. The points are engine constants. A walking target that claims 3
   and 4 give lies at the terrain surface height *H* of S-0106 (claim 2) at that point, or, on a
   bridge deck, 416 leptons (4 height levels, glossary *bridge deck*) above *H*. A walker that
   arrives (claim 6) is placed on its walking target, height included. A pre-placed unit (claim 5)
   is expected to stand at the same height as a walker that has arrived at the same position.
   - Backing: `measurable` (scenario W1 for the points on flat ground; scenario W2, part e, for the
     height on a slope and on a bridge deck, and for pre-placed units on a slope)
   - Confidence: high for the points and for the height of a walking target; medium that an
     arriving walker takes its target's height, and for the height of a pre-placed unit

2. **Position class of a point.** For a point with position within the cell (*u*, *v*), let *r* be
   its distance from the cell centre (Formulas F1). If *r* < 60, that is, when
   (*u* − 128)² + (*v* − 128)² < 3600, the class is the centre.
   Otherwise the class is chosen by quadrant: right when *u* > 128 and *v* ≤ 128, left when
   *u* ≤ 128 and *v* > 128, bottom when *u* > 128 and *v* > 128, and the **centre** (not the top)
   when *u* ≤ 128 and *v* ≤ 128. No point has the class *top*. A unit standing at the top position
   therefore has the class *centre*, and it is recorded in the occupancy record as taking the centre
   (`quirks.top_position_counts_as_centre`).
   - Backing: `measurable` (scenario W2, part c for the quadrant rule and the class of the top
     position, part b for its consequence in the occupancy record). `unmeasured` for the centre
     radius of 60 leptons: every standing position lies either on the centre (*r* = 0) or at
     *r* = 90 (table P), and no setup is known that makes the original take the class of any other
     point, so no measurement can tell 60 from any other radius from 1 to 90
   - Confidence: high for the class rule; medium for its consequence on the nearby-cell search
     (W2 b)

3. **Choosing a standing position when walking into a cell.** When a walker steps from its cell
   into the neighbouring cell in step direction *d*, its *desired point* is its current world
   position moved by the one-cell step in direction *d*. So a walker keeps the same position within
   the cell, as far as it is free. In the normal case (every case but that of claim 4) the cell's
   other occupants are taken into account, and the walker gets the position that the table below
   gives. In it, *c* is the class of its desired point (claim 2), and "free" and "taken" refer to the
   cell's occupancy record (S-0209 Terms and claim 1) for the surface the walker is on (ground or
   bridge deck). A side position that a terrain object in the cell marks (S-0209 claim 1) counts as
   taken like one an infantry unit holds, so a walker never gets it. The *vehicle-present* and
   *object-present* flags are those of S-0209 Terms. What a walker that gets no position does belongs
   to S-0209.

   | Occupancy record of the cell being entered | Class *c* | Position the walker gets | Draws from the scenario random stream |
   |---|---|---|---|
   | the record for the walker's surface has its vehicle-present flag on, or both the cell's ground record has its object-present flag on and no open gate (Terms) stands in the cell | any | none | none |
   | neither | right, left or bottom, and *c* is free | *c* | none |
   | neither | right, left or bottom, and *c* is taken | the first free position of the row of table O for *c* (S2 for right, S3 for left, S4 for bottom); none when no position of that row is free | none |
   | neither | centre | the first free position of row R*k* of table O, where *k* is one ranged draw over 0 to 3; none when no position of that row is free | one |

   The building-present flag plays no part here: buildings keep walkers out only through the entry
   outcomes of S-0209 (claims 3 and 4), so a walker that S-0209 lets into a building's cell (for
   example a cell of a building type that is invisible in play) gets a position there by this
   table. For a walker on a bridge deck too, it is the ground record's object-present flag that
   counts.

   No point has the class *top* (claim 2), so the table covers every case. The centre and the top
   position appear in no row of table O, so the table never gives them, even when free. Because
   rows R0 and R1 both lead with the right position, a walker whose desired point has class centre
   and that enters an empty cell stands at the right position with probability 1/2, and at the left
   and bottom positions with probability 1/4 each.

   The chosen position is reserved at once: from the tick on which the walker chooses it, the
   occupancy record of the cell being entered shows that position as taken, and the walker's
   standing position in the cell it is leaving is shown as free (S-0209 claim 1). A walker therefore
   holds exactly one entry in the occupancy record at any time: its walking target while it has one,
   otherwise the position it stands on. One exception is the top position (claim 2, Edge
   cases): a unit at the top position and a unit at the centre of the same cell share the centre's
   entry, and when either of them leaves, the record shows the centre free although the other still
   stands there.
   - Backing: `measurable` (scenarios W1, W2 part c, W3 and W9; the reservation through S-0209 scenario
     C1; the vehicle-present row through S-0209 scenario C2, the building-present flag through S-0209
     scenario C10, the terrain marks through S-0209 scenario C16). `unmeasured` for the
     object-present flag and its open-gate exception, for the reason S-0209 claim 1 gives: every
     stock object known to set the flag gets an entry outcome of its own, so its effect here, and
     so the exception to it, has no known stock trigger
   - Confidence: high for the class rule, table O, the draw and the reservation; medium for the
     object-present flag and the open-gate exception

4. **Walking into a cell without regard to its occupants.** The occupants of the cell being
   entered are ignored in two cases:
   - the unit's mission target (Terms) is an object, and it is a vehicle or aircraft whose position
     lies in that cell, or a building that occupies that cell (for example a vehicle or aircraft
     the unit is ordered to enter, or a building it is ordered to capture or sabotage). A mission
     target that is a cell, or an infantry unit, does not make this case.
   - the unit is an enslaved unit, the cell is its master's unloading cell (S-0209 Terms), and its
     master is there.

   The walker then heads for the standing position of its desired point's class, which is the
   centre position when that class is the centre, whatever the occupancy record shows. No random
   draw happens.
   - Backing: `measurable` (scenario W1: part c for a vehicle to be entered, part d for the centre
     class and no draw, part e for a building, part f for an aircraft; S-0209 scenario C13 for
     the enslaved unit, which walks into its master's unloading cell). `unmeasured` for the
     exclusion of a mission target that is a cell or an infantry unit: which behaviours have a
     mission target, and of which kind, is still open (S-0209, Open questions), so no setup is
     known that gives a walker such a mission target in the cell it walks into
   - Confidence: medium

5. **Pre-placed infantry.** An infantry unit placed by the map stands at the standing position whose
   number (table P) is the map's sub-cell value. A map value of 1 places it at the top position,
   which then counts as the centre in the occupancy record (claim 2). A map value outside 0 to 4 gives
   an undefined position in the original. Our engine places such a unit at the centre position and
   the importer warns (a departure).
   - Backing: `measurable` (scenario W2, part d for the position of each value 0 to 4; part a for the
     occupancy consequence of value 1). `unmeasured` for values outside 0 to 4: the original's
     position there is undefined, and our engine's rule is a departure that does not reproduce
     it, so no measured value would change what our engine does
   - Confidence: high for values 0 to 4

### The walking step

6. **One step per tick.** A walker with a walking target does one of two things on each tick,
   depending on its distance to the target at that moment (Formulas F2):
   - Closer than 17 leptons: it is placed exactly on the walking target and arrives there on that
     tick. From then on it has no walking target, and its occupancy entry is the position it stands
     on (claim 3). It does not choose its next walking target on that tick.
   - 17 leptons or more: it faces the walking target and moves *s* leptons towards it, where *s* is
     its current walking speed (claim 7). Its new world position is its old position plus *s* times
     the unit vector of its facing, each coordinate truncated to an integer (Formulas F3, with the
     table sine and cosine of S-0105 and the facing of F7).

   A walker that stands on a standing position with a path step ahead, having arrived there on an
   earlier tick, chooses its next walking target (claim 3) on a tick on which it does not move; this
   is the *asking tick* of S-0209. It takes its first step towards that target on the next tick.
   On an uninterrupted walk the asking tick is the tick right after the arrival tick, and on the
   first cell of a walk it is the tick before the first step.

   The step changes only the walker's map position. Between standing positions our engine keeps the
   height of the position the walker left. On arrival the walker takes the height of its walking
   target, as claim 1 states. Keeping the height of the position left, between standing positions,
   is our engine's provisional choice, kept until the open question is settled, and not a statement
   about the original: whether the original does the same was not determined (Open questions). It
   has no quirk switch. The height
   during the walk can matter only to a rule that reads a walking unit's height, and scenario
   W2(e) records it.

   Consequence: for a walking speed *s* from 2 to 38 leptons per tick, on a straight run due map
   north or map east from standing position to standing position (256 leptons per cell), a walker
   makes ⌈240 / *s*⌉ steps in each cell until it is within 17 leptons, spends one tick arriving and,
   before every cell but the first, one tick choosing its walking target. So each cell after the
   first takes ⌈240 / *s*⌉ + 2 ticks, and a run of *n* cells takes *n* × (⌈240 / *s*⌉ + 2) − 1
   ticks counted from the first step to the arrival (both ticks included). Due map west the same
   holds from *s* = 4 on. Due map south and due map west drift sideways and can take longer
   (claim 14). Speeds outside 2 to 38 behave differently (claim 14 for *s* = 1, Edge cases for fast
   walkers).
   - Backing: `measurable` (scenarios W4 and W8)
   - Confidence: high for the 17-lepton arrival rule and the truncation, and that a walker does not
     move on the tick on which it chooses a walking target; medium for the choice falling on the tick
     after the arrival tick, which assumes that a walker's movement is processed once per tick
     (scenario W4 decides between ⌈240 / *s*⌉ + 2 and ⌈240 / *s*⌉ + 1 ticks per cell); medium for
     the exact direction arithmetic (F3, F7)

7. **Walking speed.** On every step, a walker's walking speed *s* in leptons per tick is the value
   that Formulas F5 gives from the unit's values at that step: its type's base speed *b* (Formulas
   F4), its speed multiplier, its house's country speed factor for infantry, and the veteran speed
   factor if it has the faster veterancy ability. So a change to any of these shows from the next
   step. Game speed does not change it (glossary claim G3).
   - Backing: `measurable` (scenario W4: part a for the base speed and the country factor, part b for
     the veteran speed factor, part c for the speed multiplier)
   - Confidence: high for the structure and the truncation points; the base speed is S-0207's
     (claim 1 and F1), with the confidence given there

8. **Crawling is slower, and a prone non-crawler is faster.** While a walker is in the prone state,
   its speed from claim 7 is changed: when its type can crawl, it moves *s* − ⌊*s*/3⌋ leptons per
   tick; when it cannot crawl, *s* + ⌊*s*/2⌋. Only a crippled cyborg can be prone without being
   able to crawl (S-0107 claim 6 turns down the lie-down request of a type that cannot crawl), so in
   practice only such a unit gets the faster rate (`quirks.prone_noncrawler_speedup`). Which units
   are prone, and when a prone unit gets up instead of crawling, belongs to S-0107 (claim 15).
   - Backing: `measurable` (scenario W5: part a for crawling, part d for the prone non-crawler)
   - Confidence: high for both formulas; medium for the crippled-cyborg trigger

9. **Terrain does not slow walkers.** A land type's speed percentage for walkers has only one effect
   on infantry: a percentage of exactly zero makes the cell impassable (S-0209). Any other value,
   such as 50 %, leaves the walking speed unchanged.
   - Backing: `measurable` (scenario W6)
   - Confidence: medium

10. **Panic does not change speed.** A walker whose fear shows the *panic* sequence (S-0107 claim 7)
    walks at the speed of claim 7. Fear changes the speed only through the prone state of claim 8.
    - Backing: `measurable` (scenario W5, part c)
    - Confidence: medium

### Scatter destinations

11. **When an infantry unit accepts a scatter request.** A request is *effectively forced* when it
    is forced and the unit's locomotor is not moving at that moment. A unit has *scatter
    permission* for a request when at least one of these holds: its owner is computer-controlled, it
    is a team member, the game-wide player-scatter rule is on, it has the scatter veterancy ability,
    or the request is move-anywhere. The unit refuses the request, and does not move, when at least
    one row of this table holds in full. Otherwise it accepts it.

    | Case | The unit refuses when all of these hold |
    |---|---|
    | First (sequence) | it plays a sequence; that sequence is not *undeploy*; that sequence is not interruptible (S-0107 table T1) |
    | Second (deploying) | its current sequence is one of the deploy kinds (it is *deploying*, S-0107 Terms); its owner is human-controlled; the request is not both forced and move-anywhere |
    | Third (not forced) | the request is not effectively forced; and not all three of these hold: the unit is a fraidycat, its current behaviour has the scatter property (S-0215 claim 19), it has scatter permission |

    When a request that is both forced and move-anywhere reaches a unit whose current sequence is
    one of the deploy kinds, the unit is also sent a request to play *undeploy*, one that is not
    forced, on that tick, whatever its owner, whether or not its locomotor is moving, and whether or
    not it accepts the scatter request. Whether *undeploy* starts is decided by S-0107 claim 6. For
    such a unit the first case refuses the scatter request exactly when *undeploy* does not start
    and the sequence the unit keeps playing is neither *undeploy* nor interruptible (S-0107 table
    T1). Since the *undeploy* request is not forced, S-0107 claim 6 and table T1 give these
    outcomes:

    | Sequence when the request arrives | Outcome of the *undeploy* request (S-0107 claim 6) | First case |
    |---|---|---|
    | *deploy* | turned down, always: *deploy* is not interruptible and the request is not forced (row 5), or the type's *undeploy* entry has no image frames (row 1) | refuses, since the unit keeps playing *deploy* |
    | *deployed*, *deployed-fire* or *deployed-idle*, and the type's *undeploy* entry has image frames | *undeploy* starts (row 6; these three kinds are interruptible) | does not refuse, since the unit now plays *undeploy* |
    | *deployed*, *deployed-fire* or *deployed-idle*, and the type's *undeploy* entry has no image frames | turned down (row 1): the unit keeps playing its sequence | does not refuse, since that sequence is interruptible |

    No stock behaviour starts *deployed-idle* (S-0107 claim 20), so its rows matter only where a
    mechanic of another spec starts it.

    No other unit gets a request for *undeploy* or any exemption: for a unit whose current sequence is
    not a deploy kind the first case applies to a forced, move-anywhere request as to any other, so
    a unit playing a sequence that is not interruptible (for example *cheer*) refuses it.

    **In our engine** (a departure for AI houses). The owner conditions of this claim and of claim 15
    describe the original. Our engine does not reproduce them for AI houses: by the project lead's
    decision, recorded in the glossary entry *AI house*, an AI house follows the rules for
    human-controlled owners, so no unit of an AI house has a computer-controlled owner, and none is
    a team member (Terms *team member*). This departure covers every owner condition of this spec,
    not only those of claim 15, and it has no quirk switch because the decision removes
    computer-only unit rules. It changes claim 11 for every unit of an AI house in two places. The
    second case (deploy kinds) applies to it. Its scatter permission reads "the game-wide
    player-scatter rule is on, it has the scatter veterancy ability, or the request is
    move-anywhere". So an AI house's fraidycat in a behaviour with the scatter property refuses a
    request that is not effectively forced (for example request B of claim 15) unless one of those
    three holds, where the original's computer-owned fraidycat accepts it. The original's rule is
    recorded for AI designers, who may reproduce it through ordinary orders. Units of the neutral and
    special houses are not covered by that decision: by the project lead's decision of 2026-10-10,
    our engine gives them the original's rule for a computer-controlled owner (Terms).

    The cases combine yes/no answers; their order carries no meaning. So a request that is not
    effectively forced moves only fraidycats. The threat-point request that damage with a known
    attacker sends (claim 15, request B) is of that kind, so only fraidycats act on it. The general
    damage request (claim 15, request A), the request of the flee rule (S-0107 claim 15), the
    requests of the idle scatters (S-0107 claim 14) and the scatter off a building (S-0215 claim
    28) are forced and have no threat point, so they can move any infantry unit whose locomotor is
    not moving.
    - Backing: `measurable` (scenario W7: parts a, c, e, g and m for acceptance and refusal; part a
      for a forced request refused while the locomotor is moving; parts b and r for the third case
      accepting a fraidycat with a computer-controlled owner, part r with a forced request while
      its locomotor is moving; part q for scatter permission through the player-scatter rule and
      through the scatter veterancy ability, and for its refusal without them; part j, variants j2
      and j3, for the neutral and special houses giving scatter permission as computer-controlled
      owners; part m for
      the first case with a forced, move-anywhere request to a unit that is not deploying; parts n2
      and n4 for the second case, which they isolate (in part n1 the first case refuses as well,
      since *deploy* is not interruptible), and part n5 for its owner condition; parts e and o for
      the *deploy*, *deployed* and *deployed-fire* rows of the *undeploy* table, part o4 for a type
      whose *undeploy* entry has no image frames. Scenario W9, parts b and d, for the draw taken by
      an accepted request, part d when it finds no destination). `unmeasured` for three parts:
      - the *deployed-idle* rows: no stock behaviour starts *deployed-idle* (S-0107 claim 20), so
        no setup reaches it without a mechanic that another spec owns.
      - the team-member condition of scatter permission: a unit becomes a team member only through
        the original's computer-player team system (Terms), which no spec states and which then
        gives the unit orders of its own, so no setup could tell this condition's effect apart from
        the team's orders. The system is out of scope, and the condition never applies in our engine
        (Terms *team member*).
      - the move-anywhere condition of scatter permission: it decides only for a request that is not
        effectively forced, that is, a move-anywhere request that is not forced, or a forced one to a
        unit whose locomotor is moving. The move-anywhere requests known (S-0209 claim 12)
        are all forced, so the condition needs one of them to reach a fraidycat whose locomotor is
        moving. A walker's request (S-0209 claim 8) goes to a cell whose three side positions are
        taken by allied infantry with no movement in progress (S-0209 claim 4, row WR4), and the
        request of a ground vehicle blocked by friendly units (S-0209 claim 12, a pointer to the
        vehicle-movement specs) to a cell holding a unit that stands still. A walking fraidycat would
        have to be listed in such a cell, entering or leaving it, at the moment the request is sent,
        and no setup is known that arranges that reliably
    - Confidence: medium (the cases were derived, not yet observed; the scatter property of each behaviour comes from
      the rules data that S-0215 imports)

12. **Scatter away from a threat point.** When an accepted request has a threat point, the unit's
    first-choice direction is *d* = (*d*₀ + *j* − 2) mod 8, the residue from 0 to 7, in exact integer arithmetic. Here *d*₀ is
    the facing from the threat point to the unit's own position, rounded to the nearest of the eight
    step directions (Formulas F6), and *j* is one ranged draw from the scenario random stream over 0 to 4. Its candidates are
    the eight neighbours of its scatter centre cell, in the step directions *d*, *d* + 1, …, *d* + 7
    (mod 8), in that order of preference. A candidate *qualifies* when it lies inside the usable area
    by the height-aware test of S-0009 claim 4 (Terms) and is enterable for the unit. The unit's destination is the first qualifying candidate that also
    passes the screen-cover check at its own height level and is not a high-bridge cell. When no qualifying candidate is both,
    its destination is the first qualifying candidate, whatever the check gives and whether or not it
    is a high-bridge cell. The unit then moves there as under a move order. When no candidate
    qualifies, it does not move.
    - Backing: `measurable` (scenario W7, parts b and d; part f for the screen-cover check on uneven
      ground; part l for the exclusion of high-bridge cells from the first choice; part p for the
      height-aware form of the usable area; scenario W9, part d, for the draw of a request that
      finds no destination). `unmeasured` for three parts of the screen-cover
      check and the scatter centre cell:
      - the 4 levels by which a high-bridge cell counts higher: in the original this depends on a
        property of *C* that was not identified (Terms, Open questions), so no setup can be chosen
        that is known to have it or to lack it, and our engine's rule is a provisional choice.
      - the level of a cell outside the map: it can change the outcome only for a candidate at a
        negative height level whose diagonal towards the bottom of the screen leaves the map
        within 6 cells. In the original every cell outside the map reads the same values, which
        other rules can change during play, and not every rule that changes them is known (S-0009
        claim 44), so a measured value could not tell this rule apart from such an
        earlier change. Our engine reads the blank cell of that claim.
      - the tunnel exit as scatter centre cell (Terms *scatter reference point*): whether a unit in
        a tunnel can be sent a scatter request at all belongs to S-0212 and the movement specs,
        which do not say (Open questions)
    - Confidence: high for the direction rounding, the draw and the preference order; medium for the
      scatter centre cell, for the exclusion of high-bridge cells, for the height-aware usable
      area and for the screen-cover rule on uneven ground (that every cell passes on flat ground
      is high); low for the high-bridge part of the check, for the level of a cell outside the map
      and for the tunnel exit (Open questions)

13. **Scatter without a threat point.** An accepted request without a threat point also takes
    the single draw *j* of claim 12, and has a direction *d* = (*d*₀ + *j* − 2) mod 8 as in claim 12.
    Here *d*₀ is the facing from the centre of the unit's cell to its own position (or its body
    facing when it stands exactly on the centre), rounded by F6. Its destination is the cell that
    the nearby-cell search of S-0212 (claim 25) gives around its scatter centre cell, in the scatter
    use that S-0212 claim 25 describes (no reference cell and no reach region). When the search
    gives a cell, the unit moves there as under a move order. Which cells qualify (among
    them the rule that a cell's whole occupancy record must be empty), the double count of the
    centre, the movement zone and the choice among the preferred cells by the tick number of the
    request are S-0212's. This spec adds only how a scatter uses the result. The direction plays no
    part in that search, so the destination does not depend on the draw.

    Consequences, from S-0212 claim 25 and S-0209 claim 1: the search does not give the cell the
    unit stands in while the unit's own standing position is marked there, but it can give the
    scatter centre cell itself when that is a walker's next cell and its record is empty. The
    unit's own position is marked except in the case of Edge cases (the top position recorded as the
    centre): a unit left at the top or the centre position after its cell-mate at the other of these
    two positions has gone is not marked, and what the search then gives for its own cell is
    S-0212's (claim 25). On open flat
    ground with no other units within 2 cells, an idle unit's search collects exactly the eight
    neighbours of its cell, all preferred, so the destination is the neighbour at position
    (*N* mod 8) of the search's collection order, where *N* is the tick number (Terms) of the tick
    of the request.

    When the search finds no cell, the unit instead chooses among the eight neighbours of its
    scatter centre cell exactly as in claim 12, starting from the direction *d* computed here. This
    happens when no cell qualifies for the search while a neighbour is enterable, for
    example when every cell that the search could reach holds an occupant and a neighbour holds
    allied infantry with a side position free (scenario W7, part k).
    - Backing: `measurable` (scenario W7, parts c and i for the use of the search and its
      independence from the draw; part r for the scatter centre cell of a walking unit; part k for
      the fallback to the eight neighbours; scenario W2, part b, for a cell whose record shows no
      position taken; scenario W9, part d, for the draw of a request that finds no destination).
      That the search can give the scatter centre cell itself follows from S-0212 claim 25 and
      S-0209 claim 1, which carry its backing
    - Confidence: medium for the draw, the use of the search and the fallback; the search itself is
      S-0212's, with the confidence given there

### Rounding along the map axes

14. **Walking due map south or due map west drifts sideways.** Under the original's round-toward-zero
    mode (S-0205), the table trigonometry of S-0105 (F1) holds two tiny non-zero values where the
    exact value is zero: table entry 4096 holds about +1.2 × 10⁻¹⁶ and entry 8192 about
    −2.4 × 10⁻¹⁶. A walker heading exactly due map south reads entry 8192 as its cosine: it moves
    1 lepton towards map west on each step (its *x* is reduced by 1). A walker heading exactly due map
    west reads entry 4096 as its sine: it moves 1 lepton towards map north on each step (its *y* is
    reduced by 1). Each drift lasts until the accumulated deviation turns the walker's facing away
    from the exact axis. The walker then heads back towards its walking target at a slight angle and,
    for due map south, covers 1 lepton less per step than its speed while it does. Due map east is
    exact (its sine is exactly 0). Due map north reads entry 4096 as its cosine, but adding that tiny
    positive amount to a positive coordinate and truncating gives the same integer, so it is exact
    too.

    For example, the formulas give, for straight runs of 10 cells from standing position to
    standing position: at *s* = 9, 289 ticks north, east or west and 319 ticks south; at *s* = 4,
    619 and 809 ticks. Evaluated for every speed *s* from 2 to 38 leptons per tick: north and east
    always take 10 × (⌈240 / *s*⌉ + 2) − 1 ticks; west takes the same from *s* = 4 on and one tick
    more per cell at *s* = 2 and 3; south never takes fewer ticks than north, takes more at low
    speeds, and from *s* = 17 on often takes the same (at *s* = 21, for example, all four directions
    take 139 ticks), so that only the sideways drift shows. The ticks on which the walker chooses its
    walking target do not move it, so they add the same 9 ticks to every direction of a 10-cell run. The sideways deviation stays at or below 3
    leptons going south and 6 leptons going west. With round-to-nearest arithmetic there is no drift
    and all four axis directions take the same time. This follows from F3 and F7, and it is stated
    because it is the most visible consequence.

    **A walker of speed 1 can stall.** At *s* = 1, a step whose direction is not exactly along a map
    axis changes each coordinate by less than one lepton. The truncation of F3 then leaves a
    coordinate that should grow unchanged and lowers by 1 a coordinate that should shrink. So a
    walker of speed 1 whose walking target lies towards larger *x* and larger *y* (between due map
    east and due map south, exclusive) stops: its steps leave its position unchanged, and no rule
    of this spec moves it on. A walker of speed 1 heading due map south drifts into such a
    direction within its first few steps and stalls there, a few leptons from its starting point
    (`quirks.slow_walker_stall`, Edge cases).
    - Backing: `measurable` (scenario W8, part a for the drift and the tick counts, part b for the
      stall)
    - Confidence: medium (derived by exact evaluation of the formulas, not yet observed)

### Scatter requests from damage

15. **Scatter requests caused by damage.** When an infantry unit takes damage and survives it, it
    can be sent two scatter requests on the tick of the hit. Each is judged by claim 11 on its own,
    and each that is accepted takes its own draw (claims 12 and 13).
    - **Request A, the general damage request** (the scatter from damage of S-0215 claim 19): forced,
      not move-anywhere, without a threat point. It is sent when either row of this table holds in
      full:

      | Case | The damage and retaliation | All of these also hold |
      |---|---|---|
      | First | the damage has a known attacker (S-0107 Terms); the unit may retaliate against it (Terms, S-0215 claim 18); it does not retaliate against this hit | it had neither a target nor a destination (Terms) when hit; the player-scatter rule is on or the unit has the scatter veterancy ability, whatever its owner |
      | Second | the damage has no known attacker, or the unit may not retaliate against it | its current behaviour has the scatter property (S-0215 claim 19); it is not held by a tether link (Terms); its locomotor is not moving; it has neither a target nor a destination; its owner is computer-controlled, or the player-scatter rule is on, or the unit has the scatter veterancy ability |

      Retaliating always gives the unit a target (S-0215 claims 10 and 18), so a unit that
      retaliates is never sent request A by the first case.
    - **Request B, the threat-point request:** sent whenever the damage has a known attacker, with the
      attacker's position as threat point, neither forced nor move-anywhere. By claim 11 only
      fraidycats act on it.

    **Original and our engine.** The second case of request A above, as written, is the original's:
    its owner condition holds for every unit of a computer-controlled house, so such a unit is sent
    request A of the second case whatever the player-scatter rule says. In our engine an AI house is
    not computer-controlled (the departure of claim 11), so for a unit of an AI house the second case
    needs the player-scatter rule on or the scatter veterancy ability, as for a unit of a
    human-controlled house. Units of the neutral and special houses keep the original's owner
    condition (Terms, the project lead's decision of 2026-10-10). The first case is the same in our engine. S-0215 claim 19 states the same
    rule for all mobile actors and points here for infantry.

    Request A is forced, so by claim 11 it moves an infantry unit that is not a fraidycat when its
    locomotor is not moving, the first case of claim 11 (its current sequence) does not refuse it
    and, when its owner is human-controlled (in our engine, also when it is an AI house), its current
    sequence is not one of the deploy kinds. When a hit sends both requests and both are accepted,
    two draws are taken on the tick of the hit. The first draw is request A's and the second is
    request B's. When request B finds a destination, the unit heads for it at the end of that tick:
    claim 12 gives it from the second draw, around the unit's own cell. Whenever
    request B finds a destination, it replaces request A's, so request A's draw shows nothing at the
    end of the tick; on open ground it does not even decide request A's own destination, which
    claim 13 takes from the search (scenarios W7, part j, and W9, part c). Request B's scatter
    centre cell is the unit's own cell: request A gives the unit a destination but no walking
    target, which the unit chooses only at its next walking step (claim 6), so its scatter reference
    point (Terms) is still its own position. Consequences with the player-scatter rule off (its
    absent-key default, Import mapping) and without the scatter veterancy ability:
    - An infantry unit of a human-controlled house (in our engine, also of an AI house) that is not
      a fraidycat never scatters because of damage (it may still move to retaliate, S-0215 claim
      18).
    - A computer-controlled one (in the original; in our engine, one of the neutral or special
      house) scatters only by request A, through the nearby-cell search of claim 13, so its scatter
      destination does not depend on where the attacker is.
    - In particular, a computer-controlled infantry unit that is not a fraidycat and may retaliate
      (Terms) against the known attacker that hit it retaliates at once (S-0215 claim 18) and does
      not scatter: the first case of request A needs that it does not retaliate against this hit,
      the second that it may not retaliate, and claim 11 refuses request B.
    - An idle computer-controlled infantry unit that is not a fraidycat, whose behaviour has the
      scatter property and whose sequence is interruptible, scatters when it is hurt by damage
      without a known attacker, provided claim 13 finds it a cell.
    - Backing: `measurable` (scenario W7: parts a, g, h and i for the two cases of request A and for
      request B, with part g, variant g3, and part h, variants h4 and h5, for the player-scatter
      rule and the scatter veterancy ability, and part h, variants h6 and h7, for units of the
      neutral and special houses; scenario W9, part e, for the conditions that the locomotor is not
      moving and that the unit has no destination, part c for the order of the two draws; W7 part j
      for the outcome when both requests are accepted). `unmeasured` for the tether-link condition
      of the second case: no situation is known in which an infantry unit holds a tether link
      (Terms, Open questions), so no setup can show its effect
    - Confidence: medium for the two cases of request A, for request B and for the scatter centre
      cell of request B when both requests are accepted; low for the situations that set the tether
      link (Terms, Open questions)

### End of a walk and the approaching walker

16. **When a walk ends.** On each arrival tick (claim 6), after the walker has been placed on its
    walking target, the walk ends when either of these holds:
    - the unit has no locomotor destination (Terms), for example because claim 17 or another rule
      cleared its destination on this tick or earlier, or
    - the walker's new position is in the same cell as its locomotor destination, and the two heights
      differ by less than two height levels (|Δ*z*| < 208 leptons, strict).

    The test is made only for a unit that is alive, on the map and not falling. When the walk ends,
    the unit loses its destination, its speed fraction (S-0310 Terms *infantry speed fraction*)
    becomes 0 and it stands at the standing position it arrived on, with no movement in progress
    from then on (Terms *locomotor moving*). Otherwise the walk continues by claim 6. This claim
    never ends a walk between two arrival ticks; a rule that stops the unit there (claim 17's
    exception) does so itself.
    - Backing: `measurable` (scenario W10, parts a and d)
    - Confidence: high that the walk ends on the arrival tick in the destination cell; medium for the
      height condition, which matters only where the locomotor destination and the arrived position
      differ in height in one cell (a bridge deck over its ground cell, Edge cases)

17. **An approaching walker stops when its target comes into range.** On each arrival tick, before
    claim 16's test, an infantry unit loses its destination when all of these hold:
    - its target (Terms) is a mobile actor: an infantry unit, vehicle or aircraft. A building, a
      cell, an overlay or a terrain object never triggers this stop.
    - its current behaviour (S-0215), or, when it has none, its queued behaviour, is *attack*, *area
      guard*, *hunt* or *rescue* (*rescue* is the team behaviour, out of scope here; it is listed
      only so that the rule is complete).
    - the target is in range. For a type that is not open-topped, this is the weapon range test of
      S-0307 for the weapon that S-0308 selects against the target, with the source point of S-0307
      claim 8, run as if the target were the cell that holds it: the measured point is that cell's
      centre (S-0307 claim 7 for a cell), not the target's own reference point. For an open-topped
      type it is instead a three-dimensional test: the approximate distance (F8) from the target's
      reference point to the actor's, truncated to an integer, strictly less than the actor's
      weapon range ρ of S-0307 claim 13.

    The unit loses only its destination: its target is kept. Claim 16 then finds no locomotor
    destination, so the walk ends on this same arrival tick, at the standing position just reached.
    Consequences:
    - A walker stops only at standing positions, never in the middle of a step. A target that comes
      into range while the walker is between two standing positions is noticed on the next arrival
      tick. Until then the walker walks on, and its firing check answers *moving* (S-0310 claim 8),
      so it does not fire while walking.
    - A walker approaching a building, a cell or another non-mobile target gets no early stop: it
      walks on to the approach cell that S-0307 gave it (claim 15 there), keeping its destination
      (S-0215 claim 37), and stops there by claim 16.
    - Under any other behaviour (*move*, *attack move*, *guard*, *capture*, *enter* and the others)
      there is no early stop either: the walker continues to its destination.
    - Because the measured point is the target cell's centre, a target standing near
      its cell's edge can be found out of range by this test while S-0215 claim 37's own range test,
      which measures to the target's own reference point, finds it in range, and the reverse. The two disagree only
      within half a cell of the range boundary (Edge cases).

    **Exception: the target is the destination.** When the unit's destination is its target object
    itself (the object destinations of S-0307 claim 17: a hunter-seeker, or a vehicle thief), and
    its firing check (S-0310) answers *fire*, the unit loses its destination and any movement still
    in progress ends on the tick on which it starts its firing sequence. By S-0310 claim 8 the check
    answers *moving* while the speed fraction is above 0.1, so this happens only to a walker that is
    already standing (for example blocked, S-0209), never in the middle of a step.
    - Backing: `measurable` (scenario W10, parts a, b, b2 and b3 for the rule; part c checks
      only the exception's end state)
    - Confidence: medium for the rule and its four behaviours; medium for the measured point being
      the target cell's centre; low for the exception, which was found by reading and not traced
      through a whole approach (Open questions)

## Formulas

**F1. Distance from the centre.** *r* = trunc(√̃((*u* − 128)² + (*v* − 128)²)), where √̃ is the
approximate square root of F8. *u* and *v* are integers from 0 to 255, so the argument is an integer
from 0 to 32768. By F8, *r* < 60 exactly when (*u* − 128)² + (*v* − 128)² < 3600, as with the exact
root. Unit: leptons.

**F2. Arrival test.** The walker arrives when *D* < 17, with
*D* = trunc(√̃((*x*ₜ − *x*)² + (*y*ₜ − *y*)²)) from its world position (*x*, *y*) and the walking target
(*x*ₜ, *y*ₜ), all four integer world positions in leptons (table P and F3's truncation give integers),
so the argument of √̃ is a non-negative integer. Height is ignored. By F8, the test is equivalent to (*x*ₜ − *x*)² + (*y*ₜ − *y*)² < 289.
Unit: leptons.

**F3. Step.** With facing *f* towards the walking target (F7), an integer from 0 to 65535, and speed
*s* (F5),

> *f̂* = *f* when *f* ≤ 32767, and *f̂* = *f* − 65536 otherwise (so −32768 ≤ *f̂* ≤ 32767),
> θ = (*f̂* − 16383) × *c*₂ radians, with *c*₂ = −2π / 65534 (the double −9.587672516830327 × 10⁻⁵),
> *x*′ = trunc(*x* + *s* × cosT θ),  *y*′ = trunc(*y* − *s* × sinT θ),

where sinT and cosT are the table sine and cosine of S-0105 (F1, table trigonometry). *f̂* is exact
integer arithmetic. Floating point in the original: every intermediate result of the formulas (θ,
each product *s* × cosT θ and *s* × sinT θ, and each sum or difference) is a double rounded by the
round-toward-zero mode of S-0205, with the grouping written, and trunc is the conversion to an
integer toward zero. θ is negative exactly for *f* from 16384 to 32767, where S-0105's
negative-angle rule applies to sinT and cosT (Edge cases). With `quirks.table_direction` off, θ =
(*f̂* − 16384) × (−2π / 65536) instead (Edge cases); everything else in F3 is unchanged.
With round-toward-zero, adding a tiny negative product to a positive coordinate gives the next lower
representable value, which trunc then takes to the integer below (claim 14). Units: leptons, radians.

**F4. Base speed from the speed field.** *b* is the type speed *V* of S-0207 (claim 1 and Formulas
F1), computed from the type's speed field by the same rule for infantry as for vehicles, with its
clamping, its "not given" value and its importer warnings (Import mapping). This spec adds nothing
to that formula. For an infantry type the result counts in leptons per tick, from 0 to 255.
Unit: leptons per tick.

**F5. Walking speed.**

> *s*₁ = trunc((*b* × *k*꜀) × *m*)
> *s* = trunc(*s*₁ × *k*ᵥ) when the unit has the faster veterancy ability, and *s* = *s*₁ otherwise

*k*꜀ is the country speed factor for infantry (stored as a single-precision real, widened to a
double), *m* the unit's speed multiplier (a double), *k*ᵥ the veteran speed factor (a double). The
products are taken in floating point in the original, grouped as the parentheses show:
(*b* × *k*꜀) × *m*, not *b* × (*k*꜀ × *m*). Each product is rounded to a double toward zero (the
round-toward-zero mode of S-0205), and each trunc is the conversion to an integer toward zero. In the
prone state, claim 8 then gives *s* − ⌊*s*/3⌋ (can crawl) or *s* + ⌊*s*/2⌋ (cannot crawl), in exact
integer arithmetic (*s* ≥ 0, so ⌊ ⌋ and truncation agree). Unit: leptons per tick.

**F6. Rounding a facing to a step direction.** *d*₀ = ⌊(*f* + 4096) / 8192⌋ mod 8 for a facing
*f* from 0 to 65535. Exact integer arithmetic. This rounds to the nearest eighth of a turn, with a
facing exactly halfway between two directions going to the clockwise one. It is the 8-step facing of
S-0204 (F4). Units: *f* in facing units, *d*₀ a step direction (0 to 7, Terms).

**F7. Facing from one point towards another.** *f* = *φ* of S-0204 Formulas F3 (claim 8), with *P*
the "from" point and *Q* the "to" point. This spec adds nothing to that formula: its arithmetic,
grouping and rounding are S-0204's. Consequences stated there: a point straight towards map north
gives *f* = 0, map east 16383, map south 32767 and map west 49153, and a point one cell map
north-east 8315. "From" is the unit and "to" its walking target for claim 6; "from" is the threat point and "to" the unit for claim 12; for claim 13
"from" is the cell centre (128, 128) and "to" the unit's position within the cell. Unit: facing
units.

**F8. Approximate square root √̃.** √̃ is the table square root of S-0204 (claim 5, Formulas F1), the
same function as S-0105's table square root for an integer argument. The properties this spec relies
on, for integer arguments *V* ≥ 0: trunc(√̃(*V*)) never exceeds ⌊√*V*⌋, it equals ⌊√*V*⌋ for every
*V* < 16641 (= 129²), and it never decreases as *V* grows. So trunc(√̃(*V*)) < *t* exactly when
*V* < *t*² for every threshold *t* up to 128, which gives the equivalences in F1 and F2. From
16641 on it can fall short of ⌊√*V*⌋ (for example trunc(√̃(129²)) = 128). Every distance in S-0208,
S-0209 and S-0210 is taken with √̃.

## Constants

**Table P: standing positions** (engine constants; positions within the cell in leptons).

| Number | Name | *u* | *v* | Screen |
|---|---|---|---|---|
| 0 | centre | 128 | 128 | middle of the cell |
| 1 | top | 64 | 64 | towards the cell's top corner |
| 2 | right | 192 | 64 | towards the right corner |
| 3 | left | 64 | 192 | towards the left corner |
| 4 | bottom | 192 | 192 | towards the bottom corner |

**Table O: fallback orders** (engine constants; positions tried left to right. The centre and top
positions are never taken, as claim 3 says).

| Row | Used when | Order |
|---|---|---|
| S2 | desired class right, right taken | bottom, left |
| S3 | desired class left, left taken | bottom, right |
| S4 | desired class bottom, bottom taken | right, left |
| R0 | desired class centre, draw 0 | right, left, bottom |
| R1 | desired class centre, draw 1 | right, left, bottom |
| R2 | desired class centre, draw 2 | left, bottom, right |
| R3 | desired class centre, draw 3 | bottom, right, left |

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Centre radius | 60 | leptons | claim 2: *r* < 60 has class centre |
| Arrival distance | 17 | leptons | claim 6: *D* < 17 arrives |
| Draw range, position order | 0 to 3 | draw value | claim 3, one draw |
| Draw range, scatter jitter | 0 to 4 | draw value | claims 12 and 13, one draw |
| Speed multiplier at creation | 1.0 | factor | Terms |
| Crawl speed | *s* − ⌊*s*/3⌋ | leptons per tick | claim 8 |
| Prone non-crawler speed | *s* + ⌊*s*/2⌋ | leptons per tick | claim 8 |
| Step angle scale *c*₂ | −2π / 65534 | radians per facing unit | F3; a double |
| Screen-cover reach | 6 | cells along the diagonal | Terms, *screen-cover check* |
| Screen-cover threshold for the cell *i* cells away | 2*i* − 1 | height levels above *C* | Terms, *screen-cover check* |
| Screen-cover high-bridge raise | 4 | height levels | Terms, *screen-cover check* (applied whenever *C*ᵢ is a high-bridge cell) |
| Level of a cell outside the map, for the screen-cover check | 0 | height level | Terms, *screen-cover check* |
| Height tolerance for the end of a walk | 2 height levels (208 leptons, strict) | leptons | claim 16 |

## State and ordering

| State | Event | Result |
|---|---|---|
| Walking towards a target in the next cell | distance < 17 leptons | placed on the target, arrives this tick; no walking target until its next asking tick |
| Walking towards a target | distance ≥ 17 leptons | moves *s* leptons towards it this tick |
| Standing, about to step into the next cell (asking tick, at the earliest the tick after arrival) | the cell is clear for it and a position is free | that position becomes its walking target (claim 3); no movement this tick, the first step on the next |
| Standing, about to step | no position is free | no walking target; S-0209 decides |
| Any | scatter request refused (claim 11) | no draw and no new destination; a unit whose current sequence is a deploy kind, sent a forced, move-anywhere request, has still been sent a request for *undeploy* (claim 11) |
| Any | scatter request accepted, destination found | moves to that cell as under a move order |
| Any | scatter request accepted, no destination found | does not move; the draw has been taken |
| Walking, arrival tick | target is a mobile actor in range at its cell centre, behaviour attack, area guard, hunt or rescue (claim 17) | loses its destination, keeps its target; the walk ends on this tick (claim 16) |
| Walking, arrival tick | no locomotor destination, or arrived in the locomotor destination's cell within two height levels (claim 16) | walk ends: no destination, speed fraction 0, stands at the position reached |
| Walking, arrival tick | neither of the above | continues; asks for its next walking target on the next tick (claim 6) |
| Walking, destination is its target object | firing check answers *fire* (claim 17, exception) | loses its destination and stops as its firing sequence starts |

Ordering stated as consequences (the draw counts and order are checked with the read-only recorder
of the random stream, scenario W9): the random draw of claim 3 happens on the tick on which the walker
chooses its position in the next cell (its asking tick, claim 6): the tick after the one on which it
arrives in the previous cell, or, on the first cell of a walk, the tick before its first step. The draw of claim 12 or 13 happens on the tick on which the
scatter request arrives, and only when the request is accepted: a request refused by any case of
claim 11 takes no draw, while an accepted request takes its one draw even when no destination is
then found. When one hit sends both damage requests of claim 15, both arrive on the tick of the hit.
When both are accepted, two draws are taken, the first by request A and the second by request B.
When request B finds a destination, the unit heads for it at the end of the tick: claim 12 gives it
from the second draw (around the unit's own cell, claim 15), and it
replaces request A's destination (scenarios W7, part j, and W9, part c).
On an arrival tick, claim 17's in-range stop is decided before claim 16's end-of-walk test, both
after the walker has been placed on its walking target, so a walker that comes into range at a
standing position stops on that same tick and does not ask for a next walking target (scenario
W10).

## Edge cases and quirks

- **The top position is recorded as the centre** (`quirks.top_position_counts_as_centre`, default:
  the original). A unit standing at the top position is recorded as taking the centre (claim 2), so
  a unit at the top and another at the centre of the same cell share one entry of the occupancy
  record. When one of them leaves, the record shows the centre free although the other still stands
  there. Rules that read the record rather than the units then miss the remaining unit:
  - The nearby-cell search of S-0212 (claim 25) asks for a cell whose whole record is empty, so it
    can give that cell, for example as the destination of a scatter without a threat point (claim
    13). Scenario W2(b) tests this; medium confidence (claim 2).
  - For a vehicle, S-0209 claim 5 and its consequences for infantry in the cell decide, since S-0209
    owns occupancy outcomes (Scope). While the remaining unit does not count as moving, it keeps an
    allied vehicle out through its own row (row VP20), whatever the record shows, so the switch
    changes nothing for a vehicle then, and no vehicle drives into the cell of a remaining unit that
    stands still. While it counts as moving, its own row gives a value only
    in the cases S-0209 names (the close case of S-0212 claim 27, and a unit at a chain point, row
    VP22); otherwise the record decides, and a record that shows no position taken gives *clear*
    (row VR4) where a record with the centre taken would give *moving blocker* (row VR6, when the
    cell's infantry house is allied with the vehicle's house). These outcomes are S-0209's claims,
    with the backing given there.

  Arriving walkers never take the top or centre positions (claim 3). Only pre-placed units (claim
  5) and units entering the cell of their mission target or master (claim 4, which then uses the
  centre) stand there. With the switch off, a unit standing at the top position is recorded as
  taking the top position, so each of the two units has its own entry, and the record is not empty
  while either of them stands in the cell. Nothing else changes: the position class of every point
  is still given by claim 2 (no point has the class *top*), so claims 3 and 4 choose positions and
  take draws exactly as with the switch on, and arriving walkers still use the side positions
  only. The capacity rule of S-0209 claim 2 does not count either the top or the centre position,
  so it is the same with the switch on or off.
- **A prone non-crawler is faster** (`quirks.prone_noncrawler_speedup`, default: the original). With
  the switch off, a prone unit whose type cannot crawl walks at the speed of claim 7.
- **Fast walkers can step past the walking target** (`quirks.walker_overshoot`, default: the
  original). A walker arrives only when it is closer than 17 leptons to its walking target at the
  moment it takes its step (claim 6). From *s* = 34 leptons per tick on, one step can carry it from
  17 leptons or more before the target to 17 leptons or more beyond it. It then turns and walks
  back, and it can swing back and forth around the target for a while or for good. By the
  formulas, a walker of speed 46 walking one cell from the right standing position due map north
  never arrives; due map east it arrives after 15 ticks instead of 7, due map west after 28, and
  due map south after 7. On the straight runs evaluated, speeds 34 to 38 still arrive on time and
  39 is the first speed that does not. Medium confidence, derived by exact evaluation of the
  formulas (scenario W8, part c). With the switch off, a step that would end 17 leptons or more
  beyond the walking target ends exactly on the target instead, and the walker arrives on the next
  tick as under claim 6. Nothing changes for speeds up to 33, which can never end that far beyond.
- **A walker of speed 1 can stall** (`quirks.slow_walker_stall`, default: the original). By claim 14,
  a walker of speed 1 whose walking target lies strictly between due map east and due map south of
  it takes steps that leave its position unchanged, so no rule of this spec moves it on. Medium
  confidence, derived by exact evaluation of F3 (scenario W8, part b). With the switch off, a step
  that leaves both coordinates unchanged while the walker is 17 leptons or more from its walking
  target moves it instead by 1 lepton along each map axis on which the target lies off its
  position, towards the target.
- **Facing scale of the walking direction** (`quirks.table_direction`, shared with S-0204, default:
  the original). The original turns a facing back into an angle (F3) with the same short scale that
  S-0204 F3 uses to make a facing from an angle: a full turn is 65534 facing units, not 65536, and
  map east is facing 16383. So the angle F3 gives for a facing differs from that facing's exact
  angle by up to 1.5 facing units, which can now and then change a truncated step by one lepton.
  The two scales belong together: F7 gives the axis facings 0, 16383, 32767 and 49153, and F3 turns
  each of them back into an angle within an eighth of a table step of its axis, which the table
  trigonometry then reads as exactly the axis entry. With the switch off, S-0204 makes F7 the
  exact direction (axis facings 0, 16384, 32768 and 49152), and F3 uses
  θ = (*f̂* − 16384) × (−2π / 65536), with exact integer arithmetic for *f̂* − 16384 and the same
  floating-point rules as F3 for the product. Then facings 0, 16384, 32768 and 49152 give the axis
  angles π/2, 0, 3π/2 and π, up to the rounding of 2π to a double. One switch covers both formulas
  so that the direction and the step always use the same scale: a mixed setting would turn exact
  facings into angles one facing unit off. S-0204 lists S-0208 F3 among the formulas its switch
  changes (its Edge cases). Claim 14 does not depend on this switch: due map south still reads table entry
  8192 as its cosine and due map west entry 4096 as its sine, with the switch on or off.
- **Table trigonometry and the approximate root.** The table square root, the table trigonometry
  and the table direction are the original's arithmetic (S-0105, S-0204), not approximations chosen
  by this spec. Their switches, as they bear on this spec:
  - `quirks.table_trig_negative_bias` (S-0105, default: the original). The table sine and cosine of
    a negative angle can land two table steps short (S-0105 F1). In F3, θ is negative exactly for
    facings 16384 to 32767, so the switch can change some steps of walkers heading between map east
    and map south. It does not change claim 14: exact due map south is 16384 × 8192 / 65534 =
    2048.0625 table steps below zero (4096.125 half steps), and its fraction of a step is below one
    half, where the bias does not apply.
  - `quirks.table_root_distance` (S-0204, default: the original). It changes nothing in F1 or F2:
    both thresholds (60 and 17 leptons) are below 129, where trunc(√̃(*V*)) and ⌊√*V*⌋ agree (F8).
  - `quirks.table_direction` (S-0204, default: the original). It covers F7 and, by the entry above,
    the scale of F3.
  - S-0105 defines no switch for table trigonometry as a whole: sinT and cosT are always the
    tables.

  The quantised directions of the table arctangent (S-0204 claim 8, parts b and c) bend walks that
  are not along an axis slightly.
- **Sideways drift on the map axes** (claim 14). It needs both the two tiny non-zero entries 4096
  and 8192 of the table trigonometry of S-0105 and round-toward-zero arithmetic, so it is governed by
  the round-toward-zero switch `quirks.chop_rounding` of S-0205 and has no switch of its own. With
  that switch off (round to nearest), the drift disappears.
- **Rounding mode.** Every floating-point operation and truncation above follows the
  round-toward-zero mode of S-0205, switch `quirks.chop_rounding`.
- **Scatter around the next cell.** A walking unit that accepts a scatter request looks for its
  new destination around the cell of its walking target (the scatter centre cell), which is
  usually the next cell of its path, not around the cell it is in and not around its farther
  destination (claims 12 and 13), while the direction away from a threat point is still taken from
  the unit's own position. On its arrival tick and its asking tick it has no walking target, so it
  looks around its own cell. It is listed here because it is easy to miss; it is ordinary behaviour
  of the original, not a defect, and has no switch.
- **The in-range stop tests the target's cell centre** (claim 17). The approaching walker's stop
  measures range to the centre of the target's cell at that cell's terrain height, not to the
  target itself, while S-0215 claim 37 and the firing check measure to the target's own reference
  point (S-0307 claim 7). Near the range
  boundary the walker can therefore walk one standing position further than needed, or stop one
  standing position short and then be moved again by S-0215 claim 37. It is the original's
  behaviour, not a defect we fix, and has no switch; a variant measuring to the target's own
  position would differ only within half a cell of the boundary (scenario W10, part b2).
- **No early stop against non-mobile targets** (claim 17). A walker approaching a building or a
  cell always walks to its approach cell, even when it came into range earlier. It has no switch.

## Import mapping

| User data | Feeds | Form | Absent |
|---|---|---|---|
| Rules, infantry type section, `Speed` | speed field (F4) | as S-0207 claim 1 and its import mapping give for the same key: integer, clamped; −1 is the key's "not given" value (S-0006 claim 3) and keeps the value in force without a warning; the importer warns for a value above 100 or below −1 | as S-0207 gives: keeps the value in force (the engine default before any rules layer) |
| Rules, the country's own section, `SpeedInfantryMult` | country speed factor for infantry *k*꜀ | decimal (S-0003 claim 6), stored in single precision | 1.0 (engine constant) |
| Rules, `[General]`, `VeteranSpeed` | veteran speed factor *k*ᵥ | decimal, held as a double (S-0311 Import mapping) | keeps the value in force; 1.0 (engine constant) before any rules layer gives it (S-0311) |
| Rules, infantry type section, `VeteranAbilities` / `EliteAbilities` naming `FASTER` or `SCATTER` | faster and scatter veterancy abilities (Terms) | list, parsed as S-0311 Import mapping gives (letter case ignored) | empty list (S-0311) |
| Art, image section, `Crawls` | can crawl | boolean | yes (S-0107) |
| Rules, infantry type section, `Cyborg`, `Fraidycat` | cyborg, fraidycat | boolean | no (S-0107) |
| Rules, the combat-damage section, `PlayerScatter` | the player-scatter rule (claims 11 and 15) | boolean | no |
| Rules, land-type sections, `Foot` | the walker speed percentage; only zero matters (claim 9) | percent | owned by S-0211 |
| Map, `[Infantry]` entries, the sub-cell value | standing position number (claim 5) | integer, 0 to 4 | values outside 0 to 4: centre, importer warning |

The scatter property of each behaviour (claim 11) is imported by S-0215.

## Measurement scenarios

All run under the *measurement setup*, on purpose-made flat maps of one theater with clear terrain
unless stated, with one human player and one computer player whose start position is far away,
besides the neutral and special houses that every skirmish has (glossary *house*). The expected values for
computer-owned units are the original's, and in our engine they hold for units of the neutral and
special houses too (Terms, the project lead's decision of 2026-10-10). Our engine's departure for AI houses (claims 11 and 15) is not
measurable against the original, which has no such houses.
Observations are taken with a *recorder* that reads unit world positions per tick (or from
screenshots when stated). Where a scenario counts ticks "from the first step to arrival", the count
is *t*ₐ − *t*₁ + 1, where *t*₁ is the first tick on which the unit's position changes and *t*ₐ the
tick on which it is placed on its final standing position: both ticks are included. Paths are
straight runs of clear cells, so S-0212 sends every walker along the run.

**Scenario W1: standing positions and capacity of the side positions (claims 1, 3, 4).**
- *Setup:* (a, b) an empty target cell *T* = (*x*, *y*). Four infantry of one stock type, owned by the
  human player, each pre-placed at sub-cell 2 (the right position): one in each of the cells
  (*x* − 1, *y*), (*x* − 2, *y*), (*x* − 3, *y*) and (*x* − 4, *y*), on the same row as *T*. (c) A
  separate game: a stock transport vehicle that infantry can enter, standing still in *T*, and one
  stock infantry type that can enter it, pre-placed at sub-cell 2 in (*x* − 1, *y*). (d) As (c),
  with the infantry unit pre-placed at sub-cell 0 (the centre) instead. (e) A separate game: *T*
  is a foundation cell, in the foundation's west column, of a capturable building of the computer
  player, and a stock infantry type that can capture buildings, owned by the human player, is
  pre-placed at sub-cell 0 in (*x* − 1, *y*). (f) As (d), with a landed stock transport aircraft
  that infantry can enter standing in *T* instead of the vehicle. In (d), (e) and (f) the W9
  recorder reads the draws taken on every tick.
- *Procedure:* (a) order the unit in (*x* − 1, *y*) into *T*, wait until it stops, record its
  position within the cell. (b) Then order the unit from (*x* − 2, *y*), then the one from
  (*x* − 3, *y*), then the one from (*x* − 4, *y*) into *T*, each after the previous one has stopped.
  Each walks east along the row and so reaches *T* from (*x* − 1, *y*). (c) Order the infantry unit
  to enter the transport, and record its position on every tick until it disappears. (d, f) As
  (c), with the vehicle or aircraft. (e) Order the unit to capture the building, and record its
  position on every tick until it disappears.
- *Observable:* (a, b) positions within the cell (*u*, *v*) at rest, and whether the fourth unit
  enters *T*. (c to f) the unit's position within the cell on each tick while it is in *T* or
  heading into it, and in (d to f) the number of draws taken on its asking tick for *T* (the last
  tick on which it does not move before it first steps into *T*).
- *Runs:* 1 of (a, b) and 1 each of (c) to (f). In (a) to (c) every desired point has class right,
  so claim 3 takes no draw.
- *Expected:* (a) (192, 64), the right position, since a unit at sub-cell 2 walking east keeps its
  position within the cell. (b) the second and third take (192, 192) then (64, 192) (row S2: bottom,
  then left). The fourth does not enter the cell (S-0209). (c) the unit keeps *v* = 64 as it walks
  east into *T*, heading towards (192, 64), the class of its desired point, although the cell holds a
  vehicle. (d, e, f) the unit keeps *v* = 128 as it walks east into *T*, heading towards (128, 128),
  the centre position, although the cell holds a vehicle, a building or an aircraft, and no draw
  is taken on its asking tick for *T*.
- *Falsified if:* any unit stands at (128, 128) or (64, 64) in (a) or (b), the order of (b) differs, a
  fourth unit enters, in (c) the unit stops outside *T* or its *v* moves towards 128 or 192 before
  it disappears, or in (d) to (f) the unit stops outside *T*, its *v* moves towards 64 or 192
  (a side position, as claim 3 would give) before it disappears, or a draw is taken on its asking
  tick for *T*.

**Scenario W2: pre-placed units, the top position and standing height (claims 1, 2, 3, 5, 13, Edge
cases).**
- *Setup and procedure:*
  - (a) A cell *T* holds one infantry pre-placed at sub-cell 1. Three more infantry of the human
    player stand in the three cells west of *T* on its row, each at sub-cell 2 as in W1. Order them
    into *T* one after another, each after the previous one has stopped.
  - (b) The record after the unit at the top position leaves (Edge cases,
    `quirks.top_position_counts_as_centre`). The record is read through the nearby-cell search, not
    through a vehicle: a remaining unit that stands still keeps an allied vehicle out through its own
    row whatever the record shows (S-0209 claim 5, row VP20), so a vehicle cannot show the record. A
    separate game per run, on open flat ground. A
    computer-owned fraidycat civilian *F* of an unarmed type stands in *guard* at the right
    position of a cell *X*, with the named test file of W7 part (c). The cell *T* map east of *X*
    holds two idle infantry of *F*'s house, of a type that is not a fraidycat, pre-placed at
    sub-cells 0 and 1. No other unit stands within 4 cells of *X*. (b1) A map trigger's move sends
    the unit at sub-cell 1 to the cell 4 cells map east of *T*, and the run waits until it has
    stopped there, so that only the unit at the centre is left in *T*. (b2) The control: no move,
    so both units stay in *T*. Then, in both variants, a unit of the human player at least 4 cells
    from *X* hits *F* once. As in W7 part (c), the hit does not move it, and its fear later makes
    it scatter by a request without a threat point (claim 13). The recorder reads the tick number
    (Terms) of every tick. W7 part (c) is run first, for its mapping from the residue (*N* mod 8)
    to the neighbour that claim 13 gives when all eight neighbours qualify.
  - (c) A separate game per run. One infantry unit pre-placed at sub-cell 1 in a cell, with an empty
    cell east of it. Order it one cell east and record the position it takes.
  - (d) A separate game. Five infantry units of the human player, of one stock type that is neither
    a fraidycat nor the cow, each alone in its own cell, the five cells at least 2 cells apart. The
    map gives them the sub-cell values 0, 1, 2, 3 and 4. No orders are given. Read each unit's
    position within the cell 30 ticks after the start (the idle actions of S-0107 claim 14 only
    turn such a unit; they do not move it).
  - (e) Standing height. A separate game on a test map with a cell *S* of slope shape 1 (S-0106
    claim 1, rising towards map east) at height level ℓ ≥ 0, whose west neighbour is flat at level
    ℓ; the test map shapes the other cells around *S* so that the slope is valid. The map also has
    a straight high bridge whose deck runs map east to west across at least three cells, with land
    at deck level at both ends. The recorder reads every unit's world height on every tick, and
    each cell's level and slope shape from the test map.
    - (e1) Two infantry of the human player, pre-placed at sub-cells 2 and 3 in a second cell *S*′
      of the same shape and level as *S*, away from the paths of (e2). No orders.
    - (e2) As W1 (a, b), with *S* as *T*: three infantry of the human player, each pre-placed at
      sub-cell 2 in one of the three flat cells west of *S* on its row, are ordered into *S* one
      after another, each after the previous one has stopped.
    - (e3) One infantry of the human player, standing on the land at the west end of the bridge,
      ordered to the middle cell of the deck.
- *Observable:* (a) the positions taken in *T*, and whether all three enter. (b) the first cell *F* moves
  to, as a step direction from *X*, and the residue of the tick number mod 8 on the tick its
  destination was set. (c) the
  position taken. (d) the (*u*, *v*) of each of the five units. (e) the world height and the
  (*u*, *v*) of each unit at rest, and in (e2) and (e3) its height on every tick of its walk.
- *Runs:* (a) 1. (b) 40 of each variant, starting on different ticks so that the residues vary. (c) 200, a new game for each run (a different random state each
  time). (d) 1. (e) 1.
- *Expected:* (a) all three enter and take the three side positions, so *T* holds four infantry
  (claim 5 and S-0209 claim 2). (b) in (b1) the record of *T* shows no position taken, so *T*
  qualifies for the search like the other seven neighbours: every run follows the mapping of W7
  part (c), and each run whose residue that mapping sends to direction 2 has *T* as its
  destination, although a unit still stands at *T*'s centre. In (b2) the destination is never *T*.
  (c) the desired point (64, 64) has class centre, so the draw of claim 3
  applies: right in 80 to 120 runs, left and bottom each in 32 to 68 runs, never top or centre.
  (d) the positions of table P: (128, 128) for value 0, (64, 64) for 1, (192, 64) for 2, (64, 192)
  for 3 and (192, 192) for 4.
  (e) every unit at rest in *S* or *S*′ stands at the height *H* that S-0106 claim 2 gives for the
  cell's level and shape at its (*u*, *v*): in (e1) at (192, 64) and (64, 192), in (e2) at (192, 64),
  (192, 192) and (64, 192) (as W1), so that the units at *u* = 192 stand higher than those at
  *u* = 64. In (e3) the unit at rest stands 416 leptons above the *H* of its deck cell's terrain at
  its (*u*, *v*). The heights during the walks have no expected value (Open questions).
- *Falsified if:* (a) only two enter, or one of them takes the top or centre position. (b) in (b1) a run
  whose residue the mapping of W7 part (c) sends to direction 2 has another destination than *T*
  (what the switch turned off gives, since *T*'s record then shows the centre taken), or any run
  of (b1) departs from that mapping; in (b2) any run has *T* as its destination. (c) any run takes top or centre, or a count falls outside its band; in
  particular, right in every run would show that the top quadrant has a class of its own. (d) any
  of the five positions differs from table P, in particular value 1 at (128, 128). (e) any height
  at rest differs from the expected one, for example a unit in *S* or *S*′ at the height of the cell's
  centre or of the flat cell it came from.

**Scenario W3: the position-order draw (claim 3).**
- *Setup:* one infantry pre-placed at sub-cell 0 (the centre) of a cell, with an empty cell east of
  it.
- *Procedure:* order it one cell east. Record the position it takes. Start a new game for each run
  (a different random state each time).
- *Observable:* the position taken.
- *Runs:* 200.
- *Expected:* right about half the time, left and bottom about a quarter each, never top or centre.
  The count of right lies in 80 to 120, and the counts of left and of bottom each in 32 to 68 (about
  three standard deviations).
- *Falsified if:* any run takes top or centre, or a count falls outside its band.

**Scenario W4: walking step and speed (claims 6, 7).**
- *Setup:* every stock type and setting below is chosen so that F5 gives *s* from 2 to 38 in every
  part (claim 6). A straight east-west run of 12 clear cells per unit, side by side, each unit owned by the
  human player and pre-placed alone at sub-cell 2 in the west end cell of its run. No crates on the
  runs. The house's country is noted, and its country speed factor for infantry is read from the
  rules data.
  - (a) Three stock infantry types with different speed fields, at the lowest veterancy rank.
  - (b) One stock infantry type whose veteran ability list names the faster ability, placed by the
    test map at veteran rank, beside a unit of the same type at the lowest rank. The type is chosen so
    that trunc(*s*₁ × *k*ᵥ) differs from *s*₁ for the stock veteran speed factor; if no stock type
    gives such a pair, the run still checks that *s* = trunc(*s*₁ × *k*ᵥ).
  - (c) One stock infantry type, and a crate placed on the unit's own run one cell east of its start
    cell. A named test file (an override section of the test map, or a loose copy of the rules file)
    makes the speed crate the only crate outcome. The crate's speed multiplier is read from the rules
    data.
- *Procedure:* order each unit to the cell 10 cells east of its start.
- *Observable:* the world *x* and *y* of the unit on every tick of the walk, and the number of ticks
  from the first step to arrival.
- *Runs:* 1 per unit.
- *Expected:* on every tick of steady walking *x* grows by exactly the *s* of F5, and *y* does not
  change: (a) from the speed field and the country factor, with *m* = 1.0 and no veteran factor; (b)
  the veteran unit's *s* = trunc(*s*₁ × *k*ᵥ), where *s*₁ is the step of the lowest-rank unit beside
  it; (c) after the crate is taken, *s* = trunc((*b* × *k*꜀) × *m*) with *m* the crate's multiplier,
  and *s*₁ of (a) before it. In each cell the walker makes ⌈240 / *s*⌉ steps and then spends one
  tick being placed exactly on the next standing position, and before each cell but the first it
  spends one tick without moving, choosing its walking target. So for a unit whose speed does not
  change the walk of 10 cells takes 10 × (⌈240 / *s*⌉ + 2) − 1 ticks from the first step to arrival
  (for *s* from 2 to 38, claim 6). A count of 10 × (⌈240 / *s*⌉ + 1), 9 ticks fewer, would mean that
  the walking target is chosen on the arrival tick; the per-tick record then shows no tick without
  movement between two cells.
- *Falsified if:* any steady step differs from the expected *s*, the final position is not exact,
  *y* changes, or the number of ticks differs.

**Scenario W5: crawling, panic and the prone non-crawler (claims 8, 10).**
- *Setup and procedure:*
  - (a) A unit of the human player, of a stock infantry type that can crawl, on open ground, ordered
    to a cell ten cells away. While it walks (its behaviour is then *move*, in which a
    human-controlled unit does not retaliate, S-0215 claim 18), a computer-owned unit with a weak
    weapon hurts it once, enough to raise its fear to lie down (S-0107 claim 16) without killing it.
    If it stops when it lies down, order it on at once. Record its per-tick step while prone.
  - (b) The same unit and the same move, unhurt.
  - (c) A computer-owned fraidycat civilian type hurt by a known attacker, which panics and flees.
    Record its per-tick step while it shows *panic*.
  - (d) A named test file (a loose copy of the rules file, and of the art file if needed) marks a
    stock infantry type of the human player as a cyborg whose art cannot crawl. Order the unit to a
    cell ten cells away and, while it walks, hit it with an attack from a known attacker that
    deals more than its remaining health, so that it survives as a crippled cyborg. If it stops after
    the hit, order it on at once. Record its per-tick step during the first 40 ticks after the hit,
    before its fear drops below 50 and the get-up rule of S-0107 claim 15 can apply. The 40 ticks
    rest on S-0107's engine constants: a hit by a known attacker gives at least 100 fear points
    (claim 16), fear decays by 1 fear point per tick and the unit gets up only below 50 (its
    Constants), so at least 51 ticks pass before it can get up.
- *Observable:* steady per-tick displacement.
- *Runs:* 1 each.
- *Expected:* (a) *s* − ⌊*s*/3⌋, (b) *s*, (c) *s* of its type (no change by panic), (d)
  *s* + ⌊*s*/2⌋ after the hit, where *s* is its step before the hit.
- *Falsified if:* any steady step differs; in (d) in particular, a step of *s* or of *s* − ⌊*s*/3⌋
  after the hit.

**Scenario W6: terrain does not slow walkers (claim 9).**
- *Setup:* two straight runs of 10 cells side by side, one of clear land, one of a land type whose
  walker percentage is between 0 % and 100 % in stock data. One infantry of the same type at the
  start of each, at sub-cell 2.
- *Procedure:* order both to the far end on the same tick.
- *Observable:* ticks from the first step to arrival.
- *Runs:* 1.
- *Expected:* equal.
- *Falsified if:* the two counts differ (in particular, the unit on the slower land type takes
  longer).

**Scenario W7: scatter acceptance, destinations and damage requests (claims 11 to 13, 15).**
- *Setup:* Unless a part says otherwise, units are at the lowest veterancy rank, and the
  player-scatter rule is off: if stock data turns it on, a named test file (a loose copy of the rules
  file) turns it off. Where a part needs a behaviour with the scatter property and stock data does not
  give it, a named test file gives it. In parts c, g, h1, h4 to h7, i and j the unit that scatters stands on
  open flat ground with no other unit within 2 cells, so its search collects exactly its eight
  neighbours (claim 13). The recorder reads the tick number (Terms) of every tick. For a unit that
  walks when a request reaches it, *Z* is its scatter centre cell on the tick of the request: the
  cell of the standing position it is walking to (the next cell of its run), or its own cell when
  the request falls on its arrival tick or asking tick (claim 6). The recorder reads it from the
  walker's position and walking target on that tick.
  - (a) Four infantry units on open ground: an idle soldier of the human player standing in the path
    of an enemy crusher driving towards it (a forced request, S-0210); a soldier of the human player
    walking across the crusher's path at the moment it approaches (these two in a separate game, in
    which a named test file turns the player-scatter rule on, so that the crusher's request, which
    is not move-anywhere, reaches them by S-0209 claim 11); an idle soldier of the human
    player hit once by a computer-owned unit; and an idle computer-owned fraidycat civilian of an
    unarmed type, whose behaviour has the scatter property, hit once by a unit of the human player.
  - (b) A computer-owned fraidycat civilian of an unarmed type (so that it may not retaliate and
    cannot attack back) walking north along a straight run of open flat ground
    under a map trigger's move to a cell *D* 8 cells away, whose move behaviour has the scatter
    property. When it is 4 cells from *D* it is hit once by a long-range weapon fired from due map
    east, from a unit on its row. Its locomotor is moving, so the hit sends it request B only
    (claim 15).
  - (c) An idle computer-owned fraidycat civilian of an unarmed type, standing in *guard* on open
    flat ground. A named
    test file removes the scatter property from *guard* (if stock data gives it the property) and
    leaves the player-scatter rule off. A unit of the human player at least 4 cells away hits it
    once. By claim 15 neither case of request A is sent (the first needs the player-scatter rule or
    the scatter ability, the second the scatter property), and claim 11 refuses request B (it is not
    forced and *guard* lacks the scatter property), so the hit does not move it. Its fear then
    makes it scatter on its own, by the flee rule (S-0107 claim 15) or by the idle scatter of a
    frightened fraidycat (S-0107 claim 14). Both send a forced request without a threat point, which
    claim 11 accepts while its locomotor is not moving, and claim 13 gives its destination either
    way.
  - (d) The runs of (b), read for where the new destination lies.
  - (e) A gap cell *G* in an otherwise impassable straight line (for example a cliff line), with open
    ground on both sides. *G* holds three computer-owned infantry standing still on its three side
    positions; one of them is of a stock type that can deploy, whose *undeploy* entry has image
    frames, and is deployed, playing *deployed* (the recorder shows its sequence). A map trigger orders a
    fourth infantry unit of the same house, standing 2 cells before *G*, to a cell 10 cells beyond
    *G* through the gap. *G* gives it the outcome *stationary friendly* (S-0209 claim 4), and since its
    destination is farther than the close-enough distance (S-0209 Terms) it sends every occupant of
    *G* a forced, move-anywhere request without a threat point (S-0209 claim 8).
  - (f) As (b), but on uneven ground around *Z*₀, the cell of the run 3 cells before *D* (the
    walker's next cell when it is hit 4 cells from *D*); only runs whose *Z* is *Z*₀ count. The map
    raises some cells near *Z*₀ so that, for at
    least one of the five draws, the first qualifying candidate fails the screen-cover check. For
    example, the cell map south of *Z*₀ stands at least one height level above the cell map west of
    *Z*₀, which makes the west candidate fail (the cell one step towards the bottom of the screen
    from it is that raised cell). The recorder reads every cell's height level and passability from
    the test map.
  - (g) An idle soldier of the human player, not a fraidycat, on open flat ground, in a behaviour in
    which it may retaliate (S-0215 claim 18). A computer-owned unit with a long-range weapon hits it
    once from beyond the soldier's sight, so that it does not attack back (S-0215 F7). Three variants:
    (g1) a named test file turns the player-scatter rule on; (g2) the rule off; (g3) the rule off,
    with the soldier placed by the test map at veteran rank and a named test file whose veteran
    ability list for its type names the scatter ability. Each variant once with the attacker due
    map east and once due map west of the soldier.
  - (h) Three units, each hit once, each run once with the attacker due map east and once due map
    west: (h1) an idle computer-owned infantry unit that is not a fraidycat and may not retaliate
    (Terms; for example an unarmed type that is not an engineer, or a stock civilian type
    that a named test file makes not a fraidycat), in a behaviour with the scatter property, hit by
    a unit of the human player; (h2) the same type owned by the human player, hit by a computer-owned
    unit; (h3) an idle armed computer-owned soldier, not a fraidycat, in a behaviour with the
    retaliate property, of a stock type whose weapon can hit the attacker, so that it may retaliate
    (Terms), hit by a unit of the human player. Four more variants of (h2) and (h1): (h4) as (h2),
    with a named test file that turns the player-scatter rule on; (h5) as (h2), with the rule off,
    the unit placed by the test map at veteran rank and a named test file whose veteran ability
    list for its type names the scatter ability; (h6) as (h1), with the unit pre-placed by the map
    as owned by the neutral house (S-0009 claim 27); (h7) as (h1), owned by the special house in
    the same way. In (h6) and (h7) the unit of the human player is ordered to attack it. The game has
    a computer player, so the unused-house cleanup of S-0009 claim 28 does not run and both houses
    keep their pre-placed units; the variant counts only when the recorder shows the unit present
    at the start.
  - (i) Damage without a known attacker. First pick a damage source that carries no attacking
    object (which sources do belongs to the combat specs; a radiation field is the expected
    candidate) and check it with the fear rules of S-0107 claims 15 and 16 alone: an idle soldier of
    the human player at fear level 0 does not lie down after one hit from the source, but lies down
    after one hit from a known attacker that fires from beyond its sight (S-0215 F7), so that it
    does not retaliate. Then take two soldiers that are not fraidycats, each idle at fear level 0 in
    a behaviour with the scatter property, on open flat ground with no other unit within 3 cells:
    (i1) one owned by the computer player, (i2) one owned by the human player. Each takes one hit
    from that source that it survives; record from the tick of its first hit.
  - (j) Both damage requests at once (claim 15). An idle computer-owned fraidycat
    civilian standing in *guard* on open flat ground, with no other unit within 3 cells. A named
    test file gives *guard* the scatter property and removes its retaliate property, so that the
    unit may not retaliate (Terms) and the hit sends it request A by its second case as well as
    request B. A unit of the human player due map east of it, 5 cells away on its row, hits it once.
    Before the runs, the mapping from the residue (*N* mod 8) to the neighbour that claim 13 gives
    is taken from the runs of (c) and (i1). Variants: (j1) the civilian as above, owned by the
    computer player; (j2) the civilian pre-placed by the map as owned by the neutral house (S-0009
    claim 27), and attacked on the user's order; (j3) the same with the special house. The game
    has a computer player, so the unused-house cleanup of S-0009 claim 28 does not run; (j2) and
    (j3) count only when the recorder shows the civilian present at the start.
  - (k) The fallback of claim 13. A purpose-made map with a flat block of 5 × 5 cells raised at
    least 2 height levels above every cell outside it that infantry can enter (the cells around
    the block are cliffs, water or lower ground), far from the players' start positions. The
    unit of (c), with the same test file, is pre-placed at the right standing position (map value
    2) of the centre cell of the block. Each of the other 24 cells of the block holds an occupant: a vehicle of
    the human player in one corner cell, armed and ordered to hit the civilian once, and in each of
    the other 23 cells one idle, unarmed infantry unit of the civilian's house, of a type that is not
    a fraidycat (a named test file may make a stock civilian type not a fraidycat, as in (h1)),
    pre-placed at a side position. Every cell of the block then has a non-empty occupancy record,
    and every cell outside it either cannot be entered or differs from the centre cell by more than
    1 height level (the search's height condition), so the nearby-cell search finds nothing
    (S-0212 claim 25), while each of the eight
    neighbours of the centre cell holds one allied infantry unit and is enterable (S-0209 claim 4).
    As in (c), the hit itself does not move the civilian, and its fear later makes it scatter
    without a threat point.
  - (l) The exclusion of high-bridge cells (claim 12). A test map with a straight high bridge that
    runs map south from land at deck level. The land end cell *E* carries no deck; the deck cells
    *K*, map south of *E*, and *K*₂, map south of *K*, are high-bridge cells. The recorder reads
    from the test map which cells carry the deck, and the part counts only when *E* carries none.
    Every neighbour of *K* other than *E* and *K*₂ is not enterable for infantry (water below the
    deck, or cliffs). No cell within 6 cells of *E* or of *K*₂ along the diagonal towards the bottom
    of the screen stands higher than that cell, so both pass the screen-cover check and only the
    exclusion can pass *K*₂ over. A computer-owned fraidycat civilian of an unarmed type, whose move
    behaviour has the scatter property, walks from *E* onto *K* under a map trigger's move to *K*.
    While it walks it is hit once by a long-range weapon fired from due map north, by a unit on land
    beyond *E*. Its locomotor is moving, so the hit sends it request B only (claim 15), and its
    scatter centre cell is *K*. The threat lies due map north, so *d*₀ is map south and the first
    choice *d* is one of the directions 2 to 6.
  - (m) The first case for a unit that is not deploying (claim 11). As (e), with these changes: no
    occupant of *G* can deploy; the three occupants and the fourth unit belong to the human player,
    the occupants of a stock type whose *cheer* entry has image frames; and the user moves the fourth unit
    by a move order. When the fourth unit is one cell short of the cell before *G*, the user gives
    the three occupants the celebration order, so that they play *cheer* (S-0107 claim 21) when the
    fourth unit stops in front of *G* and starts sending its forced, move-anywhere requests (S-0209
    claim 8). If the recorder shows a cheer ending before the fourth unit stops, the run is repeated
    with the order given later.
  - (n) The second case of claim 11 (deploying, human-controlled owner, request not both forced and
    move-anywhere). A stock infantry type that can deploy, owned by the human player, on open flat
    ground with no other unit within 2 cells, not a fraidycat, in a behaviour in which it may
    retaliate (S-0215 claim 18). A named test file turns the player-scatter rule on, so that the
    unit has scatter permission. A computer-owned unit with a long-range weapon, outside the
    soldier's sight (S-0215 F7), hits it once, so that it does not attack back, and the hit sends
    it request A (claim 15, first case), which is forced and not move-anywhere. Variants, each hit
    given on a tick when the unit plays: (n1) *deploy*, (n2) *deployed*, (n3) not a deploy kind
    (the same unit undeployed, standing, as the control). A unit playing *deployed-fire* has a
    target, so it is not sent request A (claim 15); variant (n4) uses the crushing request
    instead: the same unit, deployed and playing *deployed-fire* at a computer-owned unit in its
    weapon range, while a computer-owned crusher, under a map trigger's move, begins a track whose
    first cell is the unit's cell, so that the unit is sent a forced request that is not
    move-anywhere (S-0210 claim 7, through S-0209 claim 11, which the player-scatter rule lets
    reach it). (n5) The owner condition of the second case: a unit of the same type, not a
    fraidycat, owned by the computer player and deployed as the occupant of part (e) is, with the
    player-scatter rule off. A named test file gives its current behaviour (the recorder shows it)
    the scatter property and removes that behaviour's retaliate property, so that it may not
    retaliate (Terms), and a unit of the human player hits it once, which sends it request A by
    its second case (claim 15), forced and not move-anywhere. The variant counts only when the
    recorder shows it playing *deployed* on the tick of the hit. *Deployed-idle* is left out,
    since no stock behaviour starts it (S-0107 claim 20).
  - (o) The *undeploy* table of claim 11 (its *deploy*, *deployed* and *deployed-fire* rows). As
    (e), with these changes: the three occupants of *G* and the fourth
    unit belong to the human player; the occupants are of a stock type that can deploy, and the user
    gives them the deploy order. The fourth unit, ordered through the gap, stops in front of *G* and
    starts sending its forced, move-anywhere requests (S-0209 claim 8). Variants, by the sequence
    that all three occupants play on the tick of the first request, reached by giving the deploy
    order earlier or later (as in (m), the recorder shows the sequences): (o1) *deploy*,
    (o2) *deployed* and (o3) *deployed-fire* (by an enemy unit in the occupants' weapon range), with
    a stock type whose *undeploy* entry has image frames; and (o4) *deployed*, with a named test
    file (a loose copy of the art file) that gives the occupants' type an *undeploy* entry with no
    image frames. *Deployed-idle* is left out, since no stock behaviour starts it (S-0107 claim 20).
    If the recorder shows another sequence on that tick, the run is repeated with the order given
    earlier or later.
  - (p) The height-aware usable area (claim 12). A purpose-made map whose cells are all flat at
    height level 2, so that the height-aware test of S-0009 claim 4 moves the top edge of the
    usable area two screen rows down from where the plain test puts it. *D* is a cell on the
    first screen row inside the height-aware area, away from the left and right edges, so that
    its map west, map north-west and map north neighbours are inside the area by the plain test
    and outside it by the height-aware test, while its map north-east and map south-west
    neighbours, on its own screen row, are inside it by both. The recorder reads the local area
    from the test map. A computer-owned fraidycat civilian of an unarmed type, whose move
    behaviour has the scatter property, walks map north-east along the screen row of *D*, through
    *D*, under a map trigger's move to the cell 3 cells beyond *D* on that row. It is hit once by a
    long-range weapon fired from a unit due map south-east of it (on its cell diagonal towards the
    bottom of the screen), on a tick on which it walks towards a standing position in *D*; runs in
    which the hit falls on another tick do not count. Its locomotor is moving, so the hit sends it
    request B only (claim 15), and its scatter centre cell *Z* is *D*. *d*₀ is map north-west (direction 7), so the first choice *d* is
    5, 6, 7, 0 or 1 for the draws 0 to 4.
  - (q) Scatter permission of a fraidycat whose owner is human-controlled (claim 11). A fraidycat
    civilian of an unarmed type, owned by the human player, at the lowest veterancy rank unless a
    variant says otherwise, walks north along a straight run of open flat ground under a move
    order to a cell *D* 8 cells away, in a move behaviour with the scatter property (a named test
    file gives it if stock data does not). When it is 4 cells from *D* it is hit once by a
    long-range weapon fired from due map east by a computer-owned unit on its row. Its locomotor is
    moving and it has a destination, so the hit sends it request B only (claim 15), which is not
    forced, and claim 11's third case decides by its scatter permission. Variants: (q1) the
    player-scatter rule off; (q2) a named test file turns it on; (q3) the rule off, the unit placed
    by the test map at veteran rank, with a named test file whose veteran ability list for its type
    names the scatter ability.
  - (r) The scatter centre cell of a walking unit, for a request without a threat point (claim
    13). A computer-owned fraidycat civilian of an unarmed type, whose move behaviour has the
    scatter property, walks along a straight run of open flat ground under a map trigger's move to a
    cell *D* at least 8 cells away. An idle infantry unit of its house, not a fraidycat, stands in
    *D*, so that *D*'s record is never empty and the search never gives *D* itself (S-0212 claim
    25); no other unit stands within 2 cells of *D*. A crusher of the human player, under the
    user's move order, crosses the run so that its forced request (S-0210 claim 7, as in part a)
    reaches the civilian while the civilian is at least 4 cells from *D*. A named test file
    turns the player-scatter rule on, so that the request reaches the civilian by S-0209 claim 11
    whatever the skill levels; it does not change claim 11 here, where the civilian already has
    scatter permission through its owner. The civilian's locomotor
    is moving, so the request is not effectively forced, and claim 11's third case accepts it
    (fraidycat, scatter property, computer-controlled owner). Runs in which the crusher's track
    passes within 2 cells of *D* do not count.
- *Procedure:* observe whether each unit moves away, and to which cell.
- *Observable:* moved or not, and the tick on which its new destination was set. In (b) and (f), the
  cell of its new destination, given as a step direction from *Z* in (b), and *Z* itself. In (c), (g), (h) and (i),
  the first cell it moves to, as a step direction from its own cell, together with the residue of
  the tick number mod 8 on the tick its destination was set. In (d), whether the cell it moves to
  is a neighbour of *Z*, of *D* or of the cell it was in when hit. In (e), whether it undeploys and leaves
  *G*. In (j), the destination it holds at the end of the tick of the hit (the first cell it walks
  to), as a cell relative to its own cell, and the residue of the tick number mod 8 on that tick.
  In (k), the first cell the civilian moves to, as a step direction from its cell, and the residue
  of the tick number mod 8 on the tick its destination was set. In (l), the cell of its new
  destination. In (m), (n) and (o), for each unit and each tick, its current sequence and whether
  it has a destination, and, from W9, whether a draw was taken on the tick of the request. In (p),
  the cell of its new destination, as a step direction from *D*. In (q), whether its destination
  changes on the tick of the hit and, if it does, the new destination as a step direction from
  *Z*. In (r), *Z*, and the cell of the new destination set on the tick of the crusher's request,
  with its distance in cells from *D* and from *Z*. A move towards the attacker by a
  unit of an armed type counts as an attack, not a scatter; for a unit of an unarmed type every
  move counts as a scatter.
- *Runs:* (a) 1 each. (b) and (f) until 100 runs have counted (see Expected). (c) 20. (d) the runs of
  (b). (e) 1.
  (g), (h1) and (h4) to (h7) 12 per variant and attacker side, (h2) and (h3) 1 per attacker side. (i) 12 per
  soldier. (j1) 40, (j2) and (j3) 20 each. (k) 40. (l) until 50 runs have counted (counted as in (b)). (m) 1. (n) 12 per variant. (o) 1 per variant. (p) until 100 runs have counted (counted as in
  (b)). (q1) 12; (q2) and (q3) until 50 runs have counted (counted as in (b)). (r) until 20 runs
  have counted. Runs start on
  different ticks so that the residues mod 8 vary.
- *Expected:*
  - (a) the idle soldier in the crusher's path moves, the walking soldier does not change its course,
    the soldier hit by a known attacker does not scatter (it may attack back), the hit fraidycat
    moves on the tick of the hit.
  - (b) The flee rule of S-0107 claim 15 can redirect a fraidycat later, so only runs whose
    destination changes on the tick of the hit count. In the counted runs the new destination is a
    neighbour of *Z* in the relative directions −2 … +2 around map west only, each in 8 to 32 of the
    100 counted runs (about three standard deviations around 20), and never in the three directions
    pointing back towards the threat.
  - (c) the hit does not move the unit. Its first move goes to a neighbour of its own cell, never
    staying in its own cell, and runs whose request ticks have the same residue mod 8 move to the
    same neighbour, whatever the draw (claim 13: the neighbour at position (*N* mod 8) of the
    search's collection order).
  - (d) in every run the cell it moves to is a neighbour of *Z*: of the next cell of its run when
    the hit falls on a tick on which it walks, of its own cell when the hit falls on its arrival
    or asking tick, and never a neighbour of *D* that is not also a neighbour of *Z*.
  - (e) the deployed unit starts undeploying on the tick the request arrives (S-0107 claim 6 row 6,
    since *deployed* is interruptible; the undeploy table of claim 11), and later leaves *G*.
  - (f) in every counted run the new destination is the cell that claim 12 gives, from the recorded
    levels, for one of the draws 0 to 4. Each predicted destination occurs with probability *n*/5,
    where *n* is the number of draws that lead to it; its count lies within three standard
    deviations of 100 *n*/5. A candidate that fails the check is never chosen while a later
    qualifying candidate passes it.
  - (g) in (g1), with the rule on, and in (g3), with the scatter ability, the soldier moves on the
    tick of the hit to a neighbour of its cell (request A, claim 13), and runs whose hit ticks have
    the same residue mod 8 give the same neighbour, whichever side the attacker is on. In (g2),
    with the rule off and no scatter ability, it does not move.
  - (h) (h1), (h4), (h5), (h6) and (h7) each move on the tick of the hit to a neighbour of the
    unit's cell, the same neighbour for equal residues mod 8 whichever side the attacker is on
    (request A, second case: (h1), (h6) and (h7) through a computer-controlled owner, (h4) through
    the player-scatter rule, (h5) through the scatter ability). (h2) does not scatter. (h3) does not scatter: it attacks back at once, as a computer-controlled actor does
    (S-0215 claim 18).
  - (i) (i1) moves on the tick of the hit to a neighbour of its cell, the same neighbour for equal
    residues mod 8 (request A, second case, through claim 13). (i2) does not move.
  - (j) in every variant the unit moves on the tick of the hit. In (j2) and (j3) this shows that
    the house gives the unit scatter permission and the second case of request A as a
    computer-controlled owner does: a unit treated as human-controlled would be sent no request A
    (the player-scatter rule is off) and would refuse request B (no scatter permission), so it
    would not move on that tick. Let *A* be the neighbour of its cell that the residue
    gives (the destination of request A). Request B ends as its destination, and B looks around
    the unit's own cell (claim 15). Two sets are named, each the set of destinations that claim 12
    gives for the five draws 0 to 4, with the threat due map east (so *d*₀ is map west): *H*₁ with
    the unit's own cell as scatter centre cell, and *H*₂ with *A* as scatter centre cell. On open
    ground these are the neighbours of the own cell, respectively of *A*, in the relative
    directions −2 … +2 around map west, unless the first candidate is not enterable. In every run
    the destination lies in *H*₁.
  - (k) the civilian moves, and its first cell is the neighbour of its cell in step direction
    (1 + *j* − 2) mod 8 for a draw *j* from 0 to 4: the facing from its cell's centre to the right
    position rounds to direction 1 (F6), so the directions are 7, 0, 1, 2 and 3, each with
    probability 1/5 (each in 1 to 15 of the 40 runs, about three standard deviations), and never 4, 5 or 6. Runs whose request ticks
    have the same residue mod 8 can end in different directions.
  - (l) in every counted run the new destination is *E*. For the draws 0, 1 and 2, *K*₂ comes
    before *E* in the order of preference, but it is a high-bridge cell, and *E* is a later
    qualifying candidate that passes the screen-cover check.
  - (m) no occupant gets a new destination on a tick on which its current sequence is *cheer*.
    Each occupant leaves *G* only after its cheer has completed, on a later request of the fourth
    unit.
  - (n) in (n1) and (n2) the unit takes no new destination and no draw on the tick of the hit, and
    in (n4) none on the tick of the crusher's request, and it keeps playing its deploy-kind
    sequence. Only (n2) and (n4) isolate the second case, which refuses there although the unit
    has scatter permission and the request is effectively forced. In (n1) the first case refuses
    as well, since *deploy* is not interruptible. In (n3), the control, the unit moves to a
    neighbour cell, as in (g1). In (n5) the unit takes a new destination on the tick of the hit:
    the second case does not refuse, since its owner is not human-controlled, and the first case
    does not, since *deployed* is interruptible. Whether and when it then leaves its cell is not
    part of this expectation.
  - (o) in (o1) no occupant gets a new destination and none takes a draw on a tick on which it
    plays *deploy*; each leaves *G* only after its *deploy* has completed, on a later request of the
    fourth unit (the first case refuses: *undeploy* is turned down while *deploy* plays, and *deploy*
    is not interruptible). In (o2) and (o3) every occupant starts *undeploy* on the tick of the
    first request, and takes a new destination and a draw on that same tick. In (o4) every
    occupant keeps playing *deployed* (*undeploy* is turned down) and still takes a new
    destination and a draw on the tick of the first request.
  - (p) in every counted run the new destination is the map south-west neighbour of *D* (draw 0)
    or its map north-east neighbour (draws 1 to 4): south-west in 8 to 32 and north-east in 68 to
    92 of the 100 counted runs (about three standard deviations). For the draws 1, 2 and 3 the
    plain test would let the map west, map north-west or map north neighbour qualify first.
  - (q) in (q1) the destination never changes on the tick of the hit (claim 11 refuses request B:
    no scatter permission). In (q2) and (q3) it changes on the tick of the hit in every run, to a
    neighbour of *Z* in the relative directions −2 … +2 around map west, as in (b), never in the
    three directions pointing back towards the threat.
  - (r) in every counted run the new destination is a neighbour of *Z* (the search around the
    scatter centre cell *Z*, claim 13), so within 2 cells of the civilian, set on the tick of the
    crusher's request, and not a neighbour of *D*, which lies at least 3 cells from the civilian.
- *Falsified if:* (a) a different move-or-stay pattern. (b) a direction pointing towards the threat,
  or a count outside 8 to 32. (c) the hit itself moves the unit, two first moves with the same
  residue mod 8 end in different cells, or it stays although free cells are next to it. (d) it
  moves to a cell that is not a neighbour of *Z* (a neighbour of *D* far from *Z* would mean the
  centre is the farther destination, as revision 13 of this spec said). (e) it
  stays deployed, or undeploys but never leaves *G* although cells next to the gap are free. (f) any counted destination differs from every predicted one; a candidate that fails
  the check is chosen while a later qualifying candidate passes; or a count falls outside its band
  (in particular, if the west candidate is chosen at the rate flat ground would give). (g) in (g1) or
  (g3), it stays while free cells are next to it, or two runs with the same residue mod 8 end in
  different cells; in (g2), it scatters. (h) (h1), (h4), (h5), (h6) or (h7) stays while free cells
  are next to it, or two of its runs with the same residue end in different cells; (h2) or (h3)
  scatters. (h6) or (h7) staying would mean that the original treats that house as
  human-controlled. (i) the check of
  the source fails (pick another source); (i1) stays while free cells are next to it, or two runs
  with the same residue end in different cells; (i2) moves. A scatter of (h3), or (i1) staying,
  would contradict this spec's claim 15 and S-0107's revised scenario M15(b), and so would the hit
  itself moving the unit in (c). (j) in any variant the unit does not move on the tick of the hit,
  or a destination lies outside *H*₁: in *H*₂ only (request B would then look around request A's
  destination), or in neither set (for example *A* itself when *A* is in neither, which would mean
  request B was refused or was judged before request A). (k) the civilian never moves while the neighbours are
  enterable (no fallback), it moves in direction 4, 5 or 6, a count falls outside its band, or every
  pair of runs with the same residue mod 8 ends in the same direction while the directions follow
  the residue (which would mean a search result and not the draw). (l) the destination is *K*₂ in
  any counted run (without the exclusion it would be *K*₂ in about three runs out of five). (m) an
  occupant gets a new destination on a tick on which its recorded sequence is still *cheer*, which
  is what a first case that exempted every forced, move-anywhere request would give. (n) in (n1)
  or (n2) it moves or takes a draw on the tick of the hit, or in (n4) on the tick of the crusher's
  request; in (n3) it stays although free cells
  are next to it; in (n5) it takes no new destination on the tick of the hit (what a second case
  without its owner condition would give). (o) in (o1) an occupant moves or takes a draw while it plays *deploy*; in (o2) or
  (o3) an occupant does not start *undeploy* on the tick of the first request, or takes no new
  destination or no draw on that tick; in (o4) an occupant starts *undeploy*, or takes no new
  destination or no draw on that tick. (p) a counted destination is the map west, map north-west
  or map north neighbour of *D* (which the plain form of the usable-area test would give), or a
  count falls outside its band. (q) in (q1) the destination changes on the tick of the hit in any
  run; in (q2) or (q3) a run keeps its destination on the tick of the hit, or a new destination
  points towards the threat. (r) a counted new destination is a neighbour of *D* (the scatter
  centre cell would then be the farther destination), or is not a neighbour of *Z*, or the
  destination does not change on the tick of the request although the neighbours of *Z* are free.

**Scenario W8: sideways drift on the map axes, and very slow and very fast walkers (claims 6 and
14, Edge cases).**
- *Setup:* four infantry of one type, owned by the human player, each pre-placed at sub-cell 2 (the
  right position) in the end cell of its own straight run of 12 clear cells: one run along map
  north, one along map east, one along map south and one along map west. A unit at a side position
  keeps that position in every cell of a straight run (claim 3), so each leg is 256 leptons long and
  parallel to its run, and no random draw is involved. The house's country speed factor for
  infantry is read from the rules data.
  - (a) A stock type at the lowest veterancy rank, chosen so that F5 gives *s* from 2 to 38 for the
    house's country (claim 6).
  - (b) A named test file (a loose copy of the rules file) sets the type's speed field and the
    veteran speed factor so that F5 gives *s* = 1 for a unit at veteran rank whose type has the
    faster ability; the test map places the four units at veteran rank.
  - (c) A named test file sets the type's speed field so that F5 gives *s* = 46 at the lowest rank.
- *Procedure:* (a) order each unit to the cell 10 cells along its run. (b, c) order each unit to the
  next cell along its run. Record the world position per tick, for at least 1000 ticks in (b) and
  600 in (c).
- *Observable:* the number of ticks from the first step to arrival (or that it does not arrive), and
  the largest sideways deviation from the straight line through its standing positions during the
  walk.
- *Runs:* 1 per direction and part.
- *Expected:*
  - (a) north and east take 10 × (⌈240 / *s*⌉ + 2) − 1 ticks; west and south take the numbers of
    ticks that F3 and F7 give for its *s*, with the same one non-moving tick before each cell after
    the first (claim 6), never fewer (for the example speeds *s* = 4, 9, 11 and 16, south: 809, 319,
    259 and 179 ticks, against 619, 289, 239 and 169 north; west equals north at these speeds; at
    *s* = 21 all four are 139). If W4 finds the count 9 ticks lower (claim 6), every value here is
    9 ticks lower too. On the early steps of each cell the south walker's
    *x* falls by 1 lepton per step and the west walker's *y* falls by 1 lepton per step; north and
    east show no sideways deviation. The deviation never exceeds 3 leptons going south and 6 going
    west.
  - (b) north and east step 1 lepton per tick and take 241 ticks, west takes 242. The south walker
    stops changing position within its first 5 ticks, a few leptons from its start, and does not
    arrive by its own steps.
  - (c) the north walker never arrives: it swings back and forth around its walking target. East
    takes 15 ticks, west 28 and south 7.
- *Falsified if:* (a) neither the south nor the west walker shows a sideways deviation (which would
  mean the arithmetic is not round-toward-zero at this point, or the table values differ from
  S-0105), north or east shows a deviation, a deviation exceeds its bound, or the number of ticks of
  any of the four walks differs from the value F3 and F7 give. Equal numbers of ticks north and south
  alone falsify nothing, since some speeds give them. (b) the south walker keeps changing position
  by its own steps after its first 5 ticks, or a tick count differs. A move that a rule of another
  spec starts later (for example a new path) does not count. (c) the north walker arrives within 600 ticks, or
  a tick count differs.

**Scenario W9: draw counts and their order (claims 3, 11, 12, 13, 15).**
- *Setup:* the state recorder of S-0202's Measurement scenarios (a *recorder* in the glossary's
  sense: it only reads), which gives, at the end of every tick, the state of the original's
  scenario random stream and its position. Purpose-made flat maps as in W3 and W7. The rules below
  apply wherever another scenario of this spec reads draws with this recorder (W1, parts d to f,
  and W7, parts m to o).
  - *Counting in draws.* The recorder shows raw values, not draws. From each recorded state the
    raw values of the next tick are replayed with S-0202's generator (claims 2 and 4), and the
    draws are counted from them: a draw over 0 to 3 takes exactly one raw value, while a draw over
    0 to 4 takes one or more, since S-0202 claim 4 rejects a raw value whose residue modulo 8
    (0 to 7) is 5, 6 or 7. A tick's expected draws are confirmed when the advance of the position
    on that tick equals the raw values that those draws, plus the background below, take from the
    replayed values.
  - *Background draws.* Other rules draw from the same stream (S-0202 claim 10), among them the
    computer player's decisions, the idle attempts of S-0107 claim 14, vehicle movement and the
    effects of a hit. The expected draws of a part are those that this spec's rules add to that
    background on the tick under test. The background is read from a reference: a variant of the
    same part, run the same way, in which only the item under test differs, compared on the
    corresponding tick (the asking tick, the tick of the hit or the tick of the request). The
    references are:
    - W9 (a): the walk of W3 from sub-cell 2 instead of sub-cell 0 (class right, so claim 3 takes
      no draw).
    - W9 (b): the same crusher drive with both soldiers one cell off its track, so that no request
      is sent.
    - W9 (c): the hit of W7 part (c), with the same attacker type and weapon at the same distance,
      where neither request moves the unit (one is not sent, the other refused).
    - W9 (d): the same part with the test file of W7 part (c), so that the hit sends no request
      that is accepted.
    - W9 (e): the same walk and hit as W7 part (b), with a named test file that removes the
      scatter property from the move behaviour, so that request B is refused, request A is not
      sent, and neither takes a draw.
    - W1 (d) to (f): the same part with the unit pre-placed at sub-cell 2 (class right, so neither
      claim 3 nor claim 4 takes a draw).
    - W7 (n1), (n2) and (n4): (n3) for the hit and, in part n4, the same crusher request to the unit
      undeployed and idle; the reference accepts its request and so takes one draw more.
    - W7 (o2), (o3) and (o4): (o1), whose occupants refuse every request while they play
      *deploy*; on the tick of the first request, (o2), (o3) and (o4) take three draws more than
      (o1).
    - W7 (m): the same part without the celebration order, whose three occupants accept the
      first request and so take three draws over 0 to 4; on the tick of the first request, (m)
      takes those three draws fewer than this reference.

    Since no seed can be fixed, a run and its reference start from different random states, and a
    draw whose range is not a power of two can take a different number of raw values in each. So a
    reference's advance is counted after its own expected draws (those that this spec's rules
    take in the reference, such as the accepted requests of (n3) and of the reference of (m)) are
    removed by replay, and a reference serves only when that remaining advance on the
    corresponding tick is the same in every reference run (ideally zero, as S-0202 scenario R5 (a) requires of its control interval); the setup (for
    example the attacker and its weapon) is chosen so that it is. That fixed advance is what the
    parts call *the hit's own draws* on the tick of a hit. When no setup gives a fixed reference
    advance, the part decides nothing. Starting a sequence takes no draw (S-0107 claim 6), so the
    *undeploy* that (o2) and (o3) start adds nothing to their count.

    Even so, a sporadic draw can fall on a compared tick of one run: the computer player, for
    example, decides on ticks that depend on the stream's values. So a disagreement between a run and its reference
    that the expected draws do not explain is first checked by repeating that run and its
    reference 5 times: it falsifies the part only when it recurs in the same way in at least 3 of
    them, and otherwise the run is discarded and replaced.
- *Procedure:* (a) the walk of W3, one run. (b) the two crusher requests of W7 part (a), the one
  accepted by the idle soldier and the one refused by the walking soldier, one run each. (c) the
  double hit of W7 part (j), variant j1, 40 counted runs. (d) No destination found. As W7 part (k), with these
  changes: each of the eight neighbours of the centre cell is of a land type that infantry cannot
  enter (a walker speed percentage of zero, claim 9) and holds no unit, so that 15 of the 16 outer
  cells of the block hold an idle infantry unit and the corner cell the vehicle; and the
  civilian's test file is that of W7 part (j), so that the hit sends it request A by its second
  case and request B, and claim 11 accepts both. No cell qualifies for the search (the outer
  cells of the block hold occupants, the neighbours cannot be entered, and the cells outside the
  block fail as in W7 part (k)), and no
  neighbour is enterable, so neither request finds a destination. 5 runs. (e) Request A not sent
  to a walking unit: the hit of W7 part (b), 40 runs, counted as there.
- *Observable:* per tick, the number of draws taken from the stream and their values, replayed
  from the state recorded at the end of the tick before.
- *Expected:*
  - (a) exactly one draw over 0 to 3 on the asking tick of each cell entered whose desired point
    has class centre (claim 3), and none on any other tick of the walk.
  - (b) one draw over 0 to 4 on the tick of the request the idle soldier accepts, and none on the
    tick of the request the walking soldier refuses.
  - (c) two draws over 0 to 4 on the tick of the hit, besides the hit's own draws. The destination
    held at the end of that tick is the one that claim 12 gives for the value of the second of
    them (request B), around the unit's own cell: the outcome set *H*₁ of W7 part (j). Request A takes the first of them, which does not decide its destination here:
    claim 13 takes that from the search, which finds a cell on this open ground.
  - (d) two draws over 0 to 4 on the tick of the hit, besides the hit's own draws, and the
    civilian takes no new destination and does not move on that tick.
  - (e) one draw over 0 to 4 on the tick of the hit, besides the hit's own draws: request B's.
    Request A is not sent, since the walking civilian's locomotor is moving and it has a
    destination (claim 15, second case).
- *Falsified if:* (a) a draw on another tick, or two on one asking tick. (b) no draw or two
  draws for the accepted request, or a draw for the refused one. (c) one or
  three draws besides the hit's own draws, or a final destination that matches what claim 12
  gives for the value of the first of the two draws, around the unit's own cell, but not for the
  value of the second, or a destination outside *H*₁. (d) fewer than two draws besides the hit's own draws (which would mean that a
  request that finds no destination takes no draw), or the civilian moves on the tick of the hit. (e) two draws besides the hit's own draws (which would
  mean that request A was sent to a walking unit and accepted), or none.

**Scenario W10: end of a walk and the approaching walker's stop (claims 16, 17).**
- *Setup:* a tiny flat map at level 0, clear terrain, game speed index 2. A named test file (a loose
  copy of the rules file, removed after the runs) gives one ordinary ground infantry type of the
  human player a weapon with range 5 cells, zero minimum range and no close-range, elevation,
  cliff or wall property, as S-0307 scenario RG10 does; the type is not open-topped. The recorder
  reads, at the end of every tick, the tick number, the attacker's world position, walking target,
  target, destination (its kind, cell or object, and whether it is empty) and infantry speed
  fraction (S-0310 Terms), and the tick on which each of its firing sequences starts. Every arrival
  tick (claim 6) is read from the walker's position and walking target. For each arrival tick, the
  recorder also evaluates S-0307's range test with the attacker's reference point of S-0307 claim 8
  and, as target point, the centre of the target's cell at that cell's terrain height: the result
  is called *in range at the cell centre*. The attacker starts ten cells along a map axis from the
  target, on a straight run of clear cells.
  - (a) The target is an unarmed enemy building of a one-cell foundation.
  - (b) The target is an idle unarmed enemy infantry unit standing at its cell's centre position.
    (b2) As (b), but the target stands at a side position (claim 1) on the attacker's side of its
    cell, and the start distance is adjusted so that, on one arrival tick of the approach, the
    target's own position is within range while its cell centre is not.
    (b3) As (b), with a move order to the cell next to the target instead of an attack order.
  - (c) The failed search of S-0307 scenario RG12: the infantry type's weapon range is 1 cell,
    the named test file also gives the type the hunter-seeker property, and the target is an
    unarmed enemy vehicle, so that the destination becomes the target object. This part does not
    separate the exception from the main rule, which can clear the destination first on an arrival
    tick in range (Open questions); it checks only the state once firing has begun.
  - (d) No target: a move order to an empty cell eight cells away.
- *Procedure:* (a) to (c): issue an attack order on the target; (b3) and (d): the move order. Record
  until the attacker has stood still for 30 ticks. One run per part, repeated on four map axes.
- *Observable:* the tick and the standing position at which the walk ends (the first tick after
  which the position stays the same), whether the destination is empty from that tick on, the speed
  fraction on that tick, the in-range result of every arrival tick, and the first tick of a firing
  sequence.
- *Expected:*
  - (a) the walk ends on the arrival tick in the approach cell that S-0307 gave as destination (as
    recorded), with the destination empty from that tick on; no firing sequence starts before that
    arrival tick, whatever the in-range result of earlier arrival ticks.
  - (b) the walk ends on the first arrival tick that is in range at the cell centre, which comes
    before the walker reaches its recorded destination cell; the destination is empty from that
    tick on, the target is kept, the speed fraction is 0, and no firing sequence starts before it.
    (b2) the walker does not stop on the arrival tick on which only the target's own position is in
    range; it stops on the first one that is in range at the cell centre. (b3) the walker walks on
    to the cell it was ordered to.
  - (c) the destination is the target object until the walker stops; from the tick on which its
    first firing sequence starts, its destination is empty and it does not move.
  - (d) the walk ends on the arrival tick on the standing position in the ordered cell: destination
    empty and speed fraction 0 from that tick on.
- *Falsified if:* (a) the walker stops before the recorded approach cell, or fires before arriving
  there. (b) the walker walks on past an arrival tick that is in range at the cell centre, stops on
  an arrival tick that is not, stops between two arrival ticks, loses its target, or fires while
  walking. (b2) it stops on the arrival tick on which only the target's own position is in range.
  (b3) it stops early. (c) it keeps a destination, or moves, after its first firing sequence starts. (d) it keeps a
  destination or a non-zero speed fraction after the arrival tick in the ordered cell, or the walk
  ends on another tick. A run in which S-0215 claim 37 renews the destination during the approach
  still counts; the parts test only when the walker stops.

## Open questions

- The screen-cover check (Terms) is exact as arithmetic, but three parts of it are not confirmed:
  the property of *C* that, in the original, lets a high-bridge cell count 4 levels higher was not
  identified, and our engine raises such a cell whenever it is a high-bridge cell (Terms); a cell
  outside the map is taken to read as height level 0, which matters only on a map with negative
  levels; and that the check matches what a viewer sees hidden on the screen is an interpretation.
  Scenario W7(f) tests the rule on uneven ground; W7(b) relies only on flat ground, where every cell
  passes.
- Which situations of an infantry unit set the tether link (Terms) was not determined, and no situation
  on the map in which an infantry unit holds one is known. The link only removes the second case of
  request A (claim 15), so it matters only for an infantry unit hurt while it holds one.
- While a unit travels through a tunnel (S-0212 Terms), its scatter centre cell appears to be the
  tunnel's exit cell (Terms *scatter reference point*). Tunnels belong to S-0212 and the movement specs, and this was not checked
  further.
- Infantry of a jumpjet type are left out of claims 16 and 17: their movement belongs to the
  jumpjet locomotor, whose arrival and stop rules were not traced. The same in-range stop is a rule of
  every mobile actor in the original, not only of infantry; for vehicles and aircraft it belongs to
  their movement specs (S-0207 for ground vehicles), and whether their locomotors apply it on their
  own arrival processing was not checked here.
- The exception of claim 17 (the target is the destination) was found by reading only. When it
  triggers before the main rule does, and whether a standing walker with that destination ever
  reaches it, were not traced; scenario W10 part c checks only the end state.
- The height condition of claim 16 (two height levels) has no scenario: it matters only where an
  arrived position and the locomotor destination share a cell at different heights (a high-bridge
  deck over its ground cell), and how a walker's locomotor destination is placed there was not
  traced.
- No other factor (for example a game-wide speed bias of the movement specs) is known to scale a
  walking infantry unit's speed during a step, and F5 has none; that none exists is not confirmed.
- The height of a walker between standing positions, while it steps across a slope or onto a bridge
  deck, was not determined. The step of claim 6 changes only the map position, and no rule was
  found that sets the height to S-0106's surface height on every step, so the original's walker may
  keep the height of the position it left until it arrives, which is what our engine does (claim 6).
  Scenario W2(e) records the height on every tick of its walks, without an expected value; a
  measured value replaces our engine's choice.
- The moment at which a walker takes a crate lying in the cell it walks into belongs to the crate
  specs (a pointer only). It appears to happen on the tick on which the walker chooses its walking target
  there, before it arrives, but this spec makes no claim about it and it was not confirmed. W4(c)
  needs only that the crate is taken during the walk.
