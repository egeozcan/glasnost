---
id: S-0208
title: Infantry movement (standing positions, walking step and speed, crawling, scatter destinations)
topic: infantry
revision: 53
sources-class: [decomp, measurement]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-07
measurements: [M-0023]
requests: [Q-0017, Q-0072, Q-0201, Q-0181, Q-0235, Q-0311, Q-0268, Q-0295, Q-1502, Q-2657, Q-2701, Q-2702, Q-2746]
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
points to S-0209 and S-0209 decides. This spec owns the infantry scatter from damage (claims 11 and 15), and the landing-point
choice for infantry using the teleporting locomotor (claims 19 and 20). S-0207 claim 20 already
answers Q-0181's jump timing, phasing-in and successive-jump questions; its part c left the
infantry landing point open. Claims 19 and 20 answer that part. They do not cover a superweapon
teleport. Q-0235 question 3 was already answered by S-0207 claim 20 e's corrected
bridge-deck landing rule; claim 19 here makes the position-choice record, reservation
record and refused replacement explicit for all four questions. S-0417 claim 10 supplies a concrete infantry factory-release tether situation.
Claim 23 answers Q-0295's walking cell-arrival and clear-report timing. Claims 6 and 22
already establish the standing-position arrival tick; claim 23 connects it to that report.
S-0417 owns the release, contact exchange and consequences of the report; S-0409 owns sight
refreshes, not the definition of a walking arrival.
For an infantry unit that approaches its target, S-0215 claim 37 owns when the approach step runs
and whether it keeps, renews or takes a destination, S-0307 owns the approach position and the
outcomes of a failed search, including claim 22's close-range searches. This spec owns the
ordinary walking outcome of its near-target shortcut (claim 21 below), and S-0310 claim 8 owns
the firing check's *moving* answer. This spec
answers Q-0072 question 3 for ordinary walking infantry: when the walker stops and what happens
to its destination (claims 16 and 17, scenario W10). The approach step keeping a destination
does not exempt it from these arrival rules. Infantry of a
jumpjet type move by the jumpjet locomotor, which belongs to other movement specs; claims 16 and
17 do not cover them. Q-0201's arrival test was already given by
claim 16; claim 18 below supplies the previously missing cell-destination point and height,
and the consequence of arriving underneath its deck. S-0212 claim 7 already specifies that
a successful path to a high-bridge cell ends on its deck; claim 23 already owns later search
failure. Neither supplied the locomotor destination's numeric height. Skirmish only; network games and
campaigns are out of scope.

Citations refer to the approved specs and their current correction-round drafts, including
S-0417 (factory unloading). The other cited specs are: S-0003 (INI value forms), S-0006 (rules
layering), S-0009 (map file semantics), S-0011 (map triggers), S-0105 (voxel rendering, for its table trigonometry and
table square root), S-0106 (slope shapes and terrain surface height), S-0107 (infantry sprite
sequences, which points here for damage scatter), S-0201 (game
speed and tick rate), S-0202 (the random streams), S-0204 (coordinates, distance and facing),
S-0205 (number domains), S-0207 (ground vehicle movement), S-0209 (cell occupancy and crowding),
S-0210 (crushing), S-0211 (land types and passability), S-0212 (pathfinding), S-0215 (behaviours)
and S-0311 (veterancy), and for claims 16 and 17 S-0307 (weapon range test), S-0308 (weapon
choice) and S-0310 (turrets and facing for fire, for the firing check's answers and the infantry
speed fraction).

**Cross-spec confirmation at hand-off.** Claim 19's infantry teleport fallback parameters
agree with S-0212 claim 25's corresponding search-use row. Retain both texts together,
including the distinct search deck mode and standing-position deck mode. The coordinated
blocked-first-check rule belongs to S-0207 claim 15 and S-0212 claim 22, and failed-approach
searches, including close-range failure, belong to S-0307 claim 17 and S-0212 claim 25.
Neither changes this infantry teleport fallback. Claim 19 now answers Q-0311's
post-jump infantry record lifecycle; it must be handed off with the matching S-0209
occupancy pointer. Claim 22 answers Q-0268's ordinary walking arrival timing. Its
teleport timing was already answered by S-0207 claim 22 in this round; destruction
and death-sequence tag retention remain S-0312's answers, not movement rules.
At sign-off, confirm claims 19 and 22 against the final paired texts of S-0207,
S-0209, S-0212 and S-0215. Retain the fixation-tick fallback selection, not a
selection at the later jump. Retain the distinction between infantry's discarded
remembered point and absent new deck standing-position mark, and vehicles'
remembered-point clearing and additional deck placement mark (S-0207 claim 20 a/e,
S-0209 claim 1). The shared `quirks.teleport_vehicle_ground_reservation` governs
vehicles, not infantry. Jump-time target loss follows S-0207 claim 20 e, including
its exception for an attacker whose strike target is the living jumping object,
even after a normal strike end (S-0519 claims 18 and 20), not an erasing lock. Override reversion and preservation of a different restored target
remain S-0215 claim 10's rules for the jump, not new infantry landing-choice rules.

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
  position in the cell it is entering (claims 3 and 4), or a position in its own cell selected
  by claim 21.
- **Step vector.** For a step direction (glossary *step direction*: 0 map north, 2 map east, 4 map
  south, 6 map west), the one-cell step in direction *d* moves a point by 256 leptons along each map
  axis that the direction uses, for example (+256, −256) for direction 1.
