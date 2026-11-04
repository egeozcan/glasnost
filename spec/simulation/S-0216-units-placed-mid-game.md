---
id: S-0216
title: Units placed after the game has started
topic: simulation
revision: 69
sources-class: [decomp, measurement, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-04
measurements: [M-0001, M-0002, M-0003, M-0006, M-0017, M-0019]
requests: [Q-1711, Q-1713, Q-2096, Q-2098, Q-2099, Q-2277, Q-2336, Q-2446, Q-2447]
supersedes: []
---

# S-0216: Units placed after the game has started

## Summary

What placing a newly created vehicle or infantry unit on the map does once the game's set-up has
ended: the facings it starts with, where an infantry unit stands in its cell, which random draws the
placement takes and in which order, the entry test that can refuse it, and which behaviour is
current before the unit's first update. It answers Q-1711 questions 2 and 3 for units placed by a
play-time placement. Question 5, how the measurement records' units were placed, is answered in
part: M-0006 shows that they were in play from tick 0 and where the archived logs place them,
and M-0019 that their standing positions in the all-infantry games are claim 2's and that the raw
values before tick 0 fit claim 3's draws (claim 7).
Questions 1, 4 and the rest of 5 need an oracle run (Open questions). It answers
Q-1713 in part (claim 6). For question 1, in the records' games no unit was placed on the cliff
cells, since every unit stated on one was placed on a neighbouring cell (M-0006), so the
request's premise does not hold for them. What ends those games stays open (Open questions).
Questions 2 and 3 are answered for an infantry unit that stands on such a cell and whose house
is one of the original's computer players, a placement that did not occur in the records' games.
Claim 8 gives our own arena's set-up rules (design), with M-0006's outcomes as the acceptance
check. It answers Q-2096 in part: where the arena puts a unit that its stated cell does not
take, and question 5 (which units stood off their stated cells in the records' games) only as far
as M-0006 does. It answers Q-2098 in part: what our arena loads on the bunker cells, while the
bunkers' kind and their fate in a stock game stay open. It answers Q-2099 in part: how our arena
settles exact halves in its group counts, while question 1, what the records' harness gave, stays
open. It answers Q-2277 question 1, except for several infantry units on one cell, and question
2: the records' infantry stood at the standing positions of claim 2 in the all-infantry games,
each at its own draw's, and in the other games, at tick 0, at positions of S-0208 table P, never
the centre,
and the totals of raw values count, before each unit, one try's for each cell of claim 8's
order before its own, as
claim 8 takes them (M-0019, the tries' attribution to cells inferred). Where several infantry units
on one cell stood in the records stays open: M-0019 gives no finding of its own for that case,
and PT2 part b tests claim 2's occupancy part (`measurable`, medium). Q-2277 question 3 stays open.
Placement by the map during the
game's set-up is S-0009's and S-0208's (claim 5), and is restated here only as a contrast.
Skirmish only: campaigns and network games are out of scope.

## Terms

Shared terms used here are defined in the glossary: *infantry*, *vehicle*, *type*, *field*,
*house*, *tick*, *facing* (with its facing units), *cell*, *usable cell*, *scenario random
stream*, *behaviour*. S-0202's ranged draw R(*a*, *b*) (S-0202 Terms) applies. A cell's
*vehicle-present* and *object-present* flags are those of S-0209 Terms, and an *open gate* is as
S-0208 claim 3 reads it. In addition:

- **Play-time placement.** Putting a newly created vehicle or infantry unit on the map, at the
  centre of a stated cell on the ground of that cell (for an infantry unit the centre is the
  desired point of claim 2, not where it stands) and with a stated facing value *v* (an integer
  0 to 255), after the game's set-up (S-0009) has ended, by an ordinary placement: one that
  respects the standing-position occupancy of the cell and applies the play-time entry test
  (claim 5). The measurement harness of this spec's scenarios (PT1 to PT5), which adds units to
  a running game, is expected to use it, and so can another rule of the game where its own spec
  says so.
  Rules that put units on the map during play with their own desired point, cell choice and draws
  are not play-time placements in this spec's sense, though several use the same table of
  standing positions: passengers coming out of a destroyed or sold absorbing building (S-0312
  claim 24, S-0408 claim 8), the survivors of a sale (S-0408 claim 7), a transport letting out its
  passengers (S-0507 claim 10) and a paratrooper's drop (S-0522 claim 9). Their specs give what
  each does. A map entry's placement, which happens during set-up (S-0009 claims 38 and 39), is
  not a play-time placement either. Some rules of the game place a unit during play with that
  occupancy ignored, as set-up does: putting garrison occupants out (S-0508 claim 11) and placing
  a released parasite on its host's cell (S-0614 claim 15). Those units stand where those specs
  say, with no standing-position draw, and are not play-time placements in this spec's sense. A
  released parasite placed instead in the neighbour cell, and a returning parasite, make the
  position choice and draw of S-0614 claims 15 and 20.
- **Placing step.** Whatever performs a play-time placement. Its outcomes are a new unit (with
  S-0202 claim 10's creation draw), that unit on the map when its placement succeeds, and
  possibly a behaviour given to it once it stands there.

## Behaviour

1. **Vehicle facings.** A vehicle placed at play time with facing value *v* starts with its body
   facing and its turret facing both exactly 256 × *v* facing units, set at once with no turn in
   progress (S-0206 claim 5). S-0206 Terms gives this conversion for a pre-placed vehicle (S-0009
   claim 39) and leaves other placing events to their own specs; this claim is that rule for a
   play-time placement, the same conversion as for a pre-placed vehicle. Nothing about it
   depends on the game's seed: two games that place the same vehicle with the same *v* start it
   with the same facings. The vehicle keeps
   them until a later rule turns or sets them (S-0206). What placement does to the barrel elevation
   is S-0105's (claim 6) and is not part of this claim.
   - Scope: the claim covers a vehicle put on the map by a placing step. A vehicle leaving a
     transport is S-0206 claim 5's case, which rates as medium that a vehicle leaving a transport
     is placed; this claim does not raise that rating.
   - Backing: `measurable` (scenario PT1) for the facing values PT1 places, which include values
     that are not multiples of 32 and so tell the rule from a rounding to 8 directions.
     `unmeasured` for the other values.
   - Confidence: medium for placement by a placing step at the facing values PT1 places, until
     PT1 runs, since no other spec gives the conversion for a play-time placement; low for the
     other values (`unmeasured`).

2. **Infantry standing position.** An infantry unit placed at play time at the centre of a usable
   cell, on the ground, does not stand at the centre. Its placement chooses a standing position
   exactly as a walker whose desired point is the cell's centre does in S-0208 claim 3: the row of
   the claim's table for class *centre*, with one ranged draw R(0, 3) from the scenario random
   stream, and the first free position of row R*k* of S-0208 table O, where *k* is the draw. The
   cell's occupancy record is read as S-0208 claim 3 reads it, so units already standing in the
   cell count. That includes the first row of that claim's table: when the cell's ground record
   has its vehicle-present flag on, or its object-present flag on with no open gate in the cell,
   no position is chosen, no draw is taken and the placement fails, so the unit is not put on the
   map. When the draw is taken but no position of row R*k* is free, the placement fails in the
   same way, after the draw. A placement whose position is chosen can still be refused by the
   entry test of claim 5, after the draw. What the placing step then does with the unit belongs to
   that step (Open questions). S-0614 claim 20 applies the same table in the same way to the
   placement of a returning infantry parasite, with the requested centre as desired point.
   - In an empty cell the unit stands at the right position with probability 1/2 and at the left
     and bottom positions with probability 1/4 each. It never stands at the centre or top
     position.
   - For a cell outside the usable area, the standing position and whether a draw is taken are not
     established here (`unmeasured`, low); PT2 part c records them. The unit is put there only if
     the entry test of claim 5 accepts it. S-0209 claim 4, part 4a blocks cells outside the usable
     area for a unit that is inside it; whether a unit not yet on the map counts as inside is not
     established, so the entry test may refuse every such placement. S-0522 claim 9 takes one R(0,
     3) for a paratrooper's standing position outside the usable area, but that is the landing of a
     drop, which tests the usable area after its own selection; it does not settle the case here.
   - A map entry placed during set-up is different: it stands at the standing position its entry
     names and takes no placement draw (S-0208 claim 5; S-0202 claim 10, map-loading table).
   - The choice for a unit placed above the ground of its cell (on a bridge deck) is not
     established (Open questions).
   - Backing: `measured` (M-0019) for the usable-cell rule in the placements that put the records'
     units on the map (claim 7) in the all-infantry games: in all 192 games of M-0002 with infantry
     on both sides, every
     placed infantry unit stands at tick 0 where this rule puts it for its draw R(0, 3), the last
     of the scenario stream's raw values that its draws take (claim 3), at the right, left or
     bottom position and never at the centre or top position. In the records' games with
     vehicles every placed infantry unit stands at a position of S-0208 table P, never the centre,
     and the raw values match claim 3's draws by their count only: that each unit stands at its
     own draw's position is a reading there (claim 7). `measurable` for other placing steps
     and for the occupancy record's part (units already standing in the cell), on which M-0019
     gives no separate finding
     (scenario PT2 parts a and b for usable cells, part d for a cell holding a vehicle, part e for
     a full row). The object-present refusal and its open-gate exception are `unmeasured` here, as
     in S-0208 claim 3: no part of PT2 isolates them, and M-0019's placements do not show them. The
     cell outside the usable area is `unmeasured`: a placement there that is refused (S-0209
     claim 4, part 4a) shows no position, and whether one is refused is not established. PT2 part
     c records the outcome, including whether a draw is taken.
   - Confidence: high for the usable-cell rule (the draw and the rows of table O) in the
     placements of the records' units in the all-infantry games, as M-0019 measures it. Medium in
     the records' games with vehicles, where the match of each position to its own draw is a
     reading. Medium for the occupancy record's
     part until PT2 part b runs. Medium for other placing steps until PT2 runs: they share the
     rule, but only the records' placements and S-0614 claim 20's returning parasite show that a
     placement makes this choice. Medium for the failed placements and for the object-present
     refusal and its open-gate exception (as in S-0208 claim 3); low for the cell outside the
     usable area.

3. **Draws of a play-time placement, and their order.** A play-time placement takes, at the
   moment of placement, these draws from the scenario random stream, by kind of unit:
   - an infantry unit: the standing-position draw R(0, 3) of claim 2, when its cell is usable and
     its ground record has neither its vehicle-present flag on nor its object-present flag on
     without an open gate (claim 2). The draw is taken even when the row then has no free
     position and the placement fails, and is kept when the entry test then refuses (claim 5).
   - a vehicle of a type with either visceroid flag (S-0202 Import mapping): one ranged draw
     R(0, 29), the same placement draw that S-0202 claim 10 gives for a pre-placed vehicle of such
     a type, taken
     only once the entry test has accepted the vehicle (claim 5).
   - any other vehicle: no draw.

   No other draw of the placement's own choices is known. Each placed unit's creation draw (S-0202
   claim 10) comes before its placement draws, at its creation. A unit whose creation also makes
   other objects, its initial slot objects (S-0506 claim 1) and any initial slave group (S-0402
   claim 19), takes their creation draws as part of its creation, after its own and before its
   placement draw, in S-0611 claim 14's order. So a placing step that creates and places units
   one at a time takes, per unit and in its placing order: the creation draws, then that unit's
   placement draw if any. Every one of these draws falls before the first tick at which
   the placed units are updated when the step runs between ticks, and they shift every later draw
   of the scenario stream. The land recompute that putting the unit into its cell makes (S-0211
   claim 5) keeps its own effects and draws, which this claim does not restate.
   - The records' units (M-0019). M-0019 counts the scenario stream's raw values (S-0202), not
     draws. A creation draw (S-0202 claim 10) and the draw R(0, 3) (S-0202 claim 4) take exactly
     one raw value each, and a draw R(0, 29) as many as S-0202 claim 4 takes at its place. So where
     a run holds only creation draws and draws R(0, 3), they are as many as its raw values, in the
     same order. Before tick 0 of the records' games, the raw values after the game-start ones
     (claim 8's acceptance item) are as many as this claim's draws take for every object created,
     each of the set-up's objects left off the map counted as claim 8 counts a refused try. In the
     all-infantry games each infantry unit's draws took consecutive raw values, before the next
     unit's, and its standing position is given by the draw R(0, 3) at the last of them (claim 2).
     In the games with vehicles M-0019 compares that count only. Some games hold spawned objects
     (M-0019's class: objects of a type no set-up entry names, created after the set-up's first
     object, off the map at tick 0). That these are objects made by a unit's creation, S-0506's
     slot objects or an initial slave group, is a reading, not identified by the recording, and
     M-0019 relies on it for no count: its count takes the made objects' creation draws from the
     creating unit's type. M-0019 does not attribute raw values to vehicles ("Which object took
     which raw value is not identified in the games with vehicles"). In the all-infantry games the
     placement draw of the last object placed takes the last raw value before tick 0 when that
     object is an infantry unit.
   - Backing: `measured` (M-0019) for the records' units. In all 1860 replays the count of raw
     values from the seeding to the first tick equals the game-start count (claim 8's acceptance
     item, *N*₀, found per seed in the all-infantry fits and taken by seed in the games with
     vehicles) plus the raw values of this claim's draws, summed over every object created, the
     objects that refused cells leave off the map counted as claim 8's tries. In the 192
     all-infantry games a model with this claim's draws per object, consecutive and in creation
     order, fits every standing position with one start only, the draw R(0, 3) at the last of each
     object's raw values giving the position, and the last object's last raw value is the last
     before tick 0. A model with one raw value fewer, or one more, for every object fits in no
     game. The recording does not show which raw value each creation or placement took. That an
     infantry object's raw values before its R(0, 3) are its creation draws is inferred from the
     counts and the fit. M-0019 classes the spawned objects by creation order only; their being
     objects made by a unit's creation is a reading, on which no count depends. `measurable` for
     the visceroid draw and its count of raw values (scenario PT3's second run, and PT5 for its
     place after the entry test): M-0019 does not attribute raw values to vehicles. `measurable`
     for other placing steps (scenario PT3).
     Where the made objects' draws fall, after the unit's own creation draw and before its
     placement draw, follows from S-0506 claim 1 and S-0402 claim 19, which make them at the unit's
     creation. It is not measured here (the records' totals do not order them). It is `measurable`:
     for slot objects by scenario PT3's third run, for an
     initial slave group by its fourth run.
   - Confidence: high in the all-infantry games of the records: each infantry object's draws on
     consecutive raw values in creation order, the standing-position draw last (M-0019). High that
     in every game the raw values after the game-start ones are as many as this claim's draws take
     for every object created, as claim 8 counts a refused try (M-0019). Medium that the spawned
     objects are objects made by a unit's creation, a reading. Medium for the order of these draws
     per object in the games with vehicles, where only the count is compared. Medium that an
     infantry unit's raw values before its R(0, 3) are its creation draws (inferred).
     Medium for other placing steps until PT3 runs, as claim 2 rates the rule there. Medium for
     where slot objects' draws fall, until PT3's third run, and for where an initial slave
     group's fall, until its fourth run. Medium for
     the visceroid draw at a play-time
     placement and for its coming only after the entry test has accepted the vehicle, until PT3
     and PT5 run: no other spec states this order (S-0009 claim 46 gives the set-up draw, where
     the test is skipped, as `unmeasured`, and rates medium that it is the only draw of a
     vehicle's placement); PT5 part b, a refused vehicle that takes no draw, tests it. Medium
     for there being no other placement draw.

4. **Behaviour before the first update.** Placement makes the unit fall back to rest in S-0215's
   sense (Terms, the *placement fall-back*), choosing a resting behaviour (S-0215 claims 14 and
   15). What can be observed once the unit is placed and before its first update: a unit that is
   ready to change (S-0215 claim 11) when it is placed has that resting behaviour current and
   nothing queued; a unit that is not ready when it is placed has it queued. S-0215 claim 14
   makes each resting choice a queue without an immediate start, so for a ready unit this rule
   goes beyond S-0215: it is an extension of S-0215 claims 14 and 15 that PT4 tests. The
   not-ready case agrees with S-0215's text on placement:
   - Its paratrooper case (claim 4) says that placement already queues the resting behaviour:
     falling makes that unit not ready, so the behaviour stays queued.
   - Its test of the placement fall-back expects *harvest* queued on placement for an ore
     harvester built by a factory: a vehicle in contact with a vehicle-factory building whose
     queued behaviour is neither *move* nor *enter* is not ready (S-0215 claim 11), so a
     vehicle placed by its factory is the not-ready case.
   - S-0215 claim 11's newly created actor that starts with no behaviour is the unit before its
     placement.

   S-0215 gives such a readiness test for a building (claim 1), but there only for a building it
   finds not ready: at set-up the test leaves *guard* queued, and the ready case is a set-up case
   that a skirmish never runs. S-0215 claim 1 says of that case that for a set-up building of a
   human-controlled owner the original would run the grand opening before *guard* is queued, so
   placement's readiness test would already make *guard* current. No other spec gives the ready
   case for a unit: it rests on analogy
   with that readiness test of S-0215 claim 1, whose ready case is `unmeasured` there, and on no
   measurement, which is why it is rated low below until PT4 runs. S-0215 claim 11's fall-back
   to rest at the first per-tick start is for an
   infantry unit with neither a current nor a queued behaviour; a placed unit has the queued
   resting behaviour, made current by the placement when the unit is ready, so that fall-back does
   not arise for it, and S-0215 claim 11 gives no such fall-back for a vehicle. A behaviour the
   placing step then gives the unit as a queued behaviour (for example *hunt*) is queued by
   S-0215 claim 7
   without being started. The unit makes it current at the first update at which it is ready to
   change (S-0215 claim 11) and no exclusion of S-0215 claim 2 (tunnel, erasing, leaving stage)
   holds:
   - ready at that update's first per-tick start: the behaviour is processed in that same update
     (S-0215 claim 3 (a)). For a unit placed between ticks and ready at its first update, the given
     behaviour's first processing comes at that update, as if it had been started with its
     processing timer expired.
   - ready only at that update's late start: the behaviour becomes current there and is first
     processed at the unit's next update (S-0215 claims 3 (b) and 11).
   Which resting behaviour the fall-back chooses, and so which one PT4 expects to be current,
   is S-0215's (claims 14 and 15).
   - Backing: `measurable` (scenario PT4)
   - Confidence: low for the resting behaviour current at placement for a ready unit, until
     PT4 runs (an extension of S-0215, as above); medium for the not-ready case, as S-0215's
     placement text has it; high for when the given behaviour is first processed once it is
     current, which is S-0215's rule

5. **The entry test.** A play-time placement also applies the unit's entry test to its cell, as
   S-0614 claim 20 states for a returning parasite: the unit's entry outcome for that cell
   (S-0209 claims 4 and 5, with the terrain rules of S-0211 claim 10), asked without an approach
   direction or source elevation and read against the cell's ground occupants and occupancy. Only
   *clear* accepts. Any other outcome refuses: the placement fails and the unit is not put on the
   map. So, for example, a vehicle that cannot cross water is refused on a water cell, and an
   ordinary vehicle on a cell holding a normal building, a tree or an intact wall that it does
   not crush (S-0209 claim 5: rows VO4 to VO6 for the wall, row VP29 for the tree). The test
   applies to every play-time placement, of a vehicle or an infantry unit, inside or outside the
   usable area. During set-up it is skipped (S-0009 claim 46).
   - On a cell whose land type is *rock*, an ordinary play-time placement's test refuses an
     infantry type whose movement class cannot enter *rock* (S-0211 claim 10). Whether the
     measurement records' units were placed this way is open (claim 7).
   - For an infantry unit, the standing-position choice of claim 2 comes first. When that choice
     takes its draw and finds a free position, the draw is kept even if the entry test then
     refuses, as S-0614 claim 20 says for a returning parasite.
   - For a vehicle of a type with either visceroid flag, the placement draw R(0, 29) of claim 3
     comes only after the entry test has accepted it, so a refused vehicle takes no placement
     draw.
   - What the placing step does with a refused unit belongs to that step (Open questions, *A
     failed placement*).
   - Backing: `measurable` (scenario PT5, for refusal on water, on an intact wall and, for an
     infantry type whose movement class cannot enter *rock*, on a *rock* cell, and the draws). The
     refusal on *rock* is `measurable` only on PT5 part d's flat *rock* cell: its cliff-cell
     fallback does not tell *rock* from the height step. The other entry outcomes rest on S-0209
     and S-0211 as cited.
   - Confidence: medium for the clear-only acceptance and the refusals on water and on a wall
     that PT5 tests, until PT5 runs: S-0614 claim 20 states the test only for a returning
     parasite, and S-0209 rates the wall and tree rows medium. Medium for the visceroid draw's
     coming after the test, as claim 3 rates it. Low for the refusal on a *rock* cell of an
     infantry type whose movement class cannot enter *rock*, until PT5 part d runs: it rests on
     S-0211 claim 10's movement class alone. M-0006 does not raise it: no unit of the records was
     placed on one of Q-1502's *rock* cells (claims 6 and 7), but units stated on other nearby
     cells were moved too, and M-0006 does not record why.
     Medium for the standing-position draw kept after a refusal, as claim 2 rates failed
     placements. Other outcomes as S-0209 rates each of them, and medium for the entry-table
     application to uncommon building and crushing exceptions, as in S-0614 claim 20.

6. **Units on cliff cells of no reach region (Q-1713).** The request concerns games of the
   measurement records M-0001 and M-0002 in which infantry units were meant to stand on cliff
   cells of no reach region whose land type is *rock* (Q-1502). In those games both houses were
   the original's computer players (M-0001), so every rule the original gives only to
   computer-controlled owners would apply to a unit of theirs, retaliation included.
   - Question 1. No unit was placed on any of the five cliff cells in any of the records' games
     (M-0006). In every game that states a unit on one of them, that unit was placed on a
     neighbouring cell, and none of the placed cells is among the cells Q-1502 lists. So the
     request's premise, that units of those games were placed on the cliff cells, does not hold, and
     question 1 is answered only in that part. What ends those games, which the request's need
     asks about, is not answered here (Open questions, *Q-1713 question 1, for the records*). Why
     each unit's placed cell differs from its stated cell is not recorded (claim 7). The finding
     is about the archived logs' placed cells only: M-0006 does not compare the 32 replays' tick-0
     cells with them.
   - Questions 2 and 3 below are about a unit that stands on such a cell. No unit of the records'
     games was placed on one, so they describe a placement that did not occur in those games and
     are not
     findings about those games' units.
   - Question 2. A hunting infantry unit on such a cell fires at an enemy on open ground only once
     that enemy is its target, and the enemy coming within its weapon's range does not by itself
     make it one. Its hunt search does not consider that enemy (S-0309 claim 7), and the
     limited-scan mark that turns a map-wide search into a range search is given only to vehicles
     (S-0309 claim 22). Retaliation has no reach-region test (S-0309 claim 18). For a unit of a
     computer-controlled house, as the records' houses were (M-0001), the original's behaviour list
     of S-0215 claim 18 does not hold back the unit (S-0215 claim 18, original only). So after the
     enemy hits it, such a hunter with no target makes that enemy its target when the *hunt*
     behaviour has the retaliate property (which a behaviour has unless the user's behaviour data
     turns it off: S-0215 Constants, absent-key default yes), and S-0309 claim 18's other
     conditions allow it; one that already has a target switches only as S-0215 claim 18's threat
     comparison allows. For contrast, in our engine a hunting unit of a player house, other than
     S-0215's test-only computer-controlled player house, retaliates only while its current
     behaviour is *guard*, *area guard* or *patrol*, unless the game-wide player return-fire
     setting is on, and then on the same conditions (S-0215 claim 18).
   - Question 3. A hunting unit on open ground rejects an enemy on such a cell in its hunt search,
     however close (S-0309 claim 7). It can still target that enemy by retaliation after a hit,
     on the same conditions as in question 2, or, for a vehicle with the limited-scan mark
     (S-0309 claim 22), by the range search that replaces its hunt search, which has no
     reach-region test (S-0309 claims 7 and 11).
   - Backing: `measured` (M-0006) for question 1's finding that no unit was placed on the cliff
     cells, and for the records' units' being in play at tick 0 in the 32 replayed games, and
     (M-0001) for their houses' being the original's computer players. How the records' units
     were placed is open (claim 7). For questions 2 and 3 the backing rests only on other specs'
     scenarios, as this spec has none of its own for them: S-0309 scenario A11, parts a and j, for
     the hunt search and for retaliation against cells of no reach region, and part c for the
     limited-scan mark; S-0215 scenario B7 for the behaviour list and the player return-fire
     setting. Retaliation from such cells is `unmeasured` here: it rests on S-0309 claim 18's
     conditions, which have no reach-region test.
   - Confidence: high for their houses' control (M-0001) and for question 1's finding, which
     M-0006 reads from every archived game's log. High for the hunt search's rejection, as S-0309
     claim 7 rates cells of no reach region, and for the retaliation behaviour list, as
     S-0215 claim 18 rates it; medium for retaliation against such cells, as S-0309 claim 18 rates
     its conditions, and for retaliation from them (unmeasured, as above); low for the switch of a
     computer-controlled hunter that already has a target, as S-0309 claim 25 part h rates the
     threat comparison that S-0215 claim 18 uses; low for the limited-scan route, as S-0309 claim 22
     rates when the mark is given.

7. **Units of the measurement records.** The units of M-0001, M-0002 and M-0003 were in play in the
   tick-0 state of the 32 replayed games of M-0006 (the replays' per-tick records hold them from
   tick 0). M-0019 reads their tick-0 state in all 1860 replays. In the all-infantry games every
   placed infantry unit stands at one of claim 2's standing positions, each where claim 2 puts it
   for its draw, and the raw values taken before tick 0 hold each infantry object's draws
   consecutively in creation order, the last giving the standing-position draw (claim 3). That the
   raw values before it are the creation draws is claim 3's inference. In the other games every
   placed infantry unit stands at a position of S-0208 table P, never the centre, the raw values
   match claim 3's draws by their count, and that each infantry unit stands at its own draw's
   position is a reading.
   Before each unit there are as many of the
   set-up's objects off the map at tick 0, created after the previous unit (spawned objects,
   M-0019's class, are not among them), as there are cells before its own in claim 8's order. None
   of them is present at the next tick, each is of that unit's type and house, and the count of
   raw values takes each as claim 8 takes a refused try, in every game. That each such object is the
   try of one
   refused cell, and which cells were tried, are read from that count, not identified by the
   recording. So the records' units of the all-infantry games were placed as claims 2 and 3 give a
   play-time placement, and the other games' positions and totals agree with it. Whether claim 5's
   entry test refused their cells, and whether claims 1 and 4 hold for them, is not observed (Open
   questions, *Q-1711 question 5*).
   - Moved units. M-0006 reads every unit's placed cell from the archived logs of all 1860 games. In
     some games some side-A units were placed on a cell next to their stated cell. No side-B unit
     was moved, and every unit was placed. M-0001's Setup says that a unit that its stated cell does
     not take stands on a free neighbouring cell, and that infantry take one cell each. The logs
     place more than one unit on some cells (M-0006), so the one-cell-each statement does not hold
     for those cells, and a "free" neighbouring cell there is one that accepts the unit, not an
     empty one. Several infantry units in one cell do not contradict claim 2, by which a placement
     into a cell that already holds infantry takes a free standing position of its row, but no
     record gives which position such a unit took (Open questions). Which nearby
     cell each moved unit was placed on is a fact of the records' Setup,
     not a rule of the game.
   - Why those stated cells were taken M-0006 does not record. For the five *rock* cells of
     Q-1502 (124,79, 124,80, 125,79, 125,80 and 125,86), a refusal by the entry test of claim 5
     would fit: an infantry type whose movement class cannot enter *rock* is refused there. For
     the other moved units, the records are silent on what their stated cells held at placement,
     so this claim does not explain their moves. The neutral bunkers stand at 126,78 and 126,87,
     next to some of those cells, and which cells they cover beyond their own is not recorded
     (M-0006).
   - Backing: `measured` (M-0006) for the units' being in play at tick 0 in the 32 replayed games,
     from the replays' records, and for the stated and placed cells of the moved units that M-0006
     lists, from the logs. `measured` (M-0019) for the standing positions, the raw values of the
     draws, the count
     of the set-up's objects off the map before each unit and their being of that unit's type and
     house. For the raw values that is their grouping per object in creation order in the
     all-infantry games, the raw values before an infantry object's R(0, 3) being its creation
     draws inferred, and only their count in the games with vehicles (claim 3). That each off-map
     object is one refused cell's try, and which cells were tried, are inferred from that count, not
     identified by the recording. `unmeasured` for why a cell refused a unit: M-0006
     does not record why units moved. PT5 part d tests claim 5's refusal rule itself.
   - Confidence: high for the units' being in play at tick 0 in the 32 replayed games and for the
     moves from the five *rock* cells that M-0006 lists, as it reads them from the logs. High for
     the standing positions at claim 2's positions in the all-infantry games, and in the other
     games for positions of S-0208 table P, never the centre, and medium there that each unit
     stands at its own draw's position (a reading). High for the count of the objects off the map
     and their being of the unit's type and house (M-0019). Medium that each is one refused cell's
     try (inferred). For the
     raw values: high for their count and grouping in the all-infantry games (each infantry
     object's draws consecutive, in creation order, the standing-position draw last) and for their
     count in every game. Medium that the raw values before an infantry object's R(0, 3) are its
     creation draws, and medium for their per-object order in the games with vehicles, as claim 3
     rates them. Low
     that the moves from the five *rock* cells are entry test refusals: that is inferred, and it
     does not explain the other moves.

8. **Our arena's set-up (design).** Our engine's arena, the tool that sets up the games of the
   measurement records M-0001 to M-0003 and M-0017 on their map, follows the rules below. They are
   rules of our own, not claims about the original. The records' placement is known by its
   observed effects at tick 0 (claim 7, M-0019): the order, the draws, the standing positions and
   the draws of refused cells below reproduce those effects. How the records counted their groups
   is not established (Open questions). The rules have no quirk switch. M-0006's and M-0019's
   committed outcomes are their acceptance check.
   - **Map objects (Q-2098, in part).** The arena loads the records' map as any game on that map
     is loaded (S-0009): every map entry that S-0009's loading creates is in play from tick 0 as
     the map gives it, the neutral house's buildings and infantry and the terrain objects
     included, except the neutral house's vehicles, which *Removal before the groups* takes off
     the map. The arena adds or changes none of these objects and removes none beyond that
     removal and what S-0009 removes. What stands on the bunker cells M-0006 lists
     (*Map content*) is whatever the map gives there, with the kind and behaviour its type's
     imported data give. The arena has no rule of its own for these objects and does not pick
     them by type. Acceptance: in the arena's tick-0 state a building of the neutral house
     covers each of those four cells, whether one building or several. What the records' games
     had there, and what a stock game on the map does with them, stay open (Open questions).
   - **Game start (Q-2336).** The arena's game is a standard skirmish game with the houses of the
     records' games: the two players' computer houses, a human player house at start 3 (M-0001's
     idle house; it holds a start position, so S-0009 claims 19 and 45 treat it as a player, not
     as an observer), then the neutral and the special house. They are created in S-0009 claim
     19's order and take S-0202 claim 14's house-creation draws, in two rounds. Our arena's setup
     supplies the game's settings (S-0412 claim 1): the bases setting on, the unit-count setting 0
     and crates off. In the original the bases setting is the base rules' bases default, copied
     when the program starts, and the unit-count and crate settings are lobby values (S-0412
     claim 1), which the records' settings file gives (M-0019, Setup: no menus). Crates off is
     M-0001's setting (Setup, *Players*), which M-0019 says the records' games use. The bases
     setting and the unit count are a reading of the records, which give neither. The records'
     houses had starting construction vehicles for the harness to remove (M-0001 and
     M-0019, Setup), and S-0009 claim 45 gives a base unit only with the bases setting on. Each
     records' house owns only its placed group (M-0001, *Players*), and its starting objects are
     vehicles of no set-up type, gone at the next tick (M-0019). So no house bought a budget unit,
     as the unit-count setting 0 gives (S-0009 claim 45's budget is then 0). The records do not
     show that their setting was 0 rather than another value whose budget bought nothing. So S-0009
     claims 45 and 46 give each of the three player houses its base unit, house by house in claim
     19's order: each with its creation draw and one creation number, put directly on its base
     cell with no draw. No house buys a budget unit (its budget is 0), and after the last house
     comes the final R(0, 65535). Every other group before the set-up is as S-0202's map-loading
     table gives it. So the base units' creation numbers come before every set-up object's.
   - **Seeds and the human player house's country (Q-2446, Q-2447).** The records name their
     games' seeds 0 to 5 (M-0001's naming, which M-0016, M-0017 and M-0019 keep). Their scenario
     streams behave as S-0202's generator seeded with 1000 + *s* for the game with seed number
     *s*: so seeded, the draws of *Game start* and S-0202's map-loading table reach M-0019's *N*₀
     exactly for each of the six seed numbers, and seeded with *s* itself they do not. So the
     arena's game for seed number *s* seeds that generator with 1000 + *s*, and the M-0019
     acceptance item below, PT6 and every other per-seed comparison with the records use that
     seed. How the records' setup turns seed number *s* into that seed is not recorded. The human
     player house takes side A's country (design). The country picks the house's base unit
     (S-0009 claim 45). The house-creation draws do not depend
     on the country (S-0202 claim 14).
     - Backing: that the records' streams start as from 1000 + *s*, `measured` (M-0019: its six
       *N*₀ values, which the acceptance item below checks). The seed rule and the country are
       design rules of our arena, `unmeasured` (a design decision; no measurement of the
       original decides it).
     - Confidence: high for the seed rule and the country as design rules (fixed by this
       claim). Medium that the records' games were seeded so: one added constant fits all six *N*₀
       values exactly, but no record states the seed.
   - **Removal before the groups (Q-2336).** After resource-glint setup and before the first
     group's first creation, the arena takes off the map every vehicle that the human player
     house does not own: the two computer houses' base units and every vehicle the map gave the
     neutral house. The removal takes no random draw, starts no animation and frees the cells the
     vehicles stood on, so the take test below does not see them. None of them exists at the next
     tick. The creation numbers they took stay taken. The human player house's base unit stays on
     its base cell for the whole game, with no order from the arena, and takes whatever draws a
     unit of its type and behaviour takes in play. No house treats the human player house as an
     enemy: no object targets or damages its objects, and the computer houses ignore it, as
     M-0001's Setup lists for the records' games (design). The neutral house's buildings,
     infantry and terrain objects stay.
   - **Order.** The arena creates and places the units one at a time: the group of the row type of
     the game's pairing first (the type that heads the pairing's row in M-0001 to M-0003, and
     in M-0017), on whichever side it plays, in increasing unit index *i* (the index of
     M-0006's stated-cell formula), then the other group in the same way. So in a game with the
     row type on side B, side B's units come first (M-0019). Each unit is created, then tested and
     placed as below, before the next unit is created. So the scenario stream takes, per unit and
     in this order, its creation draws and then its placement draw if any (claim 3), and no draw
     of another of the set-up's units falls between a unit's creation draw and its placement draw.
     The objects that a unit's creation makes with it, its initial slot objects (S-0506 claim 1)
     and any initial slave group (S-0402 claim 19), are not placed by this step: their creation
     draws come with the unit's own, after it and before its placement draw, in S-0611 claim
     14's order, for every kind of unit. Each object created, the unit and each object made with
     it, takes one creation number right after its own creation draw, in that order (S-0611
     F5), so later objects' identifiers follow from the order. Each unit is first tried on its
     stated cell.
   - **The take test.** A cell *takes* a unit when the cell is inside the map's usable area by
     S-0009 claim 4's height-aware test (as S-0009 claim 5a applies it to object placement), the
     unit's entry outcome there, asked as claim 5 asks it, is *clear*, and:
     - for a vehicle, no unit stands on the cell
     - for an infantry unit, the cell's ground record has neither its vehicle-present flag on
       nor its object-present flag on with no open gate in the cell (claim 2's first row), and at
       least one of the right, left and bottom standing positions (the positions of the centre
       rows of S-0208 table O) is free.

     A cell outside the usable area never takes a unit, so the case that claim 2 leaves open
     does not arise in the arena. A group of an aircraft-kind type is not covered here (Open
     questions). The test itself takes no random draw.
   - **Draws of a refused cell.** Each cell that refuses the unit costs the draws of one try, a
     unit of the unit's own type created for that cell, in this order and before the next cell is
     tried: the try's creation draws as in *Order* (the unit's own, then those of the objects its
     creation makes with it), then, for an infantry unit, one draw R(0, 3), taken whether or not
     claim 2's first row would refuse the cell, whose result is not used. So for an infantry unit
     of a spawner type, its slot objects' draws fall between its creation draw and that R(0, 3)
     draw, as at a placement. The unit of a refused try never enters play: at the next tick no
     object of it exists. Our engine keeps no object for it (design, a documented departure with
     no switch, listed under *Edge cases and quirks*, as S-0611 claim 11 builds none for a unit
     that is never placed), only the try's draws and its creation numbers. As S-0611 claim 11
     retains them for its never-placed units, a refused try reserves one creation number for the
     try's unit and one for each object its creation makes, each right after its own creation
     draw, in *Order*'s sequence, as if the try's objects were built. So the creation numbers,
     and with them the order of later objects' identifiers, are those of a set-up that built
     every try's objects, as the original builds the try's unit off the map (Backing).
     Whether the original's
     off-map object has any other effect in tick 0, on a house's counts for instance, is
     `unmeasured`, as there. The next cell is then tried as a new try, with its own creation draw,
     and the unit that a cell takes has the draws of claim 3 there. So a unit placed on the *p*-th
     cell of the order below (0 for its stated cell) is preceded by *p* refused tries. For a unit
     whose creation makes no other object they take 2*p* draws for infantry and *p* for a vehicle.
     A unit whose creation makes other objects adds, for each try, the creation draws of those
     objects. M-0019's count of raw values takes each off-map object of the records as such a
     try (claim 7).
   - **Moving on.** A unit that its stated cell does not take is tried on the eight neighbouring
     cells of its stated cell (*x*, *y*), in this order: (*x* − 1, *y* − 1), (*x*, *y* − 1),
     (*x* + 1, *y* − 1), (*x* − 1, *y*), (*x* + 1, *y*), (*x* − 1, *y* + 1), (*x*, *y* + 1),
     (*x* + 1, *y* + 1). It stands on the first that takes it. If none takes it, the set-up stops
     and reports an error.
     - First come, first served: no cell is kept for a later unit. A later unit whose stated cell
       an earlier unit took is tested there in turn, so an infantry unit can still be taken by a
       cell that holds infantry, and a vehicle is moved on.
   - **Placement on the cell that takes it.** The unit is placed there, on the ground of that
     cell, which is inside the usable area, by a play-time placement as claims 1 to 4 give it:
     the vehicle facings of claim 1, the standing position of claim 2 with its one draw R(0, 3)
     for an infantry unit, the draws of claim 3 in their order, and the fall-back of claim 4.
     Since the take test asks claim 2's first-row refusals, a free standing position and claim
     5's entry outcome, claim 2 then finds a position and claim 5 accepts. The group's mission is
     then queued as a placing step's behaviour (claim 4).
   - **Acceptance (M-0006, *Units off their stated cells*).** Every unit placed, no side-B unit
     moved, every move to a neighbouring cell, and the moves of the cliff-cell table. In the
     order above, wherever a group states a unit on one of the cliff cells of Q-1502, that unit
     lands on the cell the table names if the test refuses the cliff cells and the cell 126,79,
     and takes the cell the table names when its turn comes. Infantry sharing a cell fits
     M-0006's placed cells that hold more than one unit.
   - **Acceptance (M-0019).** With the records' seed numbers 0 to 5, each game seeded as the
     bullet on seeds above gives, the first object's creation draw (the first unit's, or its
     first try's when the first unit has refused tries) takes the
     scenario stream's raw value at place *N*₀ of M-0019 (S-0202's stream sequence, x₀
     at place 0). That is, the stream has taken 706, 709, 711, 718, 706 and 701 raw values since
     the seeding, in every pairing. These count raw values, not draws: a ranged draw before the
     set-up may take more than one (S-0202 claim 4), so the number of draws before the first
     creation draw is not given. From there each draw takes the raw values S-0202 claim 4 gives it
     (one each for a creation draw and a draw R(0, 3), claim 3), so every unit's draws and every
     infantry unit's standing position at tick 0 follow from the raw values in order (each unit's
     draws as claim 3 gives them, each refused try's draws as above, and claim 2's position).
     M-0019 measures this per object in the records' all-infantry games, its model counting any
     refused try as above. In the games
     with vehicles it measures the count of raw values only, and the per-object order there rests
     on claim 3's medium order. Draws taken
     before the set-up are those of *Game start* above and of S-0202's map-loading table (its
     claims 10 and 14), which this claim does not restate. M-0019 counts their raw values only.
     For each of the six seeds, the draws those rules give for the records' map and settings,
     taken in that order, take exactly M-0019's *N*₀ raw values: both house-creation rounds, the
     map objects' creation draws, the three base units' creation draws and the final
     R(0, 65535), the end-of-loading tags' draws and the resource-glint choices. That this is how
     the records' raw values divide among the groups is a reading of the records' draws, not an
     observation (medium).
   - **Group counts (Q-2099, in part).** A group's count is its budget divided by its type's
     cost, rounded to the nearest integer, then held to the records' minimum and infantry cap
     (M-0001 Setup). An exact half goes to the even one of the two counts (Formulas). The rule is
     the same for infantry and vehicle groups.
   - Backing for *Game start* and *Removal before the groups*: `measurable`. PT6 tests them in
     our engine. Crates off in the records' games rests on M-0001's Setup, which states it
     (high). The bases setting on and the unit-count setting 0 in the records' games are a reading
     of M-0001's and M-0019's Setup and of M-0019's starting objects, not a recorded setting
     (medium for the bases setting, low for the unit count's value). PT8 runs with a test copy of
     the base rules that only lowers the minimum unit count to 0 (S-0412 K3), the bases default
     as installed, as in the records' games, and the unit-count setting 0 in its settings file,
     and fails if the tick-0 state, the house rounds or the starting-unit step differ from these
     rules and S-0202 claim 14 (PT8, *Falsified if*), not by a comparison with the records,
     which hold no group checkpoints. PT8 tests, in the original under the records' harness, the
     tick-0 state (which vehicles are off the map at tick 0 and gone at the next
     tick, and the human player house's base unit on its base
     cell all game) and, group by group at each group's own
     checkpoint, the draws the house rounds and the base units take. Medium for both rules, as
     the original's tick-0 state is not yet in a record. M-0019 measures only the total, *N*₀,
     at the first creation: that the two rules' draws are part of it, and how many of its raw
     values each group takes, is the reading above (medium, `measurable` through PT8).
   - Backing: `measurable`. Scenario PT6, a run of our engine's arena on the records' map, tests
     the order, with each unit's creation and placement draws interleaved, the take test, the
     neighbour order, first come first served and the placement draws, and checks the acceptance
     items against M-0006's (`measured`) outcomes for its groups. Scenario PT7, a run of our
     engine's arena with test data, reaches the exact half, the minimum, the cap, the error stop
     and raised cells at the usable area's top and bottom edges, where the height-aware test and
     the plain test differ, which the records' groups need not reach. The acceptance items for
     the arena's other groups are `unmeasured`. `measured` (M-0019) for the records' side of the
     order in the all-infantry games, per object (claim 3). In the games with vehicles the order is
     a reading, `unmeasured`: M-0019 fits no per-object places there and does not attribute raw
     values to vehicles. Also
     `measured` (M-0019): the totals of raw values that the refused cells' tries and the placed
     units give, the
     M-0019 acceptance item,
     and a count: in all 1860 replays, the set-up's objects left off the map before each unit
     (spawned objects, M-0019's class, are not among them) equal the place
     of its cell in the neighbour order, for every set-up entry, and the counts of raw values and
     the standing positions fit claims 2 and 3 with these draws (S-0202 claim 4 for their raw
     values). The count fits
     the neighbour order, but it
     does not show which cells were tried, so the neighbour order itself is `unmeasured` in the
     records. Each off-map object is of the unit's type and house (M-0019); that each is one
     refused cell's try is inferred from the count (medium for the records).
     Whether the records' harness refused the same cells by the same test, and its rounding, are
     `unmeasured` (Open questions). The draws of the objects made with a refused unit's creation
     (a spawner's slot objects, a slave group), and their place before an infantry unit's R(0, 3)
     draw, are `unmeasured`, design: no committed record gives a finding on them
     (Open questions). The order of the objects' identifiers is `measured` (M-0019 reads it in
     all its games as the order of the numbers the recording gives the objects, its creation
     order) and is the order of objects above. In the games with vehicles that measures the
     objects' order, not the draws' places in the stream, which stay a reading there.
     The creation numbers' values, and that a refused try reserves them as if its objects were
     built, are `unmeasured`, design: M-0019 uses the identifiers only for their order and for
     which objects exist at the next tick, not for their values.
   - Confidence: high for the order (each unit created, tested and placed before the next is
     created, the pairing's row type's group first), the neighbour order, the draws of refused
     cells (for a unit whose creation makes no other object, two for an infantry unit and one
     for a vehicle) and the count rule with its exact-half tie, as design rules of our
     engine. Medium, design, for keeping no object for a refused try, its other effects in the
     original `unmeasured`. High for the identifiers' order (M-0019). Medium, design
     (`unmeasured`), for the creation numbers' values, a refused try's reserved as if its objects
     were built. Medium for the
     draws of the objects made with a try's
     creation,
     and for their falling before an infantry unit's R(0, 3) draw (design, `unmeasured`). M-0019
     measures in the records the order per object in the all-infantry games (high there), and the
     order in the games with vehicles is medium, a reading (claim 3), since M-0019 fits no
     per-object places there. M-0019 measures the totals of raw values in all games together, which
     include the refused cells' tries, and a
     count of off-map objects per unit that fits the neighbour order without showing which cells
     were tried. Medium that each such object is one refused cell's try of the unit's type, in
     the records (inferred). For the records' per-object grouping of raw values: high in the
     all-infantry games, the raw values before an infantry object's R(0, 3) being its creation
     draws medium, and medium in the games with vehicles, where only the count is compared
     (claim 3).
     High for the values of the M-0019 acceptance item, counts of raw values that M-0019
     measures. Whether our arena
     reaches them rests on our engine's game-start draws (S-0202), at their levels. Medium for
     *Game start* and *Removal before the groups*, as design rules that copy the records' game
     start: the house list, the base units and the vehicles taken off the map rest on a reading
     of the records' draws and tick-0 state that no record yet states (PT8). Of the settings,
     high for crates off (M-0001's Setup), medium for the bases setting on and low for the
     records' unit-count value being 0 (readings, above). Medium, design, for
     the human player house's objects being neither targeted nor damaged, copied from M-0001's
     Setup. Medium for the
     division of *N*₀ among the groups before the set-up (a reading). The draws
     that this order interleaves are at claim 3's levels. Medium for the take test,
     first come first served and the error stop, since they act on the take test's result on a
     given cell, which rests on claim 5's entry outcome and on claim 2's first-row
     refusals and free standing positions (medium, with the object-present refusal `unmeasured`
     there), and, for the usable-area part, on S-0009 claim 4's height-aware test as S-0009 claim
     5a applies it to object placement (medium there). For the placement on the cell that takes
     the unit, and for its finding a position and being accepted there, the levels of claims 1,
     2, 3 and 5 that it invokes. Low for claim 4's fall-back for a unit that is ready to change,
     as claim 4 rates it. For the acceptance items: medium, for the groups PT6 sets up, that
     every unit is placed, that no side-B unit is moved, that every move is to a neighbouring
     cell and that the arena's tick-0 state passes the bunker-cell check, since these rest on the
     map's content, on S-0009's map loading and on M-0006's cells. For the arena's other groups
     these items are `unmeasured` (medium): they follow from the same rules and the same map, but
     no run sets those groups up. Low that the order gives the moves of the cliff-cell table,
     since it rests on the refusals of claim 7 (low) and on the neutral building's covering
     126,79, which is not established (Open questions). Low for the records' harness's own rules.

## Formulas

> Body facing = turret facing = 256 × *v* facing units, *v* the integer facing value 0 … 255.
> Exact integer arithmetic.

> Standing-position row index *k* = R(0, 3), one ranged draw on the scenario random stream
> (S-0202 claim 4's range rule). Empty usable cell: *k* = 0 or 1 → right, *k* = 2 → left,
> *k* = 3 → bottom (S-0208 table O).

> Visceroid placement draw = R(0, 29) (S-0202 claim 10).

> Arena group count before the records' minimum and cap (claim 8, design), for budget *B* and
> cost *c*, both positive integers: *q* = ⌊*B* / *c*⌋, *r* = *B* − *q* × *c*. Count = *q* when
> 2*r* < *c*, *q* + 1 when 2*r* > *c*, and the even one of *q* and *q* + 1 when 2*r* = *c*.
> Exact integer arithmetic.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Standing-position draw range | 0 … 3 | ranged-draw result (integer) | S-0208 claim 3's centre row; claim 2 |
| Facing value scale | 256 | facing units per facing value | Claim 1 |
| Visceroid placement draw range | 0 … 29 | ranged-draw result (integer) | Claim 3 (S-0202 claim 10) |

## State and ordering

| State | Event | Result |
|---|---|---|
| unit created, not placed | play-time placement, infantry, usable cell | draw, standing position or failed placement as claim 2 says |
| unit created, not placed | play-time placement, visceroid-flagged vehicle, entry outcome *clear* | one draw R(0, 29) (claim 3); body and turret at 256 × *v* (claim 1) |
| unit created, not placed | successful play-time placement, other vehicle | no draw; body and turret at 256 × *v* (claim 1) |
| unit created, not placed | play-time placement whose entry outcome is not *clear* | not placed; an infantry draw already taken is kept; no visceroid draw (claim 5) |
| placed, resting behaviour queued by the fall-back | placement of a unit ready to change | resting behaviour current, nothing queued (claim 4) |
| placed | placing step queues a behaviour | queued by S-0215 claim 7; not started (claim 4) |
| placed, behaviour queued | first update at which it is ready and no S-0215 claim 2 exclusion holds, at the first per-tick start | queued behaviour current and processed in that update (claim 4) |
| placed, behaviour queued | first update at which it is ready and no S-0215 claim 2 exclusion holds, only at the late start | queued behaviour current; first processed at the next update (claim 4) |

Ordering: for units created and placed one at a time, each unit's creation draws, then its
placement draw if any, in the order the placing step places them (claim 3).

## Edge cases and quirks

No quirk switch. A placed infantry unit not standing at the centre of its cell follows the same
standing-position rule a walker uses, and is not a fault.

**Departure from the original (no switch).** In our arena's set-up (claim 8) a refused try keeps
no object of the unit, only the try's draws and creation numbers, where the original leaves an
object of it off the map. This is a documented departure with no switch, as S-0611 claim 11's
for a unit that is never placed. Medium, design; the original object's other effects in tick 0
are `unmeasured`.

## Import mapping

| Key | Feeds | Parse form | Absent-key default |
|---|---|---|---|
| vehicle type `SmallVisceroid`, `LargeVisceroid` | either one on: the placement draw R(0, 29) of claim 3 (as in S-0202) | boolean each | no |

## Measurement scenarios

PT1 to PT5 use stock data with no modifications except the test type data named in PT3 and
PT5, a small
purpose-made map, flat (except PT5 part d's cliff cell) and with all its cells usable unless a
part says otherwise, and a measurement
harness that places units after the game has started, before the first tick at which they are
updated, with stated cells and facing values. Every type that PT1 to PT5 place makes no other
object at its creation: its data name no spawned type with a spawn count of at least 1 (S-0506
claim 1) and no slave type with a slave count of at least 1 (S-0402 claim 19). The only
exceptions are the test copies that PT3's
third and fourth runs name for one slot each. So each placed unit takes one creation draw only,
and the exact sequences below hold. That harness is an exception to the glossary's
measurement setup: it puts units into a running game, which the setup otherwise excludes. Game
speed does not matter; record by tick. The
random-state recorder (glossary) logs the scenario stream's state with the tick number, so that
the draws at each placement can be counted and their order and values checked.

**PT1: vehicle facings (claim 1).** Place, at play time, one turreted and one non-turreted vehicle
type at facing values 0, 32, 64, 96, 128, 160, 192 and 224, and also 37, 101 and 201, which are
not multiples of 32, one vehicle per value and type, far
apart, with no enemy on the map and no behaviour given. Run 3 seeds. Record the body and turret
facing of each vehicle right after its placement, before its first update (an idle action at
that update may start a turn).
- *Expected:* body and turret both exactly 256 × *v* for every vehicle, the same in every seed.
- *Falsified if:* any facing differs from 256 × *v*, or differs between seeds.

**PT2: infantry standing positions (claim 2).**
- *a, empty cells:* place one infantry unit (a type that does not move on its own while idle, no
  behaviour given, no enemy on the map) on each of 16 empty usable cells, 4 cells apart. Run 8
  seeds (128 placements). Record each unit's standing position right after its placement and,
  with the recorder, the draw taken at its placement.
  - *Expected:* each position is the one S-0208 table O gives for the recorded draw: right for 0
    and 1, left for 2, bottom for 3. Over the 128 placements, right between 46 and 82 times, left
    and bottom each between 17 and 49 times. No unit at the centre or top position.
  - *Falsified if:* a position disagrees with its recorded draw, any unit stands at the centre or
    top, a placement takes no draw, or a count falls outside its bounds.
- *b, occupied cell:* place a first infantry unit on a cell, then a second on the same cell, and
  record both positions and both draws. Run 8 seeds.
  - *Expected:* the second unit stands at the first free position of row R*k* for its own draw
    *k*, never at the first unit's position.
  - *Falsified if:* the second unit shares the first's position or ignores the row order.
- *c, outside the usable area:* place an infantry unit on a legal cell outside the usable area,
  if the harness can place there. Record whether the unit is on the map after its placement, its
  position, and the draws at its placement. This part records the outcome for claim 2's
  `unmeasured` case and is not a test of it: claim 2 predicts neither the position nor whether a
  draw is taken there. If the harness cannot place there, report part c unreached.
- *d, cell with a vehicle:* place a vehicle of a type without the visceroid flags on
  a usable cell, then an infantry unit on the same cell. Run 4 seeds.
  - *Expected:* no draw at the infantry unit's placement, and the unit not on the map in that
    cell; the harness reports what it does with the unit.
  - *Falsified if:* the infantry unit stands in that cell, or a draw is recorded at its
    placement.
- *e, full row:* place three infantry units on one cell, then a fourth on the same cell. Each
  row of S-0208 table O orders all three positions, so the three units take the right, left and
  bottom positions whatever their draws. Run 4 seeds. A stationary friendly unit in the cell
  would also refuse the fourth unit by S-0209 row WR4, so this part checks the draw and the
  failure, not which of the two refusals acts.
  - *Expected:* one draw R(0, 3) at the fourth placement, and the fourth unit not on the map in
    that cell.
  - *Falsified if:* the fourth unit stands in that cell, or its placement takes no draw or more
    than one.
  - In parts d and e, only the draws of the attempt at the stated cell are counted. If the
    harness then tries the unit in another cell, that retry and its draws are reported
    separately and are not a falsifier.

**PT3: draw count and order (claim 3).** Place, at play time, with a placing step that creates
and places each unit before it creates the next, with types that make no other object at
their creation (the condition above), in this order: a vehicle of a type
with neither visceroid flag, an infantry unit, a second vehicle without the flags, a second
infantry unit, all on empty usable cells. Repeat with a vehicle type that has a visceroid flag
placed in the third slot instead (test type data). In a third run, the second slot's infantry
type is a test copy whose data name a spawned type and a spawn count of at least 1, so that its
creation makes slot objects (S-0506 claim 1). The spawned type's own data name no spawned type
and no slave type (a test copy where needed), so its slot objects make no objects at their
creation and the window holds their creation draws only. In a fourth run, the first slot's
vehicle type is a test copy without the visceroid flags whose data name a slave infantry type
and a slave count of
at least 1, so that its creation makes an initial slave group (S-0402 claim 19). The slave type's
own data name no spawned type and no slave type (a test copy where needed), so the creation set
of S-0611 claim 14 holds the slaves and nothing made by them. Every other slot of the four
runs, the second run's visceroid vehicle included, keeps the condition above. Run 3 seeds each.
Record with the recorder
every scenario-stream draw from just before the first creation to just before the first update
of any placed unit. Draws taken by that update (such as an idle action's) are outside the window.
The cells used hold no tile that the land recompute of a unit entering its cell could start an
animation on (S-0010 claim 19), so no tile animation's creation draws fall in the window; the
result is a property of such a map.
- *Expected:* for the first run, in order: creation draw, creation draw, R(0, 3), creation draw,
  creation draw, R(0, 3). For the second run, R(0, 29) right after the visceroid vehicle's
  creation draw. For the third run, the second slot's creation draw, then its slot objects'
  creation draws in S-0506 claim 1's order, then its R(0, 3). For the fourth run, the first slot's
  creation draw, then one creation draw per slave in S-0611 claim 14's order, then the second
  slot's creation draw: the slaves stay inside and take no R(0, 3) (S-0402 claim 19). No other
  draw in the window. A
  draw there that another system of S-0202 claim 10 accounts for, unrelated to the placed units,
  is reported and not counted as a placement draw.
- *Falsified if:* a draw is missing, an extra draw appears at a placement, or the order differs.

**PT4: behaviour before the first update (claim 4).** Place, at play time, a vehicle and an
infantry unit, each then given *hunt* as a queued behaviour, with an enemy unit far away on the
map. Record each unit's current and queued behaviour right after its placement and before the
*hunt* order, again after that order and before its first update, and at each of its first three
updates, the tick of its first *hunt* processing, and whether
the unit was ready to change (S-0215 claim 11) at its placement and at its first update's first
per-tick start.
- *Expected:* for a unit ready to change at placement, its resting behaviour current and nothing
  queued before the *hunt* order, and *hunt* queued after it. For a unit ready at its first
  update's first per-tick start, *hunt* current and processed in that same update.
- *Falsified if:* *hunt* is current before the first update, is first processed at a later
  update although the unit was ready at that update's first per-tick start, or the resting behaviour
  is still queued in the record taken before the *hunt* order although the unit was ready to
  change (S-0215 claim 11) when it was placed.
- *Void for the resting behaviour if:* the harness cannot record between placement and the *hunt*
  order; the other records still count.

**PT5: the entry test (claim 5).** On that map, give some usable cells water and one cell an
intact wall. Place, at play time: (a) a ground vehicle of a type without the visceroid flags that
cannot cross water, on a water cell, and the same type on the wall cell, the type being one that
does not crush the wall; (b) a test vehicle type with a visceroid flag that cannot cross water,
on a water cell; (c) an infantry unit of a type that cannot cross water, on an empty water
cell; and, as controls, each of these types on an empty clear cell; (d) an infantry unit of an
ordinary walking type, with no tether link, on a flat cell (no slope) whose land type is *rock*,
with the same type on an empty flat clear cell as its control. If the map has no flat *rock*
cell, a cliff cell between two height levels (as in Q-1502) is used instead, and a refusal there
is reported as not telling *rock* from the cell's height step. The type must be one whose
movement class cannot enter *rock* (S-0211 claim 10 and its table: its class's multiplier for
*rock* is 0); that is the property that refuses it. A type whose class can enter *rock* does not
test claim 5 here. Run 3 seeds. Record with the recorder every scenario-stream draw at each
placement, and whether each unit is on the map in its cell right after its placement.
- *Expected:* the controls are placed, the visceroid control with its draw R(0, 29) and the
  infantry control with its draw R(0, 3). In (a) and (b) the vehicle is not on the map in that
  cell and no placement draw is recorded. In (c) the infantry unit is not on the map in that cell
  and one draw R(0, 3) is recorded at its placement. (d) is as (c), on the rock cell.
- *Falsified if:* a unit of (a) to (d) stands in its refused cell, a placement draw is recorded in
  (a) or (b), or (c) or (d) records no draw or more than one.
- Only the draws of the attempt at the stated cell are counted. If the harness then tries a
  refused unit in another cell, that retry and its draws are reported separately and are not a
  falsifier.
- *Void if:* the harness places without the entry test, as its operator states its placement
  method. Units of (a) to (d) standing on their refused
  cells then say nothing about claim 5.

**PT6: our arena's set-up (claim 8, our engine).** This part runs our engine's arena, not the
original. Set up, on the records' map, every side-A group of M-0001 and of M-0002 against
its side-B group, each pairing as its own run at seed number 0 (claim 8). Record at tick 0
every unit's stated
and placed cell, every scenario-stream draw from the first creation to the first update, and every
building of the neutral house with the cells it covers. Record also the stream's count of raw
values since the seeding at the end of each group of S-0202's map-loading table and of each
house-creation round, every object of every house at tick 0 with whether it is on the map, and
every object at tick 1.
- *Expected (rules):* units placed in the order of claim 8; every moved unit on the first cell
  of claim 8's neighbour order that takes it; the draws in claim 8's interleaved order, each
  unit's creation draws (its own, then those of the objects its creation makes, claim 8's
  *Order*) followed by its own placement draw if any before the next unit's creation
  draw; exactly one draw R(0, 3) after the creation draws of each infantry unit placed; for each
  cell the take test refuses, the try's creation draws and, for infantry, one draw R(0, 3),
  before the next try; the first object's creation draw (the first try's when the first unit
  has refused tries) at claim 8's M-0019 raw-value place for the seed, and every
  infantry unit's standing position the one claim 2 gives for its draw, as in M-0019; and group
  counts as claim 8's rounding gives them. Five houses created in each of the two rounds,
  three base units created, the final R(0, 65535) taken; at tick 0 the two computer houses'
  base units and every vehicle of the neutral house off the map, none of them at tick 1, and the
  human player house's base unit on its base cell (claim 8's *Game start* and *Removal before
  the groups*).
- *Expected (acceptance):* every unit placed; no side-B unit moved; every move to a neighbouring
  cell; a neutral building covering each of M-0006's four bunker cells (each medium); and,
  wherever a run's group states a unit on a cell of M-0006's cliff-cell table, that unit on the
  cell the table names (low).
- *Falsified if:* a unit stands off the cell that claim 8's take test and neighbour order give it on
  the map as recorded; the draws come in another order, for example all creation draws first and
  then the placement draws; an infantry unit's placement has other than exactly one draw R(0, 3)
  after its creation draws; a refused cell takes other than claim 8's draws for it; the first
  creation draw takes another raw value, or a standing position differs from claim 2's for its
  draw, while the draws before the set-up match our engine's game start (S-0202); or a group's count
  differs from claim 8's rounding. A unit left unplaced (the error stop), a side-B unit moved, a
  move beyond a neighbouring cell, a cliff-cell unit on another cell than M-0006's table gives, or a
  bunker cell with no neutral building fails the acceptance check and is reported against that
  item's level in claim 8 (medium, or low for the cliff-cell table), not against claim 8's rules.

**PT7: our arena's edge rules (claim 8, our engine).** This part runs our engine's arena with
test data of our own, not stock data and not the records' groups. Parts a to c use a small flat
test map with every cell usable. Part d uses a small test map with flat raised cells (no slopes)
at its usable area's edges. Set up these cases, each at seed 0:
- *a, exact halves:* one infantry group and one vehicle group whose test budget is exactly an odd
  multiple of half its test type's cost, so that *B* / *c* = *n* + 1/2 with *n* even, and one
  each with *n* odd, *n* and *n* + 1 both at least the records' minimum and, for infantry, at
  most the infantry cap.
- *b, minimum and cap:* an infantry group whose rounded count is below the records' minimum, and
  one whose rounded count is above the infantry cap.
- *c, error stop:* a unit whose stated cell and all eight neighbouring cells are refused by the
  take test (for an infantry unit, each of the nine cells made to hold a vehicle; for a vehicle,
  each made to hold a unit).
- *d, raised edge cells:* one vehicle stated on a flat cell of height level 1 in the topmost
  half-row of the usable area by the plain test, which S-0009 claim 4's height-aware test puts
  outside it, and one vehicle stated on a flat cell of height level 1 in the half-row just below
  the area's bottom edge, outside it by the plain test and inside it by the height-aware test. All
  other take-test conditions hold on both cells and on their neighbours, which are flat at height
  level 0.

Record each group's count, for part c whether set-up completes, and for part d each vehicle's
stated and placed cell.
- *Expected:* in part a, the count is *n* when *n* is even and *n* + 1 when *n* is odd, for both
  kinds; in part b, the minimum and the cap; in part c, set-up stops and reports an error, with
  no unit of that case on the map; in part d, the top-edge vehicle on the first cell of claim 8's
  neighbour order that the height-aware test puts inside the area, and the bottom-edge vehicle on
  its stated cell.
- *Falsified if:* any count in parts a and b differs from these; in part c the unit is placed
  on any cell or set-up completes; or in part d the top-edge vehicle stands on its stated cell or
  the bottom-edge vehicle is moved (the plain test's result).

**PT8: the records' game start (claim 8, Q-2336).** This part runs the original, stock data
(the test rules copy below excepted), under the records' harness, whose settings file gives
the skirmish setup (M-0019, Setup: no menus), with M-0019's settings, on the records' map, one
all-infantry pairing of M-0019 at its six seeds. Every variant places a loose test copy of the
base rules before the program starts, as S-0412 scenarios K1 run 2 and K3 place one. The copy
is the installed base rules with the minimum unit count set to 0, since the slider's lower end
for the unit-count setting is read from the base rules only (S-0412 K3). The settings file then
sets the unit-count setting 0, as S-0412 K1 part b does, the value claim 8's *Game start* reads
from the records. The bases setting has no lobby control and no settings-file route: it is the
base rules' bases default, copied when the program starts (S-0412 claim 1), which the copy
leaves as installed, as in the records' games. Variant a is as above. In variant b the copy
also sets the bases default off, everything else as in variant a. Variant c adds
to the settings file one computer player at an unused start, with no group, so the game has
three computer players (the two recorded and the added one) and the human player: four player
houses and, with the neutral and special houses, six houses in each house-creation round. A
recorder keeps every house's objects at every tick of the first ten, and a read-only random-state
recorder logs every scenario-stream request, its bounds and its result, and the count of raw values
since the seeding after it, from the seeding to the first set-up
object's creation draw.
Record: at ticks 0 and 1 every object's house, kind, type and whether it is on the map; the human
player house's base unit's cell and health at every tick; and, at each group's own checkpoint (the
first and last request of the group, as S-0202's map-loading table orders the groups), the
requests and raw values of each house-creation round and of the starting-unit step, and the count
at the first set-up object's creation draw (*N*₀, read as M-0019 reads it).
- *Expected:* in variant a, at tick 0 the two computer houses' base units and every vehicle of the
  neutral house off the map, none of them at tick 1; the neutral house's buildings and infantry on
  the map at both ticks; the human player house's base unit on its base cell with its health
  unchanged at every tick. Each house-creation round is five house-creation draws (S-0202 claim
  14), one per house in S-0009 claim 19's order (the human player house, the two computer houses,
  the neutral house, the special house), each R(450, 1800) and taking the raw values that
  S-0202 claim 4's ranged-draw rule gives for that range from the stream's state at its place.
  The starting-unit step is three creation draws,
  one per base unit in that order, then the final R(0, 65535). *N*₀ is as M-0019 gives it. In
  variant b, the house rounds as in variant a, and the starting-unit step only the final
  R(0, 65535). In variant c, each round six draws, the added computer house's after the second
  recorded computer house's and before the neutral house's, and four creation draws in the
  starting-unit step. As a whole-load check, *N*₀ in variant b is M-0019's *N*₀ less the three
  base units' creation draws, with every later ranged draw retaken from the stream so shifted
  by S-0202 claim 4 (a draw over a range that is not a power of two can then take another number
  of raw values). In variant c it is M-0019's *N*₀ plus the raw values of the added computer
  house's two house-creation draws and its base unit's creation draw, with every draw after the
  first added one retaken in the same way.
- *Falsified if:* a neutral-house vehicle is on the map at tick 0; a removed vehicle exists at
  tick 1; the human player house's base unit moves or loses health; a house-creation round has
  another number of draws, draws with other bounds or another house order, or lies elsewhere in
  the stream than S-0202 claim 14 places it; or the starting-unit step takes another number of
  creation draws than there are player houses with a base unit. A difference in *N*₀ alone, with
  every group of these as expected, falsifies none of these rules: it lies in another group, which
  its own checkpoints locate.

## Changes for other specs

- S-0206 claim 5 and Terms: the conversion of a facing value *v* to 256 × *v* facing units, given
  there for a pre-placed vehicle, applies to a play-time placement as well (claim 1 here); its
  Terms can point here for that case.
- S-0215 (claims 4, 11, 14 and 15, Terms *placement fall-back*): a unit that is ready to change
  when it is placed has the fall-back's resting behaviour current, with nothing queued, before its
  first update (claim 4 here, rated low until PT4 runs). Claims 14 and 15 would then be qualified
  for placement: there the resting choice is made current at once for a ready unit, not only
  queued. S-0215's placement text covers only the not-ready cases and can point here for the
  ready one.

## Open questions

- **Q-2446 question 1.** No committed record gives the records' human player house's country,
  nor how their setup turns seed number *s* into the game seed (claim 8, *Seeds and the human
  player house's country*). A Setup line in the records would settle both.
- **Q-1711 question 1.** Whether the moments at which placed hunting units start to move, find a
  target and fire vary from seed to seed, and by how much. Claims 3 and 4 say what placement
  contributes: no placement draw for vehicles of types without the visceroid flags, one draw
  R(0, 29) for a vehicle of a type with either visceroid flag (claim 3), and one draw
  per infantry unit placed on the ground of a usable cell, except in a cell whose ground record has
  its vehicle-present flag on, or its object-present flag on with no open gate (claim 2). Later
  draws are those of the behaviours themselves (S-0215). The draws of tile animations that a
  placement may start are a separate open item (*Tile animations started by the placement*,
  below). An oracle run is needed for the rest (backlog).
- **Q-1711 question 4.** Whether the units of a group take the same paths and move off in the
  same order in every game. Needs the same oracle run.
- **Q-1711 question 5.** Whether the measurement records' units were put on the map with claim
  5's entry test, and with claims 1 and 4's facings and fall-back. M-0019 shows that their
  standing positions in the all-infantry games are claim 2's and that the raw values before tick 0
  fit claim 3's draws (claim 7). The rest needs a run that
  observes the placement itself (backlog). Our arena's own placement is claim 8's (design).
- **The records' harness (Q-2096, Q-2098, Q-2099).** Whether the records' harness refused cells
  by claim 8's take test, its rounding, and whether it removed or changed any map entry beyond
  the vehicles that claim 8's *Removal before the groups* takes off the map (the neutral house's
  other map objects, the bunkers' "in play from tick 0" among its listed changes), are not
  established (low). Which vehicles were off the map at tick 0 is PT8's to record. Its count
  of off-map objects per unit (equal to the
  place of the unit's cell in claim 8's neighbour order) and the totals of raw values of placed
  units and those objects are measured (M-0019, claim 8), and so is its order in the all-infantry
  games. In the games with vehicles its order is a reading, `unmeasured` (claim 8). That each
  object is one refused cell's try is inferred (medium), and which cells it tried is not
  observed. Claim 8's rules are
  our own, and a difference shows as a failed acceptance check.
- **Our arena's count before the set-up (Q-2336).** Whether our engine's groups before the
  set-up take, for the records' map and settings, the same raw values as the records' groups,
  group by group, is open. A first creation at another
  place than M-0019's *N*₀, with *Game start* and the removal in place, is a difference in one
  of S-0202's map-loading groups.
- **Q-2277 question 1, several infantry units on one cell.** Which position an infantry unit
  placed on a cell already holding infantry took in the records, where any was, is not
  established: M-0019 gives no finding of its own for that case. Claim 2 gives our arena's rule
  (the first free position of the unit's own draw's row), medium until scenario PT2 part b runs.
- **Q-2277 question 3.** Whether moving the side-A units of the M-0002 pairing that Q-2277 names
  between the centre and another standing position of their cells changes which side wins in a
  stock game is not established. It needs an oracle run in which only the positions differ
  between two arms (units put at named positions by map entries, which take no draw: claim 2).
- **Objects created along with a refused unit.** Whether the records' harness, when it refuses a
  try of a unit whose creation makes other objects (a spawner of any kind, S-0506 claim 1, or a
  type with a slave group, S-0402 claim 19), also makes them, with their creation draws, is not
  observed: no committed record gives a finding on it. Claim 8 gives our arena's try that full
  creation, as for any created unit, with an infantry unit's R(0, 3) draw after those draws
  (medium that the records' harness did the same).
- **Aircraft-kind groups (Q-2096 question 1, in part).** How our arena tests and places a group of
  an aircraft-kind type, if a record has one, is not given here (low): claims 1 to 5 cover
  vehicles and infantry only.
- **Q-2099 question 1.** What the records' harness gave for a group whose budget over its type's
  cost is exactly a half is not established (low). Claim 8's tie rule is our arena's own.
- **Q-2096 question 5.** Which units stood off their stated cells in the games of M-0001 to
  M-0003. M-0006 answers it in part: no unit unplaced, no side-B unit moved, every move to a
  neighbouring cell, and the moves from the cliff cells of Q-1502 that its cliff-cell table
  lists (claim 7). Which other units moved, by type and side, is not established here (low):
  M-0006 keeps those counts local.
- **Q-2098 questions 1 to 3.** What kind of object each of the records' four bunkers is (a tank
  bunker of S-0606, a garrisonable building of S-0508, or another kind), whether the stock map
  gives them or the records' harness added or changed them, what a stock game on the map would
  do with them, and whether the neutral house's other map objects were in play in the records'
  games, are not established (low). M-0006 gives one cell of each and that they were in play
  from tick 0. Which cells a bunker covers beyond its listed one, 126,79 among them, is not
  established either (low).
- **Q-1713 question 1, for the records.** The request asks how the infantry units on the cliff
  cells were destroyed in the stock games of M-0001 and M-0002, or how they destroyed the enemy,
  because those games end and ours do not. No unit of those games was placed on the cliff cells
  (claim 6): each unit stated on one was placed on a neighbouring cell. So what ends those games,
  and
  what the units on the neighbouring cells did, is open. Claims 1 to 6 do not decide it, and
  whether those neighbouring cells belong to a reach region is not established here. It needs an
  oracle run on the records' stock replays (backlog).
- **A failed placement.** What a placing step does with a unit whose placement fails (claims 2
  and 5): whether it tries another cell, keeps the unit off the map or removes it. This
  belongs to the step, not to the placement rule; PT2 parts d and e and PT5 record what the
  measurement harness does. In the records' set-up, before each unit, as many of the set-up's
  objects stood off the map at tick 0 as there are cells before its own in claim 8's order, none
  present at the next tick (M-0019, claim 7). That each was a refused cell's try, and the unit
  stood on a later cell, is the inferred reading of that count (medium).
- **Tile animations started by the placement.** Where the creation draws of a tile animation that
  the land recompute of a placement starts (S-0211 claim 5, S-0010 claim 19) fall relative to the
  placement draws of claim 3. PT3's map has no cell that could start one.
- **Infantry facing.** Which facing an infantry unit placed at play time starts with (claim 1
  covers vehicles only).
- **Above the ground.** Which standing position an infantry unit gets when it is placed at play
  time above the ground of its cell, for example on a bridge deck (claim 2).