- **Speed field.** The type's imported speed value (Import mapping), an integer.
- **Speed multiplier.** A per-unit real number, 1.0 when the unit is created. A speed crate
  changes it under S-0520 claim 20, which owns eligibility and the bonus value.
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
  destination, the centre and height defined by claim 18). A unit with no movement in progress has none. It differs from the
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
- **Scatter centre cell.** The cell around which a scattering unit looks for a destination (claims
  12 and 13): the cell of its scatter reference point. So it is the cell of the standing position a
  walker is heading for at that moment (the next cell of its path, or its own cell when it walks to
  another position in the cell it is in), and the unit's own cell when it has no walking target:
  when it stands still with no walking target, and also on a walker's arrival tick and on its asking
  tick until it has chosen its walking target there (claim 6). A walking target chosen at once at a
  scatter (claim 6) counts from that moment, so a later request in the same tick looks around its
  cell (claim 15). It is never the cell of a farther destination.
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
- **Tether link.** A link that two objects share during a docking, loading or unloading manoeuvre
  when that exchange creates one. Both hold it from the moment the exchange's rule establishes
  it—when one begins loading or unloading, gives the other clearance to unload, or docks at a
  refinery—and both lose it when that manoeuvre ends or the link is broken off. A unit holding one is not sent the second case of
  request A (claim 15). A successful infantry factory-release contact exchange gives the released
  unit one until its next cell-arrival update (S-0417 claim 10). Whether that interval can isolate
  the damage-scatter condition remains open (scenario W12).

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
   cell's occupancy record for the surface selected by this height test: let *z* be the walker's
   current integer world height and *F* the terrain surface height at its desired point, both in
   leptons. The selected surface is bridge deck exactly when the asked cell carries a high bridge
   and *z* > *F* + 312 leptons, and ground otherwise. This is exact integer arithmetic, with no
   rounding; equality selects ground. This choice is independent of the walker's current cell
   layer and of the entry check's record (S-0209 claim 15). A side position that a terrain object in the cell marks (S-0209 claim 1) counts as
   taken like one an infantry unit holds, so a walker never gets it. The *vehicle-present* and
   *object-present* flags are those of S-0209 Terms. What a walker that gets no position does belongs
   to S-0209.

   | Occupancy record of the cell being entered | Class *c* | Position the walker gets | Draws from the scenario random stream |
   |---|---|---|---|
   | the record for the selected surface has its vehicle-present flag on, or both the cell's ground record has its object-present flag on and no open gate (Terms) stands in the cell | any | none | none |
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

   **Other uses of the position choice.** S-0307 claim 22(b) and (c) use this table with
   the target's reference point as the desired point, not the walker's shifted position. Both
   read the chosen cell's ground occupancy record on any surface. For the shortcut in (b),
   that cell is the attacker's own cell and its own entry still counts as taken. These choices
   supply approach points, not reservations by themselves. Claim 21 below owns the walking
   consequence of a successful shortcut.

   Infantry jumpjet destination adjustment also uses this occupant-aware choice. S-0502
   claim 23 owns its desired point, occupancy surface and ranged draw, including refusal
   and failure cases; its backing and confidence are those of that claim. This does not
   use the ordinary walker's surface-selection test above.

   Transport exits also use this occupant-aware choice:
   - S-0507 claim 10 uses the serving neighbour's centre as the desired point for an infantry
     passenger leaving a vehicle transport. Its centre-class choice takes one ranged draw over
     0 to 3 from the scenario random stream, with table O's right/left/bottom preferences.
     Placement keeps that chosen position without another occupancy choice or position draw.
   - S-0507 claim 13 uses this choice for an infantry passenger only when the aircraft's
     integer world height equals the terrain surface height at its exact horizontal point.
     The desired point is the aircraft's exact world position, not its cell centre. Only the
     aircraft itself is excluded from occupancy; all other occupants still count. A centre-class
     desired point uses the same randomized preferences; a side-class point uses its non-random
     row. Successful placement keeps the chosen horizontal position but retains terrain height
     at the aircraft's original horizontal point, rather than recomputing it at the chosen point.
     This is not S-0507 claim 10's neighbouring-cell choice. Above terrain surface height,
     release instead preserves the aircraft's exact horizontal position and world height,
     without any standing-position choice or standing-position draw. These height comparisons
     use integer leptons without rounding.

   S-0507 claims 10 and 13 own these release placements and their failure outcomes; their
   backing is `measurable` (S-0507 scenarios T13 and T15), with medium confidence for vehicle-transport
   placement and high confidence for the air-release height distinction.
   They do not create a walking-target reservation merely by choosing a point.

   For an ordinary walker, the chosen position is reserved at once: from the tick on which the walker chooses it, the
   occupancy record of the cell being entered shows that position as taken, and the walker's
   previous mark is cleared using the former walking target's position and height, or its actual
   position and height when it had no walking target (S-0209 claims 1 and 15). The new position
   and infantry house are marked on the selected surface. Choosing it does not change the
   walker's listing layer, and crossing a cell boundary does not transfer the reservation.
   S-0209 claim 15 owns boundary-time layer changes, including crossings of a third cell on the
   way to a diagonal target. An ordinary walker therefore
   holds exactly one entry in the occupancy record at any time: its walking target while it has one,
   otherwise the position it stands on. One exception is the top position (claim 2, Edge
   cases): a unit at the top position and a unit at the centre of the same cell share the centre's
   entry, and when either of them leaves, the record shows the centre free although the other still
   stands there.
   - Backing: `measurable` (scenarios W1, W2 part c, W3 and W9; the reservation through S-0209 scenario
     C1, and the surface selection and boundary-time layer changes through S-0209 scenario C22,
     parts e and f; the vehicle-present row through S-0209 scenario C2, the building-present flag through S-0209
     scenario C10, the terrain marks through S-0209 scenario C16). `unmeasured` for the
     object-present flag and its open-gate exception, for the reason S-0209 claim 1 gives: every
     known object that sets the flag gets an entry outcome of its own, so its effect here,
     and thus the exception to it, has no known isolated measurement trigger
   - Confidence: high for the class rule, table O, the draw, the strict surface-selection test and
     the reservation; medium for the
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

   This enslaved-unit exception is for walking back to the master, not emergence from inside it.
   S-0402 claim 21 owns emergence: it uses claim 3's occupant-aware centre-class choice and
   starts each successfully placed slave facing map north, independently of the master's facing.
   Its placement and draw rules remain separate from this claim's occupant-ignoring return.
   The emergence statement carries S-0402 claim 21's backing and confidence (scenario H21).
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
   is the *asking tick* of S-0209. It takes its first step towards that target on the next tick. A
   walking target chosen at once at a scatter, outside the walker's own locomotion, is the exception
   given in *Destinations given during a unit's update* below. On an uninterrupted walk the asking
   tick is the tick right after the arrival tick, and on the first cell of a walk it is the tick
   before the first step.

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
   - Backing: `measurable` (scenarios W4 and W8). `unmeasured` for the provisional
     between-position height rule: W2(e) collects heights but supplies no confirmed prediction
     for the original; keeping the departure height is our engine's interim design choice
   - Confidence: high for the 17-lepton arrival rule and the truncation, and that a walker does not
     move on the tick on which it chooses a walking target in its own locomotion; medium for the
     choice falling on the tick after the arrival tick, which assumes that a walker's movement is
     processed once per tick (scenario W4 decides between ⌈240 / *s*⌉ + 2 and ⌈240 / *s*⌉ + 1 ticks
     per cell); medium for the exact direction arithmetic (F3, F7)

   **Destinations given during a unit's update (Q-2657).** A walker's locomotion is processed once
   in each of its updates, after its behaviour processing (S-0203, from S-0516 claim 42: ordinary
   mobile locomotion follows due behaviour processing). A walker standing with no walking target and
   no stored path first searches for a new destination (S-0212 claim 21) in the first locomotion
   after the destination is given. So a destination given before the update's locomotion, by an
   order that takes effect at the start of the tick or by the update's behaviour processing, is
   searched for in that same update, which is the asking tick of the walk's first cell. A
   destination given after it, such as by the end-of-update check of S-0215 claim 22, is searched
   for in the next tick, which is then the asking tick. For a walker standing with no walking target
   and no stored path, a scatter that takes its destination from the nearby-cell search (claim 13)
   is an exception: at the scatter the walker has one more locomotion at once, in which it searches
   for that destination and, when the search gives a path, chooses its walking target without
   moving. That choice is the walk's first position choice (claim 3), so it takes claim 3's draw
   when its desired point has class centre, after the scatter's own draw and before any later
   request's. A later request in the same tick finds that walking target (claim 15). When that
   scatter comes before the walker's own locomotion in the tick, as at a hit in another actor's
   update earlier in the tick (claim 15), that locomotion finds the walking target already chosen
   and takes the first step towards it, so the walker moves on the tick of the scatter. When the
   scatter comes after it, the first step comes in the next tick. A walker standing with no walking
   target that keeps a stored path at the scatter (*A stored path kept under enter* below) has the
   extra locomotion too, in the original and in our engine (where with
   `quirks.enter_keeps_stored_path` off no walker keeps its path at a scatter), and in it chooses
   its walking target in the path's next cell (S-0209 claim 6) without a search (medium, unmeasured:
   the scatter starts the extra locomotion whatever the walker's state, and such a walker accepts a
   scatter only by claim 11's third case, which no setup here arranges). A walker that already has a
   walking target is not given this extra locomotion in our engine (Open questions). A scatter whose
   destination is chosen among the eight neighbours, by claim 12 or by claim 13's fall-back when the
   search gives no cell, has none, so the walker has no walking target until its next locomotion.
   What decides is which choice gives the destination, not where the cell lies: the nearby-cell
   search can give a cell next to the walker's own, and a scatter that takes it has the extra
   locomotion. In the original, the factory's scatter request of S-0417 claim 17 is acted on in the
   released walker's own update, after that update's behaviour processing and locomotion. Our
   engine's order of S-0417 claim 17 also puts it after the locomotion, so what follows holds in
   both. So the tick of that request is the asking tick and the first step is in the next tick when
   the destination comes from the nearby-cell search, while for a destination chosen among the eight
   neighbours the next tick is the asking tick and the first step comes one tick later.

   **A new destination while a step is under way (Q-2657).** A walker that has a walking target
   keeps it when it is given a new destination, by an order or by any other rule. It finishes the
   step under way and arrives there by this claim, whatever the new destination is, and its stored
   path is dropped, except where the paragraph on *enter* below keeps it. What follows in this
   paragraph is for a walker whose path is dropped. Its destination reads the new cell from then on,
   but the new destination is first searched for at its first locomotion with no walking target, the
   tick after that arrival tick. If that search fails at the reachability
   gate, the destination is cleared there (S-0212 claim 23) and the walker stops on the standing
   position it arrived on. Until then no gate test is made, so no gate failure can clear the
   destination earlier. A walker whose current behaviour is *move* then falls back to rest at the
   end of that update (S-0215 claim 22). If the standing position it arrives on is in the new
   destination's cell, claim 16 decides whether its walk ends there instead.
   - Backing for the two paragraphs above: `measured` (M-0023 baseline and run 1: the first step 1
     tick after the turn of the factory's scatter request. M-0023 controls: the baseline's walk is a
     single step to a standing position in a cell next to the release cell, and an order to a cell
     in another reach region, given at the start of the tick in which that walk first changes
     position, leaves the positions those of the baseline's walk, tick for tick. The unit arrives on
     the baseline's standing position in the baseline's arrival tick with its destination still
     reading the ordered cell, and the destination is cleared in the next tick, with *move* still
     current and *guard* queued) for those cases. The original's place of the factory's request
     after the locomotion is `measurable` by W17 part b: a destination chosen among the eight
     neighbours has no extra locomotion, so a request before the walker's locomotion would give the
     first step 1 tick after the request's turn, and a request after it gives the first step 2 ticks
     after. Given that place, M-0023's first step 1 tick after the request's turn needs the extra
     locomotion, which only a destination from the nearby-cell search has. So M-0023 shows that the
     baseline's destination came from the nearby-cell search, which the record does not say by
     itself, only together with W17 part b. The first step on the tick of a scatter that comes
     before the walker's own locomotion is `unmeasured`: no scenario here sets which of two actors
     updates first in a tick, so none can place a hit before the walker's update. `measurable`
     (scenario W17: part a for an order given before the update's locomotion, part b for the
     neighbour choice, parts c and d for a walker on a longer stored path, part e for the arrival in
     the new destination's cell; W7 part j for the walking target that a later request finds, W9
     part c for the draw of its choice). `unmeasured` for a destination given by the update's
     behaviour processing: no scenario here can time such a destination apart from the order of part
     a, and it follows from the same place of the locomotion that part a tests. The end-of-update
     case is `unmeasured` here: S-0215 claim 22 owns it, and no scenario of this spec gives such a
     destination.
   - Confidence for the two paragraphs above: high for the first step 1 tick after the turn of the
     factory's scatter request in M-0023's set-up (S-0417 scenario U10 part b), and, for an order to
     a cell in another reach region given at the start of the tick in which a single step first
     changes position, for the step under way finished with the walking target kept and the
     destination cleared in the tick after the arrival tick (M-0023 controls). Medium for that
     request's destination coming from the nearby-cell search and for the extra locomotion of such a
     scatter, for the place of the update's locomotion, for the original's place of the factory's
     request in the walker's update (after the locomotion), for the first step on the tick of a
     scatter that comes before the walker's own locomotion, for the neighbour choice and the
     end-of-update case being searched one tick later, for the walking target kept and the step
     under way finished in other cases, for the stored path being dropped, for the walking target
     that a later request in the same tick finds and the draw its choice takes, for the search in
     the tick after the arrival tick and for the arrival in the new destination's cell.

   **A stored path kept under *enter* (Q-2701).** A walker given a new destination, by an order or
   by any other rule, keeps its stored path instead of dropping it when, at that moment, its
   current or queued behaviour is *enter* and it is in contact with nothing (glossary *contact*).
   Whether it has a walking target plays no part, so a walker standing between steps with a stored
   path keeps it too. With
   `quirks.enter_keeps_stored_path` off, the path is dropped in this case as in every other (Edge
   cases). An ordinary behaviour order breaks off the actor's contact, queues the order's
   behaviour and then assigns its destination (S-0213 table T3, parts b, d and f, in that order).
   So:
   - An enter order (into a transport, for example) given to a walker under another behaviour,
     such as *move*, keeps its path: *enter* is queued when the destination is assigned.
   - An order whose behaviour is *capture* is not an enter order for this rule (Q-2746). This covers
     the order to garrison a building (S-0214 row I8, S-0508 claim 2), and the capture, infiltrate
     and hijack orders. The walker sent to garrison carries *capture*, current or queued. So a
     walker under *move* that is ordered to garrison a building drops its path. It
     finishes the step under way and then heads for the building, as the paragraph above gives for a
     dropped path. A walker under *capture* that is ordered to a free cell drops its path too, and
     after the step under way it heads for the new cell, not for the building.
   - Any behaviour order given to a walker under *enter* keeps its path: *enter* is still current
     at the assignment.
   - Both hold for a walker that is not held by a tether link, since the order has broken off its
     contact by then. A walker held by a tether link keeps its link at the order (part b), so it
     drops its path, unless the link is with a building of the unloading kind, which the order
     breaks off too.
   - A destination given by a rule that is not an order, such as an accepted scatter (claims 12
     and 13), breaks off no contact. A walker under *enter* keeps its path then only when it is in
     contact with nothing, so one in contact with what it is entering drops it.

   A walker that keeps its path goes on following it: at each locomotion with no walking target it
   chooses its walking target in the path's next cell (S-0209 claim 6), and it makes no search for
   the new destination while stored steps remain. A search replaces the path when a step on it is
   blocked (S-0209 claims 7 and 10) and when no stored steps are left (the fresh search of S-0212
   claim 20). Both of these searches are for the new destination. So a walker given an enter order
   while it walks goes on along its old route to the end of its stored path (S-0212 claim 20 gives
   how many steps a path holds), unless it arrives earlier by claim 16 in the cell of its locomotor
   destination, before it first heads for what it enters. A walker under *enter* that is ordered
   elsewhere keeps its path towards what it was entering in the same way, and its next steps follow
   it. While the walker follows its old path under *enter*, its processings of *enter* ask what it
   enters to load it (S-0507 claim 5). A vehicle transport at rest accepts a requester that is
   moving, which keeps the walker in *enter*, but it does not call to its cell a requester that is
   moving towards a destination, so it does not call such a walker. For the walker ordered away, the
   order has broken off its contact, and what its processings of *enter* do until the order's
   behaviour becomes current belongs to S-0507 claim 5 and S-0215. A walker follows a stored path
   only towards a destination, so a destination that becomes nothing, as at a stop order (S-0213
   claim 8), leaves it no path to follow, whatever it keeps.
   - Backing: `measurable` (scenario W17: part f for an enter order to a walker under *move*, part g
     for a move order to a walker under *enter*, part h for an enter order to a walker standing
     between steps with a stored path, part j for a garrison order to a walker under *move*, part k
     for a move order to a walker under *capture*). `unmeasured` for the walker held by a tether
     link and for the scatter to a walker under *enter*. Which exchanges tether a walker under
     *enter*, and when, belong to S-0507 and S-0213 (Terms), and no setup here times an order within
     one. A walker under *enter* that follows a path has a destination, so its locomotor is moving,
     and it accepts a scatter only by claim 11's third case, which no setup here arranges.
   - Confidence: high for the behaviours read at the assignment and for the condition's not
     depending on a walking target. High for a garrison order, and every other order whose behaviour
     is *capture*, not keeping the path of a walker under *move*, and for its consequences in
     parts j and k. Medium for its
     contact part: every order here breaks off the contact before the assignment, so no scenario
     here has a walker in contact then, and the scatter to a walker under *enter* is `unmeasured`.
     Medium for the consequences: the old path followed to its end or to a blocked step, and the
     cases of the bullets. Medium for the transport's acceptance and its not calling such a walker:
     S-0507 claim 5 rates its replies high, and that the walker's state meets their conditions here
     was not traced further. Low for how far the walker ordered away from what it was entering
     follows the old path, which depends on its processings of *enter* before the order's behaviour
     becomes current.

   **Requests sent from an extra locomotion (Q-2702).** The extra locomotion that a scatter gives
   (*Destinations given during a unit's update* above) is a whole locomotion of the walker, the same
   as the one of its update. After its search it makes the asks of S-0209 claims 6 to 10 about the
   next cell of its path, with their reactions on that asking tick: claim 7's immediate new path,
   claim 8's give-up or its request to make room, claim 9's gate and obstacles, and claim 10's
   moving-blocker wait. A request of claim 8 sent there reaches the objects listed in that cell at
   once (S-0209 claim 11). An occupant that accepts it with a destination from the nearby-cell
   search has an extra locomotion of its own, inside the first one. So such requests can pass from
   cell to cell within one tick. When the extra locomotion comes before the walker's own locomotion
   in the tick and leaves it with no walking target, that locomotion asks again on the same tick,
   and the ask is a first reaction (S-0209 claim 7). A walker standing with no walking target that
   keeps its stored path at such a scatter (the paragraph above) has the extra locomotion too, as
   claim 6 gives above, and in it follows that path. One that keeps its path and already has a
   walking target has none in our engine (Open questions).
   - *A unit asked again while a locomotion of its own is under way.* A request reaches such a unit
     only through something that its own locomotion set off, such as its request to make room, so
     the unit has a destination then and its locomotor is moving. By claim 11 (third case) only a
     fraidycat whose current behaviour has the scatter property, and that has scatter permission,
     can accept a request then. In the original such a unit has a second locomotion inside the
     first. Nothing bounds that nesting: it ends only when no ask sends a further request, for
     example when the occupants that accepted earlier requests now have movement in progress, so
     that the cell they stand in gives *moving blocker* (S-0209 claim 4, row WR5) instead of
     *stationary friendly*. Where it never ends, the original crashes.
   - **In our engine** (a departure, no switch): no unit is given an extra locomotion while a
     locomotion of its own, that of its update or an extra one, is under way. The request is
     otherwise accepted as claims 11 to 13 give, with its draw and its destination, and the stored
     path is dropped or kept as above. The locomotion under way finishes as it would have without
     the request (after S-0209 claim 8's request, it makes no further ask). The unit's next
     locomotion is the first to act on the new destination. So each unit has at most one locomotion
     under way at a time. The ground (glossary *departure*) is a design choice of scope: telling a
     nesting that ends from one that never does would need a depth limit that no game rule gives.
     The bound also rules out the original's fault on legal data, should a legal set-up exist in
     which the nesting never ends (Open questions). The choice changes only a fraidycat asked again
     within a crowd's chain: its walking target, the start of its moving-blocker wait or a request
     it sends comes at its next locomotion instead of at once.
   - Backing: `unmeasured` for the asks of the extra locomotion and the chain: in the set-ups here
     the scatter comes before the walker's own locomotion in the same tick, which would make the
     same ask in that tick, so a request sent from either locomotion falls in the tick of the
     scatter and no record tells them apart. Scenario W17, part i, and scenario W9, part f, check
     only that the request and the draws of the occupants that accept fall in that tick.
     `unmeasured` for the original's nesting and its end: a setup needs a fraidycat in a behaviour
     with the scatter property listed in a cell that a request of the chain reaches again, with its
     occupants still giving *stationary friendly* then, and none is known that arranges that
     reliably. `unmeasured` for the walker's own locomotion asking again on the tick of an earlier
     extra one: in the set-ups here that second ask only starts a moving-blocker wait one tick
     sooner, which no record shows. Our engine's bound is a rule of our engine and needs no backing.
   - Confidence: medium that the extra locomotion is the whole locomotion, with every ask and
     reaction of S-0209 claims 6 to 10, S-0209 claim 8's request to make room among them. Medium for
     the original having no bound of its own on the nesting and crashing where the nesting never
     ends, for the chain within one tick, for the walker's own locomotion asking again on the same
     tick, for only such a fraidycat being asked again with success, as claim 11 rates its cases,
     and for the nesting's end. Low for whether a legal setup exists in which the nesting never ends
     (Open questions).

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

    No built-in behaviour starts *deployed-idle* (S-0107 claim 20), so its rows matter only where a
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
      - the *deployed-idle* rows: no built-in behaviour starts *deployed-idle* (S-0107 claim 20), so
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
    passes the screen-cover check at its own height level and is not a high-bridge cell. The high-bridge raise within that check is our engine's provisional choice; the corresponding condition in the original remains unidentified (Terms, *screen-cover check*). When no qualifying candidate is both,
    its destination is the first qualifying candidate, whatever the check gives and whether or not it
    is a high-bridge cell. The unit then attempts to move there as under a move order, subject
    to normal pathfinding (claim 24). When no candidate qualifies, it does not move.
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
    gives a cell, the unit attempts to move there as under a move order, subject to normal
    pathfinding (claim 24). Which cells qualify (among
    them the rule that a cell's whole occupancy record must be empty), the double count of the
    centre, the movement zone and the choice among the preferred cells by the tick number of the
    request are S-0212's. This spec adds only how a scatter uses the result. The direction plays no
    part in that search, so the destination does not depend on the draw.

    Consequences, from S-0212 claim 25 and S-0209 claim 1: the search does not give the cell the
    unit stands in while the unit's own standing position is marked there, but it can give the
    scatter centre cell itself when that is a walker's next cell and its record is empty. The
    unit's own position is normally marked, with two exceptions here: the top position
    recorded as the centre (Edge cases), and claim 19's infantry teleport from a
    ground-height chosen point onto a deck without a new deck mark. In the first case,
    a unit left at the top or centre after its cell-mate at the other position has gone
    is not marked. In either case what the search gives for its own cell is S-0212's
    (claim 25), including its candidate-layer selection, not a blanket exclusion of
    the unit's current cell. On open flat
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
    and, when its owner is human-controlled (in our engine, also when it is an AI house), its
    current sequence is not one of the deploy kinds. When a hit sends both requests and both are
    accepted, the two requests take their two draws on the tick of the hit, request A's first and
    request B's second, and claim 3 can add one between them (below). When request B finds a
    destination, the unit heads for it at the end of that tick: claim 12 gives it from request B's
    draw, around request B's scatter centre cell, given next. Whenever request B finds a
    destination, it replaces request A's, so request A's draw shows nothing at the end of the
    tick; on open ground it does not even decide request A's own destination, which claim 13 takes
    from the search (scenarios W7, part j, and W9, part c). When request A's destination comes
    from the nearby-cell search, that search gives it a path and the unit has no walking target,
    it chooses its walking target towards that destination at once (claim 6, *Destinations given
    during a unit's update*), and that walking target is its scatter reference point (Terms) when
    request B arrives, so request B's scatter centre cell is the cell of that walking target. A
    unit that is still finishing a step when it is hit (request A's first case asks for no target
    and no destination, not for no walking target) keeps its walking target and, in our engine,
    chooses none at once (claim 6; the original's extra locomotion there is an Open question), so
    request B's scatter centre cell is the cell of the walking target it has. An at-once choice
    takes claim 3's draw between request A's draw and request B's when the unit stands on a
    position of class centre, so three draws are taken on the tick of the hit then. When a unit
    with no walking target has request A fall back to one of the eight neighbours (claim 13), or
    request A's search gives it no path (claim 24), it has no walking target when request B
    arrives, and request B's scatter centre cell is the unit's own cell. On open ground request
    A's destination is a neighbour of the unit's cell, so the cell of a walking target chosen at
    once is request A's destination cell. The unit keeps that walking target (claim 6), so it
    first steps into that cell and then heads for request B's destination. Consequences with the
    player-scatter rule off (its absent-key default, Import mapping) and without the scatter
    veterancy ability:
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
      of the second case: factory release supplies a tether situation (S-0417 claim 10), but no
      reliable setup yet isolates this condition while the unit also has no target, destination or
      movement in progress. Scenario W12 is exploratory, not backing for that condition
    - Confidence: medium for the two cases of request A, for request B and for the scatter centre
      cell of request B when both requests are accepted; the factory-release tether situation has
      S-0417 claim 10's confidence

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
    from then on (Terms *locomotor moving*). Otherwise the destination is kept: the walk
    continues by claim 6 while path steps remain, and path exhaustion is handled by claim 18. This claim
    never ends a walk between two arrival ticks; a rule that stops the unit there (claim 17's
    exception) does so itself.
    - Backing: `measurable` (scenario W10, parts a, d and e)
    - Confidence: high for the arrival and strict height tests; medium for subsequent path
      exhaustion, specified in claim 18

17. **An approaching walker's stop and destination outcome (Q-0072, question 3).** On each
    arrival tick, before claim 16's test, an infantry unit loses its destination when all of these hold:
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
    This is an arrival rule, not a processing of the approach step: it does not wait for the next
    behaviour processing of S-0215 claim 37. That step's retention of a destination while the
    target is in range does not prevent this stop.
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
    its firing check (S-0301, S-0310) answers *can fire*, the unit loses its destination and any movement still
    in progress ends on the tick on which it starts its firing sequence. By S-0310 claim 8 the check
    answers *moving* while the speed fraction is above 0.1, so this happens only to a walker that is
    already standing (for example blocked, S-0209), never in the middle of a step.
    S-0307 claim 22(b)'s close-range shortcut also requests movement within the walker's
    own cell while assigning the target's cell as destination. Claim 21 below owns its
    second position choice and walking outcome; the immediate walking target need not be
    the shortcut's first chosen point on a bridge deck.
    - Backing: `measurable` (scenario W10, parts a, b, b2 and b3 for the attack-versus-move
      distinction and the cell-centre range point; part c checks only the exception's end state).
      `unmeasured` for the exception's exact clearing tick and its restriction to an already
      standing walker: W10(c) does not isolate it from an arrival stop, and no fixture here
      reaches it independently. Also `unmeasured` for the open-topped range variant, the
      area-guard and hunt cases and the queued-behaviour fallback: W10 does not arrange those
      states at a qualifying arrival. The rescue case belongs to the out-of-scope team system
      and has no skirmish measurement here
    - Confidence: medium for the rule and its four behaviours; medium for the measured point being
      the target cell's centre; low for the exception, which was found by reading and not traced
      through a whole approach (Open questions)

18. **Cell-destination height and arrival under a high bridge (Q-0201).** When an ordinary
    walking infantry unit receives a cell destination *C*, its locomotor destination has the
    map position of *C*'s centre, not one of the standing positions. Its height is the terrain
    surface height at that centre, plus **414 leptons when C carries a high bridge**, and no
    addition otherwise (F9). The addition is independent of the unit's current layer and of the
    layer its path will enter. It is fixed when the destination is assigned, not recomputed
    from the walker's layer on arrival. Low bridges get no addition.

    On a flat high-bridge cell with centre terrain height *H*, an arrival on the deck has height
    *H* + 416 leptons, so its difference from the locomotor destination is 2 leptons and claim
    16 ends the walk. An arrival on the ground under the same deck has height *H*, giving a
    difference of 414 leptons: claim 16 does **not** complete that destination. This does not
    teleport the unit onto the deck or force an extra physical step. The unit keeps its
    destination on that arrival tick. If no path steps remain, its speed fraction is already 0
    on that tick, but the destination is still present. A remaining path continues normally.
    An exhausted path needs another path computation under S-0212, with next-cell reactions
    owned by S-0209. In particular, if that computation fails while the walker is on the ground
    and its destination cell still carries a high bridge, S-0212 claim 23's give-up row clears
    the destination, even though the unit is already in its destination cell. Completion on
    arrival and later give-up are distinct outcomes.

    S-0212 claim 7 makes a successful path to that cell end on its deck. A route from the
    ground can visit the destination cell underneath the deck before it reaches a bridge
    approach; that visit does not end the walk, and it follows the remaining route to the
    deck. Thus a cell destination is not a request to stand on the same layer as the unit already
    occupies. The unconditional deck-height target is `quirks.walker_bridge_destination_deck`
    (Edge cases); no ground-height cell destination is produced merely by ordering from the
    ground. The reverse height mismatch requires some other source of a destination point,
    not an ordinary cell destination while that cell carries its deck.
    - Backing: `measurable` (scenario W10, part e; the subsequent failed-search outcome also
      has S-0212 claim 23's backing)
    - Confidence: high for the destination point, height and layer independence; medium for
      the path-exhaustion consequences, which depend on the subsequent search

### Infantry landing with the teleporting locomotor

19. **Landing-point choice (Q-0181).** S-0207 claim 20 owns the jump and its phasing in.
    For infantry, let *P* be the destination point supplied to the locomotor, and *C* its cell.
    The desired standing position is the position class of *P*, not the position class of the
    unit's departure point. Occupants are normally respected: use claim 3's position-choice
    table with that desired class, including its draw and its refusal cases. For this choice,
    read the deck occupancy record only in standing-position deck mode, and the ground
    record otherwise. This mode is on only when *C* carries a high bridge and the unit's
    present height exceeds the terrain surface height at *P* by more than 312 leptons
    (3 height levels of 104 leptons). This strict comparison uses integer lepton heights.
    The choice's height is claim 1's surface height at the selected position, plus 416
    leptons in this mode. Thus a ground-height departure normally chooses using ground
    occupancy even for a destination carrying a high bridge; the entry check below
    independently reads deck occupancy.

    A ready queued behaviour can start when the landing point is chosen (S-0215); the current
    behaviour below is the one then in force. Occupants are ignored when it is *capture*, *sabotage*, *enter*
    or *eaten*, and the destination is a vehicle or aircraft in *C*, or the building occupying
    *C*. This is a destination-object condition, not the broader mission-target rule of claim 4.
    In that case use claim 4's position choice: the standing position of *P*'s class, without a
    draw. A destination infantry unit does not give this exception. Whether the candidate cell
    can be entered is still judged by S-0209's infantry entry outcomes. This check uses the
    deck's objects and occupancy record on a high-bridge cell, and the ground's otherwise,
    independently of the standing-position deck mode above. It has no connecting-step height
    restriction. For a cell at terrain height level −6 or lower, the entry outcome is instead
    *clear* without an occupant or terrain test: the check's initial reference level is −1,
    more than four height levels above such a cell. Levels and this comparison are integers.
    This bypass does not remove the preceding standing-position refusal
    (`quirks.teleport_infantry_negative_level_entry`, Edge cases). An outcome other than
    *clear* rejects the landing point even when occupants were ignored for position choice.

    A refused position choice or entry outcome gives no landing point for a cell destination:
    the destination is removed and the unit does not jump. Only an object destination enables
    a nearby-cell search after that refusal. That search is S-0212 claim 25 with these choices:
    - centre: the cell of the destination object's heading point (its next movement point, or
      its own position when it has none),
    - locomotion class and movement zone: the infantry type's own, without zone simplification,
    - reach region: that of the unit's own heading-point cell, using its present bridge-deck
      status; the teleporting locomotor reports its own position as its heading point,
    - search deck mode: on when the centre cell carries a high bridge,
    - a height condition, no further test, no no-overlay condition, bridge cells allowed,
    - no reference cell, so selection uses the landing-point fixation tick number by
      S-0212 claim 25, rather than a random draw.

    A found cell gets another standing-position choice with its centre as desired point,
    the same occupant-ignoring condition and the same standing-position deck mode as the first
    choice. Thus an occupant-respecting second choice takes one ranged draw over 0 to 3 when
    it reaches the centre-class row; an occupant-ignoring choice takes none. There is no second
    infantry entry-outcome test after this choice. If no cell or no position is found, there is
    no jump and the destination is removed. No path between departure and landing is required.
    The point is fixed when the destination is supplied, not followed as the object moves.
    S-0207 claim 20 e owns the height at the actual jump, which can differ from the point chosen
    here on a bridge. It also owns jump-time target loss: an attacker whose strike target
    is the living jumping object retains it (S-0519 Terms). This includes a strike that
    ended normally and kept its strike target (S-0519 claims 18 and 20); no strike need
    be in progress. An erasing lock gives no such exception.
    Override reversion and retention of a different restored target follow S-0215 claim 10.
    These target-retention rules carry those claims' backing and confidence.
    An infantry unit reserves its chosen standing position, not a
    vehicle-present mark (S-0209 claim 1). The reservation uses the chosen point's height:
    it goes into the deck record when its cell carries a high bridge and its height is at
    least 416 leptons above the terrain surface at that point, and into ground otherwise.
    For the initial choice this is the same record the standing-position choice read.
    For the fallback choice the original standing-position deck mode is retained, even
    when the found cell differs from the requested cell; the reservation still uses the
    found cell and chosen height, not the nearby search's deck mode.

    At the actual jump onto a high-bridge cell the infantry is listed on the deck and has
    deck height, regardless of the earlier position-choice mode (S-0207 claim 20 e).
    Subsequent occupancy checks, scatter and movement use its deck status, not ground status
    merely because the earlier reservation used ground. The teleporting locomotor still
    governs movement until some rule changes it; this is not a change to walking locomotion.

    **Post-jump standing-position record (Q-0311).** Landing does not transfer the
    infantry reservation or add a standing-position mark at its actual position. For a
    ground-height chosen point followed by deck placement, the chosen position remains
    taken in the landing cell's ground record. The deck record acquires no mark for it
    at the jump, although the infantry is listed on the deck. A position already marked
    on the deck by another occupant remains marked by that occupant. If the point was
    chosen at deck height instead, its earlier deck reservation remains there normally.

    On a later destination choice, after the completed jump, the old entry is cleared
    using the infantry's actual position and height, not its former chosen ground-height
    point. A deck departure therefore clears the corresponding deck position, even if
    this infantry never marked it there, and leaves its ground mark taken. A successful
    choice marks the next landing point as above; a refused choice marks the infantry's
    current deck position. In the otherwise empty successful-next-jump case, the former
    landing cell is left with a ground standing-position mark but no infantry listed on
    either surface. Ground standing-position choices still treat that mark as taken.
    These are plain taken/free entries, not counted reservations: clearing can also
    clear a mark another infantry shares (S-0209 claim 1).
    This mismatch is `quirks.teleport_infantry_ground_reservation` (Edge cases).
    It is not the vehicle reservation rule of S-0207 claim 20 a.

    **Refused replacement before the jump (Q-0235).** When a new destination reaches this
    landing-point choice before an earlier chosen landing point has been jumped to, the
    earlier reservation is released. A successful new choice reserves the new point.
    If the new choice is refused, including when an object-destination fallback finds no
    cell or no position, the earlier landing point is discarded, not retained: there is
    no jump to it. The destination is removed and the infantry's occupancy entry is restored
    at its present position, using that position's height and cell for the record choice.
    Its listing has stayed at the departure cell throughout. This differs from the retained
    earlier landing point of non-infantry in S-0207 claim 20 b. It concerns a destination
    that reaches landing choice, not one rejected earlier by S-0207 claim 20 a's disabling
    conditions.
    - Backing: `measurable` (scenario W11, especially parts d and h for Q-0235,
      and part i for Q-0311's post-jump records and later clearing)
    - Confidence: high for position choice, record choice, reservation, refused replacement,
      the four behaviours, the cell/object distinction and draws; medium for the search
      choices, post-jump record lifecycle and combined bridge consequences

20. **Placement-time clearing is not owner-dependent.** When placement's fall-back to rest
    supplies a destination to an infantry unit with the teleporting locomotor, the destination
    cell's ground occupants receive a cell-wide scatter request with no threat point, forced
    and move-anywhere, before the landing-point choice. S-0209 owns the recipients and their
    listing order; claims 11 to 13 govern an infantry recipient's refusal, destination and
    draw. These recipients can therefore take draws before claim 19's position-choice draw.
    This request is conditional on the placement situation, not on a human-controlled owner,
    and an ordinary later move to the same occupied cell does not send it merely because the
    mover is infantry. It is not a promise that every placement supplies a destination.
    - Backing: `measurable` (scenario W11 part f, exploratory identification of the placement
      destination followed by a discriminating comparison)
    - Confidence: medium for the placement condition; high that there is no owner test

### Close-range movement inside the current cell

21. **Walking outcome of the near-target shortcut.** When S-0307 claim 22(b)'s shortcut
    succeeds and the search may change the actor, an ordinary walking infantry attacker is
    given an immediate walking target in its own cell. Let *P* be the shortcut's selected
    point. The walking target is chosen again by claims 3 and 4, now with *P* as the desired
    point. For this second choice the attacker's old reservation is released, so its own entry
    does not count as taken. The choice uses ground occupancy unless the cell carries a high
    bridge and the attacker's current height exceeds the terrain surface height at *P* by more
    than 312 leptons, a strict integer comparison. In that case it uses deck occupancy and the
    deck standing-position height. The chosen walking target is reserved as in claim 3.

    On ground in the ordinary occupant-respecting case, *P* is a side position and this
    second choice keeps it: the shortcut found it free, and releasing the attacker's own entry
    cannot make it taken. No second position-choice draw occurs. On a deck the second choice
    can instead select another side position, or find none, because the shortcut read ground
    occupancy while this choice can read deck occupancy. No position means no immediate
    walking target, not a transfer to the ground.

    The target object is kept and the destination becomes the cell of the target's reference
    point, not the attacker's own cell or *P*. Its locomotor destination is therefore the cell
    centre and height of claim 18. The immediate walking target is not replaced merely by
    assigning that destination. The infantry walks towards it by claims 6 to 10, without an
    intervening path step into a neighbouring cell. Reaching it is a cell-arrival update even
    though the infantry has not changed cell. Claims 16 and 17 then decide whether the walk
    ends: a different destination cell alone does not complete on that arrival, but claim 17
    can clear the destination when its mobile-target range and behaviour conditions hold.
    Otherwise the destination remains, and further walking and path computation use the
    ordinary rules. A later order or approach processing can change it normally. When the
    search may not change the actor, neither the walking target nor the destination changes.

    This claim covers ordinary walking infantry only. The shortcut's selection and initial
    draw are S-0307's; it does not prescribe another locomotor's response to the point.
    - Backing: `measurable` (scenario W13; S-0307 scenario RG23 backs the shortcut selection)
    - Confidence: high for the two distinct points and the second position choice; medium
      for the combined arrival and subsequent destination outcome

### Cell-arrival effects

22. **Walking cell-tag arrival timing (Q-0268 part 1).** An ordinary walking infantry
    unit raises the current cell's entered-by tag during the arrival update on the
    tick it snaps to its walking target under claim 6. Its position and height already
    equal that target, and its walking target has been cleared. The arrival update
    can clear its destination under claim 17 before the tag's actions, so those
    actions observe the resulting destination, not necessarily the one held at the
    start of the tick. S-0011 claim 18 supplies tag eligibility, including the
    deck-only condition in a high-bridge cell.

    Stopping is not required. Every intermediate standing-position arrival on an
    uninterrupted walk can raise the cell's tag, even when the destination lies
    farther away and the walker continues next tick. Reserving the next cell on the
    asking tick, crossing the cell boundary between walking targets, and merely
    being within the arrival radius after a physical step do not themselves raise
    it. A same-cell standing-position arrival from claim 21 also runs this update.
    If an entry interaction has already removed the infantry before the common
    arrival update, no current-cell arrival raise is promised by this rule.

    Infantry using their own teleporting locomotor instead follow S-0207 claim 22:
    landing raises the tag in the jump tick after position, height and listing have
    changed, not at reservation or at the end of phasing in. Only the landing cell
    receives this arrival update; cells between departure and landing do not.
    S-0011's tag eligibility still applies. This does not change claim 19's
    post-jump standing-position record rule.
    - Backing: `measurable` (scenario W14; teleport timing also S-0207 scenario D24)
    - Confidence: high for walking-target arrival versus reservation or boundary
      crossing; medium for the combined observable ordering and same-cell case

23. **Infantry arrival and the clear report (Q-0295).** For ordinary walking infantry,
    a cell-arrival update is the arrival tick of claim 6 at each walking target: the
    tick the unit snaps to the cell's chosen standing position, not the tick it
    crosses a cell boundary. It is not limited to the end of the whole walk. The
    same-cell arrival of claim 21 also counts. Reserving a position on the asking
    tick, taking an ordinary physical step, or stopping without reaching a walking
    target does not by itself produce this update.

    A tethered infantry unit gives its clear report on that update. For an ordinary
    unit released by an infantry factory under S-0417 claim 10, this is the first
    standing-position arrival after the tether is established. Neither leaving the
    foundation nor reaching the exit destination is required: an arrival still on
    the foundation reports too. If the release walk spans several cells, it reports
    at the first such arrival, even while its exit destination is farther away and
    movement continues. A one-cell release walk can instead make that first arrival
    coincide with reaching the exit cell. There is no separate test for the first
    cell outside the foundation, and the vehicle report's building-cell suppression
    of S-0417 claim 7 does not apply to this infantry report.

    S-0417 claim 10 owns the resulting end of factory contact and tether; the report
    does not by itself finish or redirect the infantry's walk to the exit. Claims 16
    and 17 still own walking completion.

    A tethered infantry unit using its own teleporting locomotor instead reports
    clear on its first jump's cell-arrival update after tether establishment
    (S-0207 claim 22). Under S-0417 claim 10, factory contact and tether end in that
    jump tick, after landing position, height and landing-layer listing change but
    before the landing cell's entered-by tag is raised. Its tag actions therefore
    observe the factory contact place already free. No foundation-exit or
    different-destination test applies, and the vehicle building-cell suppression
    does not apply. The ordinary infantry factory gives no run-away reply or extra
    exit-cell leg. Landing reservation alone does not report clear; neither
    phasing-in completion nor falling back to rest is required for release.
    Without a jump, another cell-arrival update or an independent contact-breaking
    event, contact and tether remain, with no factory-release contact timeout
    (S-0417 claim 10).

    These report rules cover ordinary factory release without an entry interaction
    that removes the unit before it can report. They do not prescribe jumpjet or
    passenger-unloading behaviour.
    - Backing: `measurable` (scenario W15; S-0417 U10 a supplies the walking release
      setup, U10 d/e cover teleporting release, entered-by ordering and no-jump retention)
    - Confidence: high for the walking and jump arrival events, report on the first
      update, contact/tether loss before the entered-by raise and absence of a
      foundation/destination test; medium for the combined release, continuing-walk
      and no-jump consequences

### Scatter and unreachable starting cells

24. **A scatter destination is not an escape path.** Claims 12 and 13 choose a destination,
    not a walking target or a route. Neither the neighbouring-cell choice nor the nearby-cell
    search requires that destination to share the unit's reach region. Once chosen, however,
    an ordinary walking infantry unit reaches it only through normal pathfinding, including
    S-0212 claim 6's reachability gate, and the entry outcomes of S-0209. The forced and
    move-anywhere options do not exempt that walk from the gate. A failed gate gives no
    path, not a direct step to the chosen cell. Destination retention, stopping and retry
    after failure remain separate from destination selection (S-0212 claims 21 and 23).

    In particular, take a stationary ordinary walker with no path or walking target, on a
    legal cell inside the usable area that belongs to no reach region of its movement zone,
    with none of S-0212 claim 6's exceptions applying. An accepted scatter request can give
    it a destination on otherwise usable ground in a reach region, but it cannot leave its
    starting cell for that destination: the regions differ and it gets no path. This holds
    even for an immediately adjacent destination, and for a request without a threat point
    whose destination search has no reach-region restriction. There is no scatter-only
    one-cell escape or relocation. A chosen cell that itself belongs to no reach region
    passes the region comparison under S-0212 claim 4, but still needs a path and legal entry
    outcomes; passing that comparison alone does not make a cliff enterable.

    Being placed on such a cell does not itself send a scatter request. Claims 11 and 15
    still decide acceptance and damage requests. These rules do not establish which attack
    or acquisition event destroys a stranded unit; S-0309 claims 7, 11 and 18 distinguish
    the searches and retaliation that can reach it. This answers the ordinary walking
    infantry scatter part of Q-1502 question 1, not other locomotors or every cause of death
    in its arena games.
    - Backing: `measurable` (scenario W16 for stationary scatter and direct-move controls,
      both damage-request forms, and placement without a request). The comparison when the
      chosen destination also has no reach region carries S-0212 claim 4's backing.
      `unmeasured` for a forced move-anywhere request on the stranded fixture: W16's human
      scatter and damage requests are not move-anywhere, and no reliable setup here makes
      crowding send that request to the cliff occupant. Its options do not supply a path,
      but that particular combination is not independently tested
    - Confidence: high for destination selection not bypassing pathfinding; medium for the
      combined cliff-cell outcome, pending a discriminating observation

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

**F9. Cell locomotor destination.** For cell coordinates (*c*ₓ, *c*ᵧ),

> *x*ᴅ = 256*c*ₓ + 128, *y*ᴅ = 256*c*ᵧ + 128
> *z*ᴅ = *H*(*c*ₓ, *c*ᵧ, 128, 128) + *a*

*a* is 414 leptons when the cell carries a high bridge at destination assignment, and 0
otherwise. Coordinates and addition are exact integers, with no additional rounding.
*H* and its floating-point calculation and truncation are S-0106 claim 2 and F1's.
The 414-lepton addition is an engine constant, distinct from the deck's 416-lepton height.
Units: leptons.

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
| Walk-end height threshold | 208 | leptons; the absolute height difference must be strictly less than this value (two height levels) | claim 16 |
| High-bridge addition to a cell locomotor destination | 414 | leptons | claim 18; independent of the walker's layer |
| Height above destination terrain surface needed for teleport standing-position deck mode | more than 312 | leptons | claim 19; strict integer comparison |
| Height above chosen-point terrain surface needed for an infantry deck reservation | at least 416 | leptons | claim 19; high-bridge cell required |

## State and ordering

| State | Event | Result |
|---|---|---|
| Walking towards a target in the next cell | distance < 17 leptons | placed on the target, arrives this tick; no walking target until its next asking tick |
| Walking towards a target | distance ≥ 17 leptons | moves *s* leptons towards it this tick |
| Standing, about to step into the next cell (asking tick, at the earliest the tick after arrival) | the cell is clear for it and a position is free | that position becomes its walking target (claim 3); no movement this tick, the first step on the next (a walking target chosen at once at a scatter that comes before the walker's own locomotion is stepped towards on that tick, claim 6) |
| Standing, about to step | no position is free | no walking target; S-0209 decides |
| Any | scatter request refused (claim 11) | no draw and no new destination; a unit whose current sequence is a deploy kind, sent a forced, move-anywhere request, has still been sent a request for *undeploy* (claim 11) |
| Any | scatter request accepted, destination found | attempts to move to that cell under normal pathfinding and entry rules (claim 24) |
| Any | scatter request accepted, no destination found | does not move; the draw has been taken |
| Walking, arrival tick | target is a mobile actor in range at its cell centre, behaviour attack, area guard, hunt or rescue (claim 17) | loses its destination, keeps its target; the walk ends on this tick (claim 16) |
| Walking, arrival tick | no locomotor destination, or arrived in the locomotor destination's cell within two height levels (claim 16) | walk ends: no destination, speed fraction 0, stands at the position reached |
| Walking, arrival tick | neither of the above; path steps remain | destination kept; asks for its next walking target on the next tick (claim 6) |
| Walking, arrival tick | destination height test fails; no path steps remain | destination kept, speed fraction 0; subsequent path computation and give-up follow S-0212 (claim 18) |
| Walking, destination is its target object | firing check answers *can fire* (claim 17, exception) | loses its destination and stops as its firing sequence starts |
| Teleporting infantry, destination supplied | first landing choice succeeds | reserves the chosen standing position; jump follows S-0207 claim 20 |
| Teleporting infantry, first landing choice refused | cell destination | no jump, destination removed |
| Teleporting infantry, first landing choice refused | object destination | nearby search and second position choice of claim 19; no jump if either finds nothing |
| Teleporting infantry, earlier landing reserved but not jumped to | replacement landing choice refused, including failed fallback | earlier reservation released, no retained landing point, destination removed, present-position entry restored (claim 19) |

Ordering stated as consequences (the draw counts and order are checked with the read-only recorder
of the random stream, scenario W9): the random draw of claim 3 happens on the tick on which the
walker chooses its position in the next cell (its asking tick, claim 6): the tick after the one on
which it arrives in the previous cell, or, on the first cell of a walk, the tick before its first
step. A walking target chosen at once at a scatter (claim 6) takes that draw, when its desired point
has class centre, on the tick of the scatter, which is also the tick of the first step when the
scatter comes before the walker's own locomotion in that tick. The draw of claim 12 or 13 happens on
the tick on which the scatter request arrives, and only when the request is accepted: a request
refused by any case of claim 11 takes no draw, while an accepted request takes its one draw even
when no destination is then found. When one hit sends both damage requests of claim 15, both arrive
on the tick of the hit. When both are accepted, request A's draw comes first and request B's last,
and claim 3's draw for the walking target that request A chooses at once can fall between them
(claim 15). When request B finds a destination, the unit heads for it at the end of the tick: claim
12 gives it from request B's draw (around the cell of the walking target the unit has when request B
arrives, which is request A's walking target when request A chose one at once and the walking target
of the step under way when the unit is still finishing a step, and otherwise around the unit's own
cell, claim 15), and it replaces request A's destination (scenarios W7, part j, and W9, part c). On
an arrival tick, claim 17's in-range stop is decided before claim 16's end-of-walk test, both after
the walker has been placed on its walking target, so a walker that comes into range at a standing
position stops on that same tick and does not ask for a next walking target (scenario W10).

## Edge cases and quirks

- **Crate pickup timing.** S-0520 claim 6 owns pickup: a walker attempts it when it commits
  to the step into the crate's cell, before arrival, not on the arrival tick. Its pickup gates
  can prevent that step. S-0520 claim 20 owns speed bonuses; claim 7 here applies a changed
  speed multiplier from the next walking step. These pickup and bonus rules carry S-0520
  claims 6 and 20's backing and confidence. W4(c) needs only that pickup happens during the walk.

- **Negative-level teleport entry** (`quirks.teleport_infantry_negative_level_entry`, default:
  original). Claim 19's entry check accepts a cell at terrain level −6 or lower without testing
  terrain or occupants, though standing-position availability is still tested. With the switch
  off, that early bypass is omitted and the ordinary landing entry check applies. W11(g) tests
  the bypass. This does not change the map importer or ordinary walking entry rules.

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

  Ordinary occupant-respecting walkers never take the top or centre positions (claim 3).
  Pre-placed units can stand there (claim 5); occupant-ignoring walking can choose the centre
  (claim 4), as can occupant-ignoring teleport placement (claim 19). With the switch off,
  a unit standing at the top position is recorded as
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
- **An *enter* walker keeps its old path** (`quirks.enter_keeps_stored_path`, default: the
  original). By claim 6, a walker given a new destination keeps its stored path when its current or
  queued behaviour is *enter* and it is in contact with nothing at that moment (glossary *contact*).
  So an infantry unit ordered into a transport while it walks elsewhere walks on along its old
  route, up to the whole stored path, before it turns towards the transport, and one ordered away
  from a transport walks on towards it first. An order to garrison a building gives *capture*, not
  *enter*, so it drops the path of a walker under *move* under either setting (claim 6). With
  the switch off, every new
  destination drops the stored path, whatever the behaviour, and the walker searches for the new
  destination as claim 6 gives for a dropped path. Scenario W17, parts f to h, tests the original's
  behaviour.
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
- **Scatter around the next cell.** A walking unit that accepts a scatter request looks for its new
  destination around the cell of its walking target (the scatter centre cell), which is usually the
  next cell of its path, not around the cell it is in and not around its farther destination (claims
  12 and 13), while the direction away from a threat point is still taken from the unit's own
  position. On its arrival tick, and on its asking tick until it has chosen its walking target
  there, it has no walking target, so it looks around its own cell. A walking target chosen at once
  at a scatter (claim 6) counts from that moment (claim 15). It is listed here because it is easy to
  miss; it is ordinary behaviour of the original, not a defect, and has no switch.
- **The in-range stop tests the target's cell centre** (claim 17). The approaching walker's stop
  measures range to the centre of the target's cell at that cell's terrain height, not to the
  target itself, while S-0215 claim 37 and the firing check measure to the target's own reference
  point (S-0307 claim 7). Near the range
  boundary the walker can therefore walk one standing position further than needed, or stop one
  standing position short and then be moved again by S-0215 claim 37. It is the original's
  behaviour, not a defect we fix, and has no switch; a variant measuring to the target's own
  position would differ only within half a cell of the boundary (scenario W10, part b2).
- **A cell destination under a deck still aims at deck height**
  (`quirks.walker_bridge_destination_deck`, default: original). Claim 18 sets its height to
  the centre's terrain height plus 414 leptons, even for a walker travelling under the deck.
  Arrival there can leave a destination present despite a speed fraction of 0, until a later
  search or behaviour clears it (W10 e). With the switch off, the cell destination's height
  for the arrival test is the terrain height at the cell centre on the walker's arrival layer:
  that height plus 416 leptons on the deck, or without an addition on the ground. Its map
  position stays the centre, and pathfinding and object destinations are unchanged.
- **No early stop against non-mobile targets** (claim 17). A walker approaching a building or a
  cell always walks to its approach cell, even when it came into range earlier. It has no switch.

- **Infantry teleport ground reservation left behind**
  (`quirks.teleport_infantry_ground_reservation`, default: original). Claim 19's
  ground mark survives deck placement without a matching new deck mark. A later
  destination choice clears by actual deck height, leaving ground taken and
  potentially clearing another infantry's deck mark. With the switch off, the
  reservation moves to the actual landing surface at the jump, at the same standing
  position: ground is released, deck is marked, and later clearing uses that deck
  entry. This switch is separate from the vehicle switch because infantry landing
  does not add the vehicle's placement-time occupation mark.

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
| Rules, infantry type section, `Locomotor` | whether claims 19 and 20 apply, including claim 19's pre- and post-jump record rules and refused-replacement rule | identifier parsed and recognised by S-0207 claim 14 and its Import mapping | keeps the value in force; teleporting locomotor before any layer gives one |
| Rules, infantry type section, `SpeedType`, `MovementZone` | landing-search locomotion class and movement zone (claim 19) | named choices parsed by S-0211 Import mapping | S-0211's defaults and layering rules |
| Rules, land-type sections, `Foot` | the walker speed percentage; only zero matters (claim 9) | percent | owned by S-0211 |
| Map, `[CellTags]`, `[Tags]`, `[Triggers]`, `[Events]`, `[Actions]` | cell-tag attachment and entered-by event/action eligibility at claim 22's arrival | map records and event/action parameters parsed by S-0009 and S-0011 | no attached tag or trigger when absent, as those specs give; arrival timing is an engine rule, not an imported delay |
| Map, `[Infantry]` entries, the sub-cell value | standing position number (claim 5) | integer, 0 to 4 | values outside 0 to 4: centre, importer warning |
| Map terrain and overlay blocks, `[IsoMapPack5]`, `[OverlayPack]`, `[OverlayDataPack]`, with theater tile and overlay definitions | destination cell's centre terrain height and whether it carries a high bridge (claim 18, F9), and landing surface heights, bridge recognition and occupancy-record selection (claim 19) | block encodings and terrain import owned by S-0012, S-0009 and S-0010; high-bridge recognition owned by S-0211 | the cited specs' empty-cell and absent-overlay rules; the 414-lepton addition and 208-lepton tolerance are engine constants, not imported settings |

| Rules, infantry type section, `Locomotor`; rules, factory type section, `ExitCoord`; art, factory image section, `Foundation` | locomotion and release geometry for claim 23's factory walk | locomotor identifier owned by S-0207 claim 14; integer lepton triple for release displacement and foundation form owned by S-0404 and S-0417 | those specs' defaults and layering rules; claim 23's arrival event and report timing have no imported delay or foundation-clearance setting |

The scatter property of each behaviour (claim 11) is imported by S-0215.
Claim 19's 312-lepton strict choice threshold, 416-lepton reservation threshold and
refused-replacement outcome are engine rules, with no separate imported setting.

## Measurement scenarios

All run under the glossary's *measurement setup*, with the director off and game speed index 2
unless stated otherwise. Use the installed release's unmodified data except the named test files
and purpose-made maps explicitly allowed by that setup; remove those test files afterwards.
Choose types by the imported properties each scenario requires, and record the effective fields
used to compute its expected results. No particular shipped type, speed, weapon value or release
geometry is assumed. Runs use purpose-made flat maps of one theater with clear terrain
unless stated, with one human player and one computer player whose start position is far away,
besides the neutral and special houses that every skirmish has (glossary *house*). The expected values for
computer-owned units are the original's, and in our engine they hold for units of the neutral and
special houses too (Terms, the project lead's decision of 2026-10-10). Our engine's departure for AI houses (claims 11 and 15) is not
measurable against the original, which has no such houses.
Observations are taken with a *recorder* that reads unit world positions per tick (or from
screenshots when stated). In scatter scenarios, "moves on the tick of the request/hit" means
that a new scatter destination is assigned on that tick, not that a physical walking step must
occur then. Record destination assignment separately from the first physical step; claim 24
still governs whether any route exists. Where a scenario counts ticks "from the first step to arrival", the count
is *t*ₐ − *t*₁ + 1, where *t*₁ is the first tick on which the unit's position changes and *t*ₐ the
tick on which it is placed on its final standing position: both ticks are included. Paths are
straight runs of clear cells, so S-0212 sends every walker along the run.
For each scenario, a qualified observation contradicting any stated expected result is a
falsifier, not just the examples in its falsifier list. A failed setup qualification or an
explicitly exploratory subpart is inconclusive. Statistical parts use their stated bands.

**Scenario W1: standing positions and capacity of the side positions (claims 1, 3, 4).**
- *Setup:* (a, b) an empty target cell *T* = (*x*, *y*). Four infantry of one type, owned by the
  human player, each pre-placed at sub-cell 2 (the right position): one in each of the cells
  (*x* − 1, *y*), (*x* − 2, *y*), (*x* − 3, *y*) and (*x* − 4, *y*), on the same row as *T*. (c) A
  separate game: a transport vehicle whose imported passenger rules allow infantry, standing still
  in *T*, and one infantry type eligible to enter it, pre-placed at sub-cell 2 in (*x* − 1, *y*). (d) As (c),
  with the infantry unit pre-placed at sub-cell 0 (the centre) instead. (e) A separate game: *T*
  is a foundation cell, in the foundation's west column, of a capturable building of the computer
  player, and an infantry type with the imported building-capture property, owned by the human player, is
  pre-placed at sub-cell 0 in (*x* − 1, *y*). (f) As (d), with a landed transport aircraft
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
  - (d) A separate game. Five infantry units of the human player, of one type that is neither
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
- *Setup:* every type and setting below is chosen so that F5 gives *s* from 2 to 38 in every
  part (claim 6). A straight east-west run of 12 clear cells per unit, side by side, each unit owned by the
  human player and pre-placed alone at sub-cell 2 in the west end cell of its run. No crates on the
  runs. The house's country is noted, and its country speed factor for infantry is read from the
  rules data.
  - (a) Three infantry types with different imported speed fields, at the lowest veterancy rank.
  - (b) One infantry type whose imported veteran ability list names the faster ability, placed by the
    test map at veteran rank, beside a unit of the same type at the lowest rank. The type is chosen so
    that trunc(*s*₁ × *k*ᵥ) differs from *s*₁ for the imported veteran speed factor; if no available type
    gives such a pair, the run still checks that *s* = trunc(*s*₁ × *k*ᵥ).
  - (c) One infantry type, and a crate placed on the unit's own run one cell east of its start
    cell. A named test file (an override section of the test map, or a loose copy of the rules file)
    makes the speed crate the only crate outcome. The crate's speed multiplier is read from the rules
    data.
- *Procedure:* order each unit to the cell 10 cells east of its start. Repeat (a) at game speed
  indices 1 and 6, using the same effective imported fields, to test speed invariance per tick.
- *Observable:* the world *x* and *y* of the unit on every tick of the walk, and the number of ticks
  from the first step to arrival.
- *Runs:* 1 per unit and game speed tested.
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
  movement between two cells. Changing game speed leaves the per-tick steps and tick counts
  unchanged; only elapsed real time changes.
- *Falsified if:* any steady step differs from the expected *s*, the final position is not exact,
  *y* changes, or the number of ticks differs.

**Scenario W5: crawling, panic and the prone non-crawler (claims 8, 10).**
- *Setup and procedure:*
  - (a) A unit of the human player, of an infantry type whose art enables crawling, on open ground, ordered
    to a cell ten cells away. While it walks (its behaviour is then *move*, in which a
    human-controlled unit does not retaliate, S-0215 claim 18), a computer-owned unit with a weak
    weapon hurts it once, enough to raise its fear to lie down (S-0107 claim 16) without killing it.
    If it stops when it lies down, order it on at once. Record its per-tick step while prone.
  - (b) The same unit and the same move, unhurt.
  - (c) A computer-owned fraidycat civilian type hurt by a known attacker, which panics and flees.
    Record its per-tick step while it shows *panic*.
  - (d) A named test file (a loose copy of the rules file, and of the art file if needed) marks a
    chosen infantry type of the human player as a cyborg whose art cannot crawl. Order the unit to a
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
  imported walker percentage is strictly between 0 % and 100 %. One infantry of the same type at the
  start of each, at sub-cell 2.
- *Procedure:* order both to the far end on the same tick.
- *Observable:* ticks from the first step to arrival.
- *Runs:* 1.
- *Expected:* equal.
- *Falsified if:* the two counts differ (in particular, the unit on the slower land type takes
  longer).

**Scenario W7: scatter acceptance, destinations and damage requests (claims 11 to 13, 15).**
- *Setup:* Unless a part says otherwise, units are at the lowest veterancy rank, and the
  player-scatter rule is off: if the effective imported rule is on, a named test file (a loose copy
  of the rules file) turns it off. Where a part needs a behaviour with the scatter property and the
  effective imported behaviour lacks it, a named test file gives it. In parts c, g, h1, h4 to h7, i and j the unit that scatters stands on
  open flat ground with no other unit within 2 cells, so its search collects exactly its eight
  neighbours (claim 13). The recorder reads the tick number (Terms) of every tick. For a unit that
  walks when a request reaches it, *Z* is its scatter centre cell on the tick of the request: the
  cell of the standing position it is walking to (the next cell of its run), or its own cell when
  the request falls on its arrival tick or asking tick (claim 6). The recorder reads it from the
  walker's position and walking target on that tick.
  - (a) Four infantry units on open ground: an idle soldier of the human player standing on a side
    position of its cell (map sub-cell 2) in the path of an enemy crusher driving towards it (a
    forced request, S-0210); a soldier of the human player walking across the crusher's path at the
    moment it approaches (these two in a separate game, in which a named test file turns the
    player-scatter rule on, so that the crusher's request, which is not move-anywhere, reaches them
    by S-0209 claim 11); an idle soldier of the human player on a side position of its cell (map
    sub-cell 2) hit once by a computer-owned unit; and an idle computer-owned fraidycat civilian of
    an unarmed type, whose behaviour has the scatter property, hit once by a unit of the human
    player.
  - (b) A computer-owned fraidycat civilian of an unarmed type (so that it may not retaliate and
    cannot attack back) walking north along a straight run of open flat ground
    under a map trigger's move to a cell *D* 8 cells away, whose move behaviour has the scatter
    property. When it is 4 cells from *D* it is hit once by a long-range weapon fired from due map
    east, from a unit on its row. Its locomotor is moving, so the hit sends it request B only
    (claim 15).
  - (c) An idle computer-owned fraidycat civilian of an unarmed type, standing in *guard* on open
    flat ground. A named
    test file removes the scatter property from *guard* if its imported behaviour properties include it and
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
    positions; one of them is of a type with the imported infantry-deploy property, whose *undeploy* entry has image
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
    (Terms; for example an unarmed type that is not an engineer, or a civilian type
    that a named test file makes not a fraidycat), in a behaviour with the scatter property, hit by
    a unit of the human player; (h2) the same type owned by the human player, hit by a computer-owned
    unit; (h3) an idle armed computer-owned soldier, not a fraidycat, in a behaviour with the
    retaliate property, of a type whose imported weapon properties allow it to hit the attacker, so that it may retaliate
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
  - (j) Both damage requests at once (claim 15). An idle computer-owned fraidycat civilian standing
    in *guard* on open flat ground on a side position of its cell (map sub-cell 2), with no other
    unit within 3 cells. A named test file gives *guard* the scatter property and removes its
    retaliate property, so that the unit may not retaliate (Terms) and the hit sends it request A by
    its second case as well as request B. A unit of the human player due map east of it, 5 cells
    away on its row, hits it once. Before the runs, the mapping from the residue (*N* mod 8) to the
    neighbour that claim 13 gives is taken from the runs of (c) and (i1). Variants: (j1) the
    civilian as above, owned by the computer player; (j2) the civilian pre-placed by the map as
    owned by the neutral house (S-0009 claim 27), and attacked on the user's order; (j3) the same
    with the special house. The game has a computer player, so the unused-house cleanup of S-0009
    claim 28 does not run; (j2) and (j3) count only when the recorder shows the civilian present at
    the start.
  - (k) The fallback of claim 13. A purpose-made map with a flat block of 5 × 5 cells raised at
    least 2 height levels above every cell outside it that infantry can enter (the cells around
    the block are cliffs, water or lower ground), far from the players' start positions. The
    unit of (c), with the same test file, is pre-placed at the right standing position (map value
    2) of the centre cell of the block. Each of the other 24 cells of the block holds an occupant: a vehicle of
    the human player in one corner cell, armed and ordered to hit the civilian once, and in each of
    the other 23 cells one idle, unarmed infantry unit of the civilian's house, of a type that is not
    a fraidycat (a named test file may remove the fraidycat property from a civilian type, as in (h1)),
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
    the occupants of a type whose imported *cheer* entry has image frames; and the user moves the fourth unit
    by a move order. When the fourth unit is one cell short of the cell before *G*, the user gives
    the three occupants the celebration order, so that they play *cheer* (S-0107 claim 21) when the
    fourth unit stops in front of *G* and starts sending its forced, move-anywhere requests (S-0209
    claim 8). If the recorder shows a cheer ending before the fourth unit stops, the run is repeated
    with the order given later.
  - (n) The second case of claim 11 (deploying, human-controlled owner, request not both forced and
    move-anywhere). An infantry type with the imported infantry-deploy property, owned by the human
    player, on a side position of its cell (map sub-cell 2) on open flat ground with no other unit
    within 2 cells, not a fraidycat, in a behaviour in which it may retaliate (S-0215 claim 18). A
    named test file turns the player-scatter rule on, so that the unit has scatter permission. A
    computer-owned unit with a long-range weapon, outside the soldier's sight (S-0215 F7), hits it
    once, so that it does not attack back, and the hit sends it request A (claim 15, first case),
    which is forced and not move-anywhere. Variants, each hit given on a tick when the unit plays:
    (n1) *deploy*, (n2) *deployed*, (n3) not a deploy kind (the same unit undeployed, standing, as
    the control). A unit playing *deployed-fire* has a target, so it is not sent request A (claim
    15); variant (n4) uses the crushing request instead: the same unit, deployed and playing
    *deployed-fire* at a computer-owned unit in its weapon range, while a computer-owned crusher,
    under a map trigger's move, begins a track whose first cell is the unit's cell, so that the unit
    is sent a forced request that is not move-anywhere (S-0210 claim 7, through S-0209 claim 11,
    which the player-scatter rule lets reach it). (n5) The owner condition of the second case: a
    unit of the same type, not a fraidycat, owned by the computer player and deployed as the
    occupant of part (e) is, with the player-scatter rule off. A named test file gives its current
    behaviour (the recorder shows it) the scatter property and removes that behaviour's retaliate
    property, so that it may not retaliate (Terms), and a unit of the human player hits it once,
    which sends it request A by its second case (claim 15), forced and not move-anywhere. The
    variant counts only when the recorder shows it playing *deployed* on the tick of the hit.
    *Deployed-idle* is left out, since no built-in behaviour starts it (S-0107 claim 20).
  - (o) The *undeploy* table of claim 11 (its *deploy*, *deployed* and *deployed-fire* rows). As
    (e), with these changes: the three occupants of *G* and the fourth
    unit belong to the human player; the occupants are of a type with the imported infantry-deploy property, and the user
    gives them the deploy order. The fourth unit, ordered through the gap, stops in front of *G* and
    starts sending its forced, move-anywhere requests (S-0209 claim 8). Variants, by the sequence
    that all three occupants play on the tick of the first request, reached by giving the deploy
    order earlier or later (as in (m), the recorder shows the sequences): (o1) *deploy*,
    (o2) *deployed* and (o3) *deployed-fire* (by an enemy unit in the occupants' weapon range), with
    a type whose imported *undeploy* entry has image frames; and (o4) *deployed*, with a named test
    file (a loose copy of the art file) that gives the occupants' type an *undeploy* entry with no
    image frames. *Deployed-idle* is left out, since no built-in behaviour starts it (S-0107 claim 20).
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
    file gives it if the imported behaviour lacks it). When it is 4 cells from *D* it is hit once by a
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
  cell of its new destination, given as a step direction from *Z* in (b), and *Z* itself. In (c),
  (g), (h) and (i), the first cell it moves to, as a step direction from its own cell, together with
  the residue of the tick number mod 8 on the tick its destination was set. In (d), whether the cell
  it moves to is a neighbour of *Z*, of *D* or of the cell it was in when hit. In (e), whether it
  undeploys and leaves *G*. In (j), the destination it holds at the end of the tick of the hit and
  the first cell it walks to, each as a cell relative to its own cell, and the residue of the tick
  number mod 8 on that tick. In (k), the first cell the civilian moves to, as a step direction from
  its cell, and the residue of the tick number mod 8 on the tick its destination was set. In (l),
  the cell of its new destination. In (m), (n) and (o), for each unit and each tick, its current
  sequence and whether it has a destination, and, from W9, whether a draw was taken on the tick of
  the request. In (p), the cell of its new destination, as a step direction from *D*. In (q),
  whether its destination changes on the tick of the hit and, if it does, the new destination as a
  step direction from *Z*. In (r), *Z*, and the cell of the new destination set on the tick of the
  crusher's request, with its distance in cells from *D* and from *Z*. A move towards the attacker
  by a unit of an armed type counts as an attack, not a scatter; for a unit of an unarmed type every
  move counts as a scatter.
- *Runs:* (a) 1 each. (b) and (f) until 100 runs have counted (see Expected). (c) 20. (d) the runs of
  (b). (e) 1.
  (g), (h1) and (h4) to (h7) 12 per variant and attacker side, (h2) and (h3) 1 per attacker side. (i) 12 per
  soldier. (j1) 40, (j2) and (j3) 20 each. (k) 40. (l) until 50 runs have counted (counted as in (b)). (m) 1. (n) 12 per variant. (o) 1 per variant. (p) until 100 runs have counted (counted as in
  (b)). (q1) 12; (q2) and (q3) until 50 runs have counted (counted as in (b)). (r) until 20 runs
  have counted. Runs start on
  different ticks so that the residues mod 8 vary.
- *Expected:*
  - In this scenario a unit that "moves on the tick of the hit" takes its new destination on that
    tick.
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
  - (j) in every variant the unit takes its new destination on the tick of the hit, and its first
    step on that tick when its own update comes after the hit in that tick, otherwise in the next
    tick (claim 6). In (j2) and (j3) this shows that the house gives the unit scatter permission and
    the second case of request A as a computer-controlled owner does: a unit treated as
    human-controlled would be sent no request A (the player-scatter rule is off) and would refuse
    request B (no scatter permission), so it would take no new destination on that tick. Let *A* be
    the neighbour of its cell that the residue gives (the destination of request A). Request B ends
    as its destination, and B looks around *A*, the cell of request A's walking target (claim 15).
    Two sets are named, each the set of destinations that claim 12 gives for the five draws 0 to 4,
    with the threat due map east (so *d*₀ is map west): *H*₁ with the unit's own cell as scatter
    centre cell, and *H*₂ with *A* as scatter centre cell. On open ground these are the neighbours
    of the own cell, respectively of *A*, in the relative directions −2 … +2 around map west, unless
    the first candidate is not enterable. In every run the destination lies in *H*₂, and the unit's
    first step goes into *A*.
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
- *Falsified if:* (a) a different move-or-stay pattern. (b) a direction pointing towards the
  threat, or a count outside 8 to 32. (c) the hit itself moves the unit, two first moves with the
  same residue mod 8 end in different cells, or it stays although free cells are next to it. (d)
  it moves to a cell that is not a neighbour of *Z* (a neighbour of *D* far from *Z* would mean
  the centre is the farther destination). (e) it stays deployed, or undeploys but never leaves *G*
  although cells next to the gap are free. (f) any counted destination differs from every
  predicted one; a candidate that fails the check is chosen while a later qualifying candidate
  passes; or a count falls outside its band (in particular, if the west candidate is chosen at the
  rate flat ground would give). (g) in (g1) or (g3), it stays while free cells are next to it, or
  two runs with the same residue mod 8 end in different cells; in (g2), it scatters. (h) (h1),
  (h4), (h5), (h6) or (h7) stays while free cells are next to it, or two of its runs with the same
  residue end in different cells; (h2) or (h3) scatters. (h6) or (h7) staying would mean that the
  original treats that house as human-controlled. (i) the check of the source fails (pick another
  source); (i1) stays while free cells are next to it, or two runs with the same residue end in
  different cells; (i2) moves. A scatter of (h3), or (i1) staying, would contradict this spec's
  claim 15 and S-0107's revised scenario M15(b), and so would the hit itself moving the unit in
  (c). (j) in any variant the unit takes no new destination on the tick of the hit, or has not
  taken its first step by the end of the next tick, or a destination lies outside *H*₂: in *H*₁
  only (request B would then look around the unit's own cell, so request A would have chosen no
  walking target), or a first step into a cell other than *A*, or in neither set (for example *A*
  itself when *A* is in neither, which would mean request B was refused or was judged before
  request A). (k) the civilian never moves while the neighbours are enterable (no fallback), it
  moves in direction 4, 5 or 6, a count falls outside its band, or every pair of runs with the
  same residue mod 8 ends in the same direction while the directions follow the residue (which
  would mean a search result and not the draw). (l) the destination is *K*₂ in any counted run
  (without the exclusion it would be *K*₂ in about three runs out of five). (m) an occupant gets a
  new destination on a tick on which its recorded sequence is still *cheer*, which is what a first
  case that exempted every forced, move-anywhere request would give. (n) in (n1) or (n2) it moves
  or takes a draw on the tick of the hit, or in (n4) on the tick of the crusher's request; in (n3)
  it stays although free cells are next to it; in (n5) it takes no new destination on the tick of
  the hit (what a second case without its owner condition would give). (o) in (o1) an occupant
  moves or takes a draw while it plays *deploy*; in (o2) or (o3) an occupant does not start
  *undeploy* on the tick of the first request, or takes no new destination or no draw on that
  tick; in (o4) an occupant starts *undeploy*, or takes no new destination or no draw on that
  tick. (p) a counted destination is the map west, map north-west or map north neighbour of *D*
  (which the plain form of the usable-area test would give), or a count falls outside its band.
  (q) in (q1) the destination changes on the tick of the hit in any run; in (q2) or (q3) a run
  keeps its destination on the tick of the hit, or a new destination points towards the threat.
  (r) a counted new destination is a neighbour of *D* (the scatter centre cell would then be the
  farther destination), or is not a neighbour of *Z*, or the destination does not change on the
  tick of the request although the neighbours of *Z* are free.

**Scenario W8: sideways drift on the map axes, and very slow and very fast walkers (claims 6 and
14, Edge cases).**
- *Setup:* four infantry of one type, owned by the human player, each pre-placed at sub-cell 2 (the
  right position) in the end cell of its own straight run of 12 clear cells: one run along map
  north, one along map east, one along map south and one along map west. A unit at a side position
  keeps that position in every cell of a straight run (claim 3), so each leg is 256 leptons long and
  parallel to its run, and no random draw is involved. The house's country speed factor for
  infantry is read from the rules data.
  - (a) A type at the lowest veterancy rank, chosen so that F5 gives *s* from 2 to 38 for the
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
    - W9 (f): a run set up as part (f) but with the celebration order of W7 part (m) given to the
      three occupants of *b* before the scatter order, so that they refuse the request that *A*
      sends; on the tick of the scatter order, part (f) itself takes three draws over 0 to 4 more
      than this reference run.

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
  double hit of W7 part (j), variant j1, 40 counted runs, with the civilian on a side position
  (c1) and, in 10 more runs, at its cell's centre (map sub-cell 0, c2). (d) No destination found.
  As W7 part (k), with these changes: each of the eight neighbours of the centre cell is of a land
  type that infantry cannot enter (a walker speed percentage of zero, claim 9) and holds no unit,
  so that 15 of the 16 outer cells of the block hold an idle infantry unit and the corner cell the
  vehicle; and the civilian's test file is that of W7 part (j), so that the hit sends it request A
  by its second case and request B, and claim 11 accepts both. No cell qualifies for the search
  (the outer cells of the block hold occupants, the neighbours cannot be entered, and the cells
  outside the block fail as in W7 part (k)), and no neighbour is enterable, so neither request
  finds a destination. 5 runs. (e) Request A not sent to a walking unit: the hit of W7 part (b),
  40 runs, counted as there. (f) Requests from an extra locomotion: the set-up and order of W17
  part (i), 5 runs.
- *Observable:* per tick, the number of draws taken from the stream and their values, replayed
  from the state recorded at the end of the tick before.
- *Expected:*
  - (a) exactly one draw over 0 to 3 on the asking tick of each cell entered whose desired point
    has class centre (claim 3), and none on any other tick of the walk.
  - (b) one draw over 0 to 4 on the tick of the request the idle soldier accepts, and none on the
    tick of the request the walking soldier refuses. The idle soldier stands on a side position (W7
    part a), so the walking target that it chooses at once (claim 6) has a desired point of class
    right and takes no draw of claim 3. The same holds for the unit of W7 part n and for the
    occupants of *G*, which stand on side positions, in the references for W7 parts m to o.
  - (c) in (c1) two draws over 0 to 4 on the tick of the hit, besides the hit's own draws, and in
    (c2) three: one over 0 to 4, one over 0 to 3 and one over 0 to 4, in that order. The destination
    held at the end of that tick is the one that claim 12 gives for the value of the last of them
    (request B), around the cell *A* of W7 part (j), the cell of request A's walking target: the
    outcome set *H*₂ of that part. Request A takes the first of them, which does not decide its
    destination here: claim 13 takes that from the search, which finds a cell on this open ground.
  - (d) two draws over 0 to 4 on the tick of the hit, besides the hit's own draws, and the
    civilian takes no new destination and does not move on that tick.
  - (e) one draw over 0 to 4 on the tick of the hit, besides the hit's own draws: request B's.
    Request A is not sent, since the walking civilian's locomotor is moving and it has a
    destination (claim 15, second case).
  - (f) four draws over 0 to 4 on the tick of the scatter order: *A*'s, then one for each of the
    three occupants of *b*, which accept the request that *A*'s extra locomotion sends (claim 6,
    *Requests sent from an extra locomotion*). All four stand on side positions, so as in part (b)
    the walking targets they choose at once take no draw of claim 3.
- *Falsified if:* (a) a draw on another tick, or two on one asking tick. (b) no draw or two draws
  for the accepted request, or a draw for the refused one. (c) another count or order of draws
  besides the hit's own draws than the one expected in part c1 or c2, or a final destination that
  matches what claim 12 gives for the value of the first of the two draws, around *A*, but not for
  the value of the last, or a destination outside *H*₂. (d) fewer than two draws besides the hit's
  own draws (which would mean that a request that finds no destination takes no draw), or the
  civilian moves on the tick of the hit. (e) two draws besides the hit's own draws (which would mean
  that request A was sent to a walking unit and accepted), or none. (f) one draw on the tick of the
  scatter order and the occupants' three on a later tick. Part (f) does not tell the extra
  locomotion from *A*'s own locomotion of that tick (claim 6).

**Scenario W10: end of a walk and the approaching walker's stop (claims 16 to 18).**
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
  - (e) Separate runs under the common measurement setup, with no weapon overrides, game speed
    index 2. A tiny map has a straight high bridge over flat enterable clear ground, with
    approaches at deck level and a ground route beneath it. For e2, shape the access route
    so that the only route from the ground start to a bridge approach passes through *C*
    underneath the deck, then leaves *C*, reaches the approach and returns to *C* on the deck.
    An ordinary infantry unit of the
    human player starts at least three cells from an empty interior deck cell *C*: (e1) on
    the deck route, (e2) on the ground route under it. Give a move order whose applied cell
    destination is *C*, recording that cell to exclude click-height ambiguity. Record both
    the unit's world height and its locomotor destination point from assignment through
    every arrival and subsequent path computation, with its layer, remaining path steps,
    behaviour and speed fraction. If the ordinary click does not produce *C* in e2, a
    purpose-made map trigger may assign that same cell destination to the human unit.
    Record the assignment and do not substitute another cell silently.
- *Procedure:* (a) to (c): issue an attack order on the target; (b3) and (d): the move order. Record
  until the attacker has stood still for 30 ticks. One run per part, repeated on four map axes.
  In (e), record for 300 ticks after the first arrival in *C*, or until the destination is
  cleared; one run per layer from each bridge end. An e2 run tests arrival only when the
  recorded path actually reaches *C* on the ground without substituting the destination.
  A run whose search refuses it earlier records that separate outcome, not a false arrival.
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
  - (e) Let *H* be *C*'s centre terrain height. At assignment the locomotor destination
    is the centre at *H* + 414 leptons in both layers. In e1 the arrival height is *H* + 416,
    the destination clears on that arrival tick and the speed fraction is 0. In a qualifying
    e2 arrival the height is *H*, and the destination remains present on that arrival tick.
    With remaining steps it follows the access route and finally stops on the deck in *C*.
    If the path is exhausted the speed fraction is 0 nonetheless. It is not transferred
    onto the deck. If a later search fails with the unit still under the deck, the destination
    clears on that failed-search tick, by S-0212 claim 23, not by claim 16's completion test.
- *Falsified if:* (a) the walker stops before the recorded approach cell, or fires before arriving
  there. (b) the walker walks on past an arrival tick that is in range at the cell centre, stops on
  an arrival tick that is not, stops between two arrival ticks, loses its target, or fires while
  walking. (b2) it stops on the arrival tick on which only the target's own position is in range.
  (b3) it stops early. (c) it keeps a destination, or moves, after its first firing sequence starts. (d) it keeps a
  destination or a non-zero speed fraction after the arrival tick in the ordered cell, or the walk
  ends on another tick. A run in which S-0215 claim 37 renews the destination during the approach
  still counts; the parts test only when the walker stops. (e) either assigned height differs
  from *H* + 414, e1 retains the destination after arrival, a qualifying e2 arrival clears it
  by the completion test despite the 414-lepton mismatch, the unit is moved vertically to
  satisfy that test, or the specified later failed search keeps the destination.

**Scenario W11: teleporting infantry landing (claims 19 and 20; Q-0181, Q-0235).**
- *Setup:* a tiny flat clear skirmish map, a human player's infantry with the imported teleporting locomotor, one
  bystander computer player kept outside the observed area, game speed index 2, no engine add-on and
  no mods (the named test files of a scenario and the purpose-made test maps excepted). Use scripted orders and a read-only recorder for positions, destinations, occupancy
  and scenario stream state. Repeat each part with three recorded seeds. Void runs in which
  an unnamed bystander acts on a test object or enters the observed area.
- *Procedure:* (a) Order empty cell centres, then cells with one, two and three allied infantry
  side positions taken. (b) Order a vehicle-occupied cell and an impassable cell, leaving empty
  neighbours. (c) Use named test-map copies of infantry types whose imported properties allow them to enter a transport,
  capture, sabotage or be absorbed by a building, changing only their locomotor to the
  teleporting one. Give their corresponding object destinations and check the occupant-ignoring
  choice. Also use a blocked object destination in a behaviour outside those four and compare
  its search result with S-0212 claim 25. Repeat on a slope, a high bridge and with destination
  and departure in different reach regions, logging the inputs to claim 19's search. The named
  test data is removed afterwards. (d) Use a high bridge over enterable ground, with
  different allied infantry side positions occupied in ground and deck records. Vary
  departure elevation below and above claim 19's 312-lepton boundary using ground starts
  and elevated banks, without writing game state. Record the applied destination to
  exclude click substitution. Also use an empty ground record with all deck side positions
  taken: below the boundary the position choice can succeed but the entry check refuses.
  After each successful jump record deck status, listing and both occupancy records.
  Once phasing in ends, issue a player scatter and record the centre and occupancy
  conditions of the resulting destination choice. (e) Order across a cliff and to an isolated island.
  (f) Place the infantry with the teleporting locomotor by a production release or map-trigger placement
  with ground infantry occupants nearby. Identify any destination supplied during placement,
  then compare that event with a later ordinary move, for human and non-player owners where
  placement can be arranged. Do not manufacture a destination by writing game state.
  (i) On a level high bridge above enterable ground, start with an otherwise empty
  landing cell. Teleport there from ground height and record its ground and deck
  standing-position entries at fixation, after landing and after phasing in. Then
  order another teleport to an empty ground cell. Record both entries of the former
  landing cell through the next fixation and jump. Repeat with a deck-height
  departure. Use an ordinary ground walker under the former landing cell to test
  the residual ground position's availability. No recorder writes game state.
  (h) On flat ground give a move to empty cell *A*, then a replacement move to cell *B*
  with all three allied infantry side positions taken, using scripted ordinary input.
  Record each applied destination and movement update. A qualifying run must show *A*'s
  reservation followed by *B*'s refusal before any jump; two inputs that collapse to one
  destination or straddle a jump do not test replacement. Repeat with *B* impassable but
  its standing positions free, to distinguish entry refusal from position refusal.
  Seek three qualifying runs for each refusal kind; if normal input cannot arrange the
  interval, report that limitation rather than writing game state.
- *Observable:* landing cell, standing position and height, refusals, occupants' destinations,
  ground and deck records and listings at assignment and after the jump, the unit's deck
  status, and draws attributable to position choice or scatter rather than other actors.
  In h also record *A*'s released reservation and the departure-position mark through the
  next three movement updates.
- *Expected:* (a) a centre desired point gets only a free side position by table O, with its
  one draw when that row is reached; a fully occupied side record refuses without moving to
  an empty neighbour. (b) no fallback for a cell destination. (c) an enter destination meeting
  the exception can choose the centre without a draw; a refused object destination alone
  permits the nearby search, then the centre-desired position choice. (d) point selection
  reads and reserves ground below or at the strict height boundary and deck above it,
  while the independent initial entry check reads deck for every high-bridge cell.
  Thus a fully taken deck rejects the low-departure case even with free ground positions.
  Actual jump height and listing are on the deck in both successful cases, following
  S-0207 claim 20 e; later scatter uses deck status. (i) For ground-height choice,
  the ground position stays taken after landing and after the later successful
  destination choice and departure. Deck remains unmarked by this infantry. The
  ordinary ground walker cannot choose the residual taken position. For deck-height
  choice, the deck reservation remains after landing and is cleared on the next
  successful destination choice. (h) *A*'s reservation is removed,
  the departure-position entry is restored, the destination clears and no jump to *A*
  occurs, for both position refusal and entry refusal at *B*.
  (e) no connecting path is needed. (f) where placement supplies a destination, ground
  occupants receive the forced move-anywhere request, irrespective of owner; an ordinary
  later move does not reproduce that placement-only request. (g) On a named test map with
  impassable ground at levels −5 and −6 and free standing positions, repeat a direct
  teleport destination. Expect refusal at −5 and acceptance at −6, not a walking route. Remove
  the test map afterwards. This part tests the negative-level entry bypass.
- *Falsifiers:* a cell refusal landing in a neighbouring empty cell, a position not given by
  the stated table, an incorrect draw count, a fallback requiring a path, or an owner-only
  difference in placement clearing, or refusal of the −6 cell with a free standing position
  while the default quirk is on. In d, using deck for the low-departure position choice,
  reserving in a record different from that specified, or a successful jump leaving the
  infantry listed on ground falsifies the respective rule. In i, a transfer from
  ground to deck, a new deck mark at landing, or clearing the old ground mark on the
  next successful destination choice falsifies the ground-height lifecycle. In h, retaining *A* or its
  reservation, failing to restore the departure mark, or jumping to *A* falsifies the
  replacement outcome. A placement that supplies no destination is not evidence
  for or against claim 20.

**Scenario W12: candidate factory tether damage test (claim 15).**
- *Setup:* a tiny clear skirmish map, human-owned infantry factory and an infantry type it can produce, a
  bystander computer player far away, game speed index 2, no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted). Produce
  infantry with no rally point and then with a rally point, three recorded seeds each.
- *Procedure:* use scripted production and a read-only recorder as in S-0417 U10. Record
  contact, tether, current and queued behaviour, target, destination and movement in progress
  from release through the first cell arrival. Seek an interval in which the tether is the
  only failed condition of claim 15 request A's second case. Only if such an interval exists,
  use damage without a known attacker, with the player-scatter rule enabled by a named test-map
  override, and compare the same surviving unit after tether loss. Remove the override.
- *Observable:* which conditions coincide with the tether, and whether the hit sends an
  accepted scatter and a draw. Record health to exclude lethal hits.
- *Expected:* successful release contact tethers as S-0417 claim 10 states; if all the other
  conditions can be matched, damage causes no general scatter while tethered and causes it
  after tether loss. A tethered unit with a destination or movement in progress does not
  isolate the tether condition. The first phase is exploratory and cannot upgrade backing
  merely by finding a tether.
- *Falsifiers:* a successful clearance exchange without a tether falsifies the setup's
  premise; a general scatter while the tether is the only failed condition falsifies claim
  15. Failure to find the required interval leaves that condition unmeasured.

**Scenario W13: close-range movement within the current cell (claim 21).**

- *Setup:* a tiny flat clear skirmish map, one human-controlled house and one bystander
  computer player, game speed index 2, no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted). Use ordinary walking
  infantry with the close-range property described by S-0307. Pre-place the attacker at the
  top position and a legal enemy target in a neighbouring cell. Keep the observed area free
  of other actors. Use S-0307 RG23(b)'s successful-shortcut geometry and repeat with each side
  position occupied in turn. Repeat on a high bridge with different ground and deck occupants,
  including all three deck side positions taken while a ground side position remains free.
- *Procedure:* give an attack order with scripted input. In three runs per arrangement,
  record ticks, positions, walking target, destination, target and occupancy reservations
  through the first arrival and the next approach processing. A read-only recorder is
  proposed for points and destinations not distinguishable in a screen capture. Apply the
  glossary's bystander void rule.
- *Observable:* whether the first movement stays in the attacker's cell, its reserved side
  position, whether its destination names the target-reference cell, and the destination on
  arrival. Separate these events from later approach processing.
- *Expected:* on ground a successful shortcut walks to its selected side position while the
  destination is the target-reference cell. On arrival the destination is cleared exactly
  when claim 16 or 17 applies, not merely because the immediate point was reached. On a deck,
  the final walking target respects deck occupancy even though the shortcut read ground
  occupancy. A refused second choice gives no immediate walking target.
- *Falsifiers:* movement directly to the target-cell centre instead of the in-cell position,
  replacing the destination with the attacker's cell, ignoring deck occupants in the second
  choice, or completion on that arrival despite neither claim 16 nor claim 17 holding.

**Scenario W14: infantry entered-by timing (claim 22; Q-0268 part 1).**
- *Setup:* a tiny clear skirmish map with human-owned infantry with the ordinary walking locomotor and one
  bystander computer player kept away, speed index 2, no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted). Attach
  independent one-time any-house entered-by tags to successive intermediate and
  final cells. Each tag's action sets a distinct map variable. Include a level high
  bridge with tagged ground/deck passage. Repeat each arrangement three times.
- *Procedure:* order a walk past the tagged cells to a farther destination, then a
  separate walk stopping at the final tagged cell. Record the tick, physical
  position, walking target, occupancy reservation, destination and variable changes
  with a read-only recorder. Compare the asking tick, boundary-crossing tick and
  snap-to-standing-position tick. For the same-cell case use W13's successful
  close-range shortcut, with the attacker's current cell tagged. For teleport use
  W11's ground-to-deck setup with tagged departure-to-landing intermediate cells.
- *Observable:* the first tick each variable changes and the unit's position,
  height, destination and listing at that moment. If a destination-clearing arrival
  can be arranged under claim 17, use a trigger action whose result reveals the
  unit's destination or record it at the action's effect, not only at tick end.
- *Expected:* a walking arrival fires in the snap tick, never at reservation or
  mere boundary crossing. Intermediate cells fire even though the unit does not
  stop its walk there. A same-cell shortcut arrival also fires. Ground passage
  beneath a high bridge does not fire its cell tag, but deck arrival does. Where
  claim 17 clears the destination, the tag's action observes it already cleared.
  Teleport fires only at landing in the jump tick, not for intermediate cells or
  on ending phasing in, independently of the ground mark left by claim 19.
- *Falsifiers:* a variable change at reservation or boundary crossing before the
  snap, a missing eligible intermediate arrival, or a tag action preceding the
  stated destination clearing. A polled trigger firing later than the variable
  change is not a falsifier: record the entered-by action itself.

**Scenario W15: walking factory clear-report timing (claim 23; Q-0295).**
- *Setup:* a tiny flat clear skirmish map with a human-owned infantry factory,
  enough credits and prerequisites to produce ordinary walking infantry, one
  bystander computer player kept away, game speed index 2, no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted).
  Set a rally point beyond the exit cell. Observe the factory, its foundation and
  the full route to the rally point. Use S-0417 U10(a)'s ordinary release setup.
  If the imported release geometry gives only one walking-target arrival before the exit,
  repeat with a named test-map release-displacement override placing the unit
  at a legal appear position at least three cells from the factory's unchanged
  exit cell. Qualify this setup by recording successful placement, contact and
  tether; leave all other rules unchanged. Include a route with an intermediate standing position on the
  foundation if the type's legal placement and exit geometry permits it.
- *Procedure:* produce one unit and give no further orders. In three runs per
  arrangement, use a read-only per-tick recorder for position, cell, walking target,
  destination, factory contact and tether. Compare boundary crossing, arrival snaps,
  first departure from the foundation and reaching the exit. Apply the glossary's
  bystander void rule. A run without a successful contact/tether exchange cannot
  test report timing.
- *Observable:* the first tick contact and tether end, the standing position reached
  on that tick, and whether the exit destination is still farther away. Continue
  recording until the exit and rally legs complete.
- *Expected:* contact and tether end on the first standing-position arrival after
  tether establishment, not at its preceding boundary crossing. On the multi-cell
  release walk the unit still has farther to walk to the exit and continues there.
  If a foundation arrival is reached first, it reports there; leaving the foundation
  is not a prerequisite. No extra report is required at later arrivals once untethered.
- *Falsifiers:* loss of contact/tether merely at boundary crossing, retention until
  the exit despite an earlier ordinary arrival, a foundation-arrival suppression,
  or premature completion of the exit walk caused solely by this report. Failure
  to obtain a foundation arrival leaves that subcase untested, not disproved.
- *Measurement method:* scripted input and a read-only recorder; S-0417 U10(a) can incorporate
  these recordings without changing the oracle.

**Scenario W16: scatter from a cell with no reach region (claim 24; Q-1502 question 1).**

- Setup: a tiny skirmish map with an infantry unit with the ordinary walking locomotor map-placed on a
  cliff cell whose imported passability properties exclude it from the unit's movement zone, inside the usable area and outside every reach region of
  its movement zone. Provide empty, enterable ground neighbours in a reach region. Use no
  bridge, transport, tether or existing movement. Place the same type on nearby open ground
  as a control. Keep both houses alive independently. Use the common measurement setup, game speed
  index 2, director off and mods off. Qualify the fixture by recording the starting cell's
  lack of a reach region, the neighbours' reach-region membership and the absence of all
  reachability-gate exceptions in S-0212 claim 6. Repeat with the enterable neighbours on
  different sides of the cliff cell; the fixture is defined by these conditions, not by a
  particular existing map.
- Procedure: for the placement-only control, use a non-fraidycat type other than the cow
  and record the first 30 ticks without orders or damage to test whether placement alone
  assigns a scatter destination or consumes a scatter draw. Then, from fresh stationary
  placements, send the human scatter order to each unit
  separately (S-0213 claim 9). Repeat at eight successive request ticks in fresh runs. Also
  test the damage path in claim 15 with an infantry type whose imported weapon list is empty and a
  survivable hit. For a human-owned non-fraidycat, enable the player-scatter rule in a named
  test-map override and use a current behaviour with the scatter property to obtain request A.
  For request B, use a computer-owned fraidycat with that property and a known attacker.
  Distinguish its two requests using W9. Run each damage variant three times per cliff
  arrangement. Remove the test-map override afterwards. Retain
  only runs in which the recorder confirms an accepted request and a chosen
  destination on the surrounding usable ground. Record separately whether the request has
  a threat point. Do not infer request acceptance from lack of motion. Give a direct move
  order to the same neighbouring destination in a separate fresh run.
- Observable: a read-only recorder records the request options, chosen destination, its
  assignment and clearing ticks, and the unit's cell and world position each tick for
  300 ticks after the request. Video records whether the unit ever leaves the cliff.
  Exclude runs in which death, mind control, transport or another order interrupts the test.
  A recording harness can host this test; support for distinguishing request acceptance
  from route failure is a proposed harness addition, not a modification here.
- Expected: placement alone gives no scatter destination or scatter draw. After an accepted
  request, destination selection can succeed for the cliff unit, but normal pathfinding
  rejects a destination in a reach region and takes no walking step. Record whether the
  destination is retained or cleared separately from that result. The direct move control also fails; the open-ground scatter control walks normally. A cliff unit
  taking a step to such a destination after a confirmed scatter, without another movement
  cause or a reachability exception, falsifies the claim. An unaccepted request or a search
  that finds no destination is inconclusive, not confirmation. Arena victory timings alone
  do not identify an escape path. A scatter destination or attributable scatter draw during
  the undisturbed initial interval falsifies the placement-only statement; an unrelated idle
  action or externally supplied order makes that interval inconclusive instead.

**Scenario W17: destinations given during an update, and a new destination during a step
(claim 6; Q-2657, Q-2701, Q-2702, Q-2746).**
- *Setup:* M-0023's map and settings, stock data. The recorder reads at the end of each tick, for
  the test unit, its current and queued behaviour, destination and position. Unit *I* is an ordinary
  infantry unit of a human-controlled house, resting in *guard* on open, flat ground with no other
  unit within 3 cells.
- *Procedure and expected results:* (a) Give *I* a move order to a free cell 3 cells away, taking
  effect at the start of a tick *T*. Expected: *I*'s position first changes in tick *T* + 1.
  Falsified by a first change in tick *T* + 2 or later. (b) S-0417 scenario U10 part b's set-up on
  this map, with every cell that the nearby-cell search could give held and one neighbour held by
  allied infantry with a side position free, as in W7 part k, so that the factory's request takes a
  neighbour. Read the tick *T*' at whose end *I* first holds a destination. Expected: the first
  change of position in tick *T*' + 2. Falsified by tick *T*' + 1. (c) Order *I* to a cell 6 cells
  away in a straight line. While it is between its first and second standing positions, at the start
  of a tick, order it to the centre of M-0023's walled pen. The harness gives this order directly,
  as in M-0023, not through a player's input: an order given at the interface would have the pen's
  cell replaced by a cell in reach (S-0214 claim 11). Expected: it finishes the step under way,
  arrives, has no destination at the end of the tick after that arrival tick and moves no further.
  Falsified by any further step towards the old cell. (d) As (c), the second order to a cell on
  *I*'s straight line beyond the cell it is stepping to. Expected: the walk continues to that cell,
  with one asking tick after the arrival tick. (e) As (c), the second order to the free cell *I* is
  stepping into. Expected: *I* arrives there, has no destination at the end of that arrival tick and
  moves no further. Falsified by a further step or a destination held after that tick.
- *Stored path kept under enter (Q-2701):* an empty friendly infantry transport *V*, of a type that
  accepts *I* as a passenger, stands at rest 2 cells from *I*'s start, on the backward extension of
  the line that part c orders it along. (f) As (c), the second order an enter order on *V*.
  Expected: *I* finishes the step under way and goes on along its line to the cell of the first
  order (its stored path is shorter than a path can hold), with *enter* queued or current, and only
  then walks back towards *V*. Falsified by a step towards *V* before *I* reaches the end of its
  line. (g) On a purpose-made map of open ground, with *V* placed in a straight line from *I*
  farther away than a stored path holds steps (S-0212 claim 20), order *I* into *V*. While it is
  between its first and second standing positions, at the start of a tick, order it to a free cell 3
  cells to the side of its start. Expected: its first step after the step under way is along its
  line towards *V*, and the recorded behaviours show when *move* becomes current. Falsified by a
  first step towards the new cell right after the step under way. How far it goes on along the line
  is recorded without an expected value (claim 6, low). (h) As (f), with the enter order given at
  the start of the tick after *I*'s first arrival tick, when it stands with no walking target and a
  stored path. Expected as in (f). Falsified as in (f).
- *Requests from an extra locomotion (Q-2702):* (i) Infantry *A* stands on a side position of cell
  *a*. A neighbouring cell *b* holds three allied infantry of the same house on its three side
  positions, resting in *guard*, so that *b* gives *stationary friendly* to a walker from *a*
  (S-0209 claim 4, row WR4). Every other neighbour of *a* is of a land type that infantry cannot
  enter (as in W9 part d), so every path from *a* leads through *b*, and the cells beyond *b* are
  free. Give *A* the human scatter order (S-0213 claim 9) at the start of a tick *T*. Keep only runs
  in which *A*'s recorded destination is at least S-0209's close-enough distance (Formulas F1) away.
  Expected: at the end of tick *T* the three occupants of *b* each hold a destination (claim 13), so
  the request of S-0209 claim 8 was sent in tick *T*. The run does not tell whether it was sent from
  *A*'s extra locomotion or from its own locomotion of that tick (claim 6). Falsified if they first
  hold destinations at the end of tick *T* + 1 or later, or not at all.
- *Garrison order and the stored path (Q-2746):* *I*'s type is one that can garrison (S-0214 row
  I8), and an empty building *G* of a type that *I* can garrison under S-0508 claim 1, owned by the
  passive neutral house, stands where part f puts *V*: on the backward extension of part c's line,
  its nearest foundation cell 2 cells from *I*'s start. The harness gives each garrison order
  directly as a *capture* order with *G* as destination, as row I8 gives. (j) As (c), the second
  order a garrison order on *G*. Expected: *I* finishes the step under way, and its next step is
  back along its line towards *G*, with *capture* queued or current, and it then garrisons *G*.
  Falsified by a step on along the line away from *G* after the step under way. Part f, run on the
  same layout with *V* in *G*'s place, is the control: there *I* goes on along its line. (k) As
  (g), with *G* in *V*'s place and a garrison order in place of the enter order. Expected: its first
  step after the step under way goes towards the new cell, and the recorded behaviours show *move*
  becoming current with the new cell as destination. Falsified by a first step after the step under
  way along its line towards *G*.

## Open questions

- A walker that already has a walking target and accepts a scatter whose destination comes from the
  nearby-cell search (such as the requests without a threat point of W7 part r) has the same extra
  locomotion in the original: the scatter starts it whatever the walker's state (medium: the start
  of the extra locomotion at a scatter was traced without a condition on the walker's state, and no
  scenario here gives such a scatter to a walker with a walking target). What that locomotion does
  where a step is under way was not traced, so there is no traced behaviour to copy. Our engine
  gives such a walker no extra locomotion for now: it keeps its walking target and steps as in claim
  6, a provisional rule until that is traced (low). The same holds for a walker under *enter* that
  is given such a scatter in the middle of a step (claim 6, *A stored path kept under enter*).
- A walker asked again while a locomotion of its own is under way (claim 6, *Requests sent from an
  extra locomotion*): whether a legal setup exists in which the original's nesting never ends was
  not shown (low). One would need two cells, each with three still occupants that refuse every
  request and one such fraidycat whose new paths lead through the other cell. Our engine's bound
  makes the question matter only for matching the original's finite nestings.
- The screen-cover check (Terms) is exact as arithmetic, but three parts of it are not confirmed:
  the property of *C* that, in the original, lets a high-bridge cell count 4 levels higher was not
  identified, and our engine raises such a cell whenever it is a high-bridge cell (Terms); a cell
  outside the map is taken to read as height level 0, which matters only on a map with negative
  levels; and that the check matches what a viewer sees hidden on the screen is an interpretation.
  Scenario W7(f) tests the rule on uneven ground; W7(b) relies only on flat ground, where every cell
  passes.
- Infantry factory release gives a tether situation (S-0417 claim 10), but it has not been
  established that a released infantry unit can be hurt while tethered, stationary and without
  target or destination in a behaviour with the scatter property. W12 checks that conjunction.
  Until it does, the damage-scatter tether condition of claim 15 remains unmeasured.
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
- No other factor (for example a game-wide speed bias of the movement specs) is known to scale a
  walking infantry unit's speed during a step, and F5 has none; that none exists is not confirmed.
- The height of a walker between standing positions, while it steps across a slope or onto a bridge
  deck, was not determined. The step of claim 6 changes only the map position, and no rule was
  found that sets the height to S-0106's surface height on every step, so the original's walker may
  keep the height of the position it left until it arrives, which is what our engine does (claim 6).
  Scenario W2(e) records the height on every tick of its walks, without an expected value; a
  measured value replaces our engine's choice.


## Changes for other specs

- **S-0212, claim 20.** Its "when its destination changes" holds for the walk locomotor only once
  the step under way has ended: a walker with a walking target finishes that step and first searches
  for the new destination in the tick after its arrival tick (S-0208 claim 6). The same holds for
  its State and ordering row "| any | destination changes | stored path cleared, fresh search.": for
  the walk locomotor the fresh search waits until the step under way has ended. With
  `quirks.enter_keeps_stored_path` on, the stored path is not cleared for a walker whose current or
  queued behaviour is *enter* and that is in contact with nothing at the change, with or without a
  walking target. Such a walker follows the old path and first searches for the new destination
  when a step on it is blocked or no stored steps are left (S-0208 claim 6, *A stored path kept
  under enter*). The bullet and the row should say so for the walk locomotor.
- **S-0209, claims 6 to 9.** Claim 6's "The walker stops for that tick and asks again on the next
  one", for a *clear* cell with no free standing position, claim 9's "On the repeated ask it waits
  for the rest of that tick", and claims 7 and 8's "waits for the rest of that tick", after claim
  8's request and after a repeated ask that gives *blocked*, hold for an ask made in the walker's
  own locomotion. A walker that asks in the extra locomotion of a scatter that comes before its own
  locomotion in the tick and is left with no walking target asks again in its own locomotion, on the
  same tick, as a first reaction (S-0208 claim 6, *Requests sent from an extra locomotion*, medium).
  The same holds for S-0209's State and ordering rows for *clear, no position free* ("stops this
  tick, asks again next tick") and for the repeated asks that give *stationary friendly* and
  *blocked* ("waits for the rest of the tick; on the next tick it asks again as a first reaction").
  The four claims and those rows should say "until the walker's next locomotion" (claim 6: "asks
  again at its next locomotion") or point to S-0208 claim 6 for that case.
- **S-0209, Terms, *first ask* and the two cells of an asking tick.** "On one asking tick the entry
  outcomes of up to two cells decide what the walker does", with the first ask the next cell of "the
  path it holds when the tick begins", holds for one locomotion. A walker whose extra locomotion at
  a scatter (S-0208 claim 6) comes before its own locomotion in the tick makes the asks of each
  locomotion on that tick: in the extra one its first ask is the next cell of the path it holds
  after the scatter (the one its search there gives, or the stored path it keeps), and in its own
  one it is the next cell of the path it then holds, so the
  outcomes of up to two cells per locomotion decide what it does on that tick (S-0208 claim 6,
  *Requests sent from an extra locomotion*, medium). The term should say "per locomotion" or point
  to S-0208 claim 6 for that case.
- **S-0209, Terms, *Asking tick*.** Its sentences "The walker does not move on the tick on which it
  chooses a walking target, and takes its first step towards it on the next tick (S-0208 claim 6).
  So at a free cell boundary the asking tick is the last tick before the walker's position changes
  again." hold for a walking target chosen in the walker's own locomotion. A walking target chosen
  at once at a scatter that comes before the walker's own locomotion in the tick is stepped towards
  in that locomotion, so the walker moves on that tick (S-0208 claim 6, *Destinations given during a
  unit's update*, medium). The term should limit the two sentences to a walking target chosen in the
  walker's own locomotion and point to S-0208 claim 6 for the scatter case.
- **S-0417, its Changes entry for S-0208.** Its sentence "The immediate update is asked for the
  teleporting unit only: for a walking unit this spec relies on no such update, and S-0208 claim
  15's rule that the walking target is chosen only at the next walking step stays as it is" no
  longer holds in its last part. A scatter whose destination comes from the nearby-cell search gives
  the walker its walking target at once (S-0208 claim 6), and S-0208 claim 15 gives request B's
  scatter centre cell as the cell of request A's walking target. S-0417 still relies on no immediate
  update of its own for a walking unit. The entry's last part should point to S-0208 claims 6 and 15
  instead.
- **S-0417, claim 17.** The scatter request that the factory sends to a released walker, when its
  destination comes from the nearby-cell search, is searched for in the request's own turn, which is
  the asking tick, and the first step comes in the next tick (S-0208 claim 6). The claim should give
  that absolute timing as well as the relative one. Its sentence "the original's order within the
  turn is `unmeasured` (Open question 11)" and Open question 11's "Where the request falls within
  the turn, relative to the behaviour processing, its draws and the locomotor's progress, is not
  established for the original" can cite S-0208 claim 6 for the request's place after the walker's
  behaviour processing and locomotion in the original (medium). S-0208 claim 6 does not place the
  request relative to the late start, so that part of Open question 11 stays open.