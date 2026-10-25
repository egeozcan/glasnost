---
id: S-0216
title: Units placed after the game has started
topic: simulation
revision: 20
sources-class: [decomp]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-25
measurements: [M-0001, M-0002]
requests: [Q-1711, Q-1713]
supersedes: []
---

# S-0216: Units placed after the game has started

## Summary

What placing a newly created vehicle or infantry unit on the map does once the game's set-up has
ended: the facings it starts with, where an infantry unit stands in its cell, which random draws
the placement takes and in which order, the entry test that can refuse it, and which behaviour is
current before the unit's first update. It answers Q-1711 questions 2 and 3 for units placed by a
play-time placement. Question 5, which placement the measurement records' units had and so
whether those answers hold for them, stays open (Open questions); claims 1 to 5 give only what
the game does for each kind of placement. So for the units of the measurement records the
answers to questions 2 and 3 wait on question 5. Questions 1 and 4 need an oracle run (Open
questions). It also answers Q-1713 questions 2 and 3 (claim 6) for the records' units, which
stood on their stated cells and whose houses were the original's computer players. Q-1713
question 1, how those units were destroyed or destroyed their enemy, stays open. Placement by
the map during the game's set-up is S-0009's and S-0208's (claim 5), and is restated here only as
a contrast. Skirmish only: campaigns and network games are out of scope.

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
   - Backing: `measurable` (scenario PT2 parts a and b for usable cells, part d for a cell holding a
     vehicle, part e for a full row). The object-present refusal and its open-gate exception are
     `unmeasured` here, as in S-0208 claim 3: no part of PT2 isolates them. The cell outside the
     usable area is `unmeasured`: a placement there that is refused (S-0209 claim 4, part 4a) shows
     no position, and whether one is refused is not established. PT2 part c records the outcome,
     including whether a draw is taken.
   - Confidence: medium for the usable-cell rule until PT2 runs: the table and its draw are
     S-0208 claim 3's, but only S-0614 claim 20 states that a placement makes this choice, and
     only for a returning parasite, as for claim 5. Medium for the failed placements and for the
     object-present refusal and its open-gate exception (as in S-0208 claim 3); low for the cell
     outside the usable area.

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
   claim 10) comes before its placement draws, at its creation. So a placing step that creates and
   places units one at a time takes, per unit and in its placing order: the creation draw, then
   that unit's placement draw if any. Every one of these draws falls before the first tick at which
   the placed units are updated when the step runs between ticks, and they shift every later draw
   of the scenario stream. The land recompute that putting the unit into its cell makes (S-0211
   claim 5) keeps its own effects and draws, which this claim does not restate.
   - Backing: `measurable` (scenario PT3)
   - Confidence: medium for the standing-position draw and its order after the creation draw,
     until PT3 runs, as claim 2 rates the rule. Medium for the visceroid draw at a play-time
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
     infantry type whose movement class cannot enter *rock* (S-0211 claim 10). The records M-0001
     and M-0002 do not show that their harness used an ordinary placement (Open questions, *Q-1711
     question 5*); the refusal is rated low below.
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
     infantry type whose movement class cannot enter *rock* (S-0211 claim 10), until PT5 part d
     runs and the measurement records' placement method is settled (Open questions, *Q-1711
     question 5*).
     Medium for the standing-position draw kept after a refusal, as claim 2 rates failed
     placements. Other outcomes as S-0209 rates each of them, and medium for the entry-table
     application to uncommon building and crushing exceptions, as in S-0614 claim 20.

6. **Units on cliff cells of no reach region (Q-1713).** The request concerns games of the
   measurement records M-0001 and M-0002 in which infantry units were meant to stand on cliff
   cells of no reach region whose land type is *rock* (Q-1502). In those games both houses were
   the original's computer players (M-0001), so every rule the original gives only to
   computer-controlled owners applied to the records' units, retaliation included.
   - Question 1. The records state that every unit stood on its stated cell, so the units meant for
     those cells stood on them, and the answers to questions 2 and 3 below describe them. How they
     were placed there is open (Open questions, *Q-1711 question 5*): an ordinary play-time
     placement would have refused them (claim 5). How they were destroyed, or destroyed their
     enemy, is open too (Open questions, *Q-1713 question 1*). S-0309 claim 7 (a hunter on open
     ground rejects such units in its map-wide search) and S-0215 claim 27 (a hunter that finds
     nothing stays put), which stay as they are, explain why such units' hunt searches give no
     advance; they do not by themselves establish how the recorded games ended.
   - Question 2. A hunting infantry unit on such a cell fires at an enemy on open ground only once
     that enemy is its target, and the enemy coming within its weapon's range does not by itself
     make it one. Its hunt search does not consider that enemy (S-0309 claim 7), and the
     limited-scan mark that turns a map-wide search into a range search is given only to vehicles
     (S-0309 claim 22). Retaliation has no reach-region test (S-0309 claim 18). For the records'
     units, whose houses were computer-controlled (M-0001), the original's behaviour list of
     S-0215 claim 18 does not hold back the unit (S-0215 claim 18, original only). So after the
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
   - Backing: `measured` (M-0001, M-0002) for where the records' units stood and (M-0001) for
     their houses' being the original's computer players. `measurable` for the rest: scenario PT5
     part d tests question 1's refusal of claim 5. For questions 2 and 3 the
     backing rests only on other specs' scenarios, as this spec has none of its own for them:
     S-0309 scenario A11, parts a and j, for the hunt search and for retaliation against
     cells of no reach region, and part c for the limited-scan mark; S-0215 scenario B7 for
     the behaviour list and the player return-fire setting. Retaliation from such cells is
     `unmeasured` here: it rests on S-0309 claim 18's conditions, which have no reach-region
     test.
   - Confidence: high for where the records' units stood and for their houses' control, as the
     records state. Low for question 1's refusal, as claim 5 rates it, until PT5 part d runs and the
     records' placement method is settled. High for the hunt search's rejection, as S-0309 claim 7
     rates cells of no reach region, and for the retaliation behaviour list, as S-0215 claim 18
     rates it; medium for retaliation against such cells, as S-0309 claim 18 rates its conditions,
     and for retaliation from them (unmeasured, as above); low for the switch of a
     computer-controlled hunter that already has a target, as S-0309 claim 25 part h rates the
     threat comparison that S-0215 claim 18 uses; low for the limited-scan route, as S-0309 claim
     22 rates when the mark is given.

## Formulas

> Body facing = turret facing = 256 × *v* facing units, *v* the integer facing value 0 … 255.
> Exact integer arithmetic.

> Standing-position row index *k* = R(0, 3), one ranged draw on the scenario random stream
> (S-0202 claim 4's range rule). Empty usable cell: *k* = 0 or 1 → right, *k* = 2 → left,
> *k* = 3 → bottom (S-0208 table O).

> Visceroid placement draw = R(0, 29) (S-0202 claim 10).

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

Ordering: for units created and placed one at a time, each unit's creation draw, then its
placement draw if any, in the order the placing step places them (claim 3).

## Edge cases and quirks

None found. A placed infantry unit not standing at the centre of its cell follows the same
standing-position rule a walker uses, and is not a fault.

## Import mapping

| Key | Feeds | Parse form | Absent-key default |
|---|---|---|---|
| vehicle type `SmallVisceroid`, `LargeVisceroid` | either one on: the placement draw R(0, 29) of claim 3 (as in S-0202) | boolean each | no |

## Measurement scenarios

All scenarios use stock data with no modifications except the test type data named in PT3 and
PT5, a small
purpose-made map, flat (except PT5 part d's cliff cell) and with all its cells usable unless a
part says otherwise, and a measurement
harness that places units after the game has started, before the first tick at which they are
updated, with stated cells and facing values. That harness is an exception to the glossary's
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
and places each unit before it creates the next, in this order: a vehicle of a type
with neither visceroid flag, an infantry unit, a second vehicle without the flags, a second
infantry unit, all on empty usable cells. Repeat with a vehicle type that has a visceroid flag
placed in the third slot instead (test type data). Run 3 seeds each. Record with the recorder
every scenario-stream draw from just before the first creation to just before the first update
of any placed unit. Draws taken by that update (such as an idle action's) are outside the window. The cells used hold
no tile that the land recompute of a unit entering its cell could start an animation on (S-0010
claim 19), so no tile animation's creation draws fall in the window; the result is a property of
such a map.
- *Expected:* for the first run, in order: creation draw, creation draw, R(0, 3), creation draw,
  creation draw, R(0, 3). For the second run, R(0, 29) right after the visceroid vehicle's
  creation draw. No other draw in the window. A draw there that another system of S-0202 claim 10
  accounts for, unrelated to the placed units, is reported and not counted as a placement draw.
- *Falsified if:* a draw is missing, an extra draw appears at a placement, or the order differs.

**PT4: behaviour before the first update (claim 4).** Place, at play time, a vehicle and an
infantry unit, each then given *hunt* as a queued behaviour, with an enemy unit far away on the
map. Record each unit's current and queued behaviour right after its placement and before the
*hunt* order, again after that order and before its first update, and at each of its first three
updates, the tick of its first *hunt* processing, and whether
the unit was ready to change (S-0215 claim 11) at its placement and at its first update's first
per-tick start.
- *Expected:* for a unit ready to change at placement, its resting behaviour current and nothing
  queued before the *hunt* order, and *hunt* queued after it. For a unit ready at its first update's first per-tick start, *hunt*
  current and processed in that same update.
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
  method (Open questions, *Q-1711 question 5*). Units of (a) to (d) standing on their refused
  cells then say nothing about claim 5.

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
- **Q-1711 question 5, for the records, and so questions 2 and 3 for them.** Claims 1 to 5 give
  what the game does for a play-time placement, and S-0009 and S-0208 (claim 5) what it does for a
  map entry. How the measurement harness behind M-0001, M-0002 and M-0003 placed those units, what
  it does when a placement is refused, and so how each unit came to stand on its cell, are facts
  about how those records were made. They belong in the records' Setup and are left open here. The
  records say that in every game every unit stood on its stated cell, and Q-1502 shows that some of
  those cells have land type *rock*, which the entry test of claim 5 refuses to an infantry type
  whose class cannot enter *rock* (S-0211 claim 10, S-0209 claim 4, part 4a). The lead's note on
  M-0001 and M-0002 that Q-1502 quotes says that the stock harness placed units on these cells like
  anywhere else. So those infantry units stood on the *rock* cells, and an ordinary play-time
  placement would have refused them (claim 5): either the harness did not place them that way, or
  claim 5's rock refusal is wrong for those cells. Which one is open; PT5 part d tests the refusal
  directly. PT2 part c records the outcome of a placement outside the usable area, PT5 tests the
  stated entry refusals, and any retry a harness makes elsewhere is reported separately (PT2, PT5).
  Until the records' placement method is known, the answers to Q-1711 questions 2 and 3 (claims 1
  to 3) hold for those records' units only if they were placed by a play-time placement; for map
  entries, S-0206 and S-0208 claim 5 give them.
- **Q-1713 question 1, for the records.** In the stock games of M-0001 and M-0002, how are the
  infantry units on the cliff cells destroyed, or how do they destroy the enemy: by enemies that
  attack them from open ground, by their own fire at enemies on open ground, or after they leave
  the cliff cells? Claims 5 and 6 rule out none of the three:
  - Attack from open ground stays possible. A hunter on open ground rejects them in its hunt
    search (S-0309 claim 7), but can make one its target by retaliation after that unit hits it,
    which the records' computer-controlled houses allow (claim 6, question 3), or, for a vehicle
    with the limited-scan mark, by the range search (S-0309 claim 22).
  - Their own fire stays possible, but only at an enemy that has become their target. Their hunt
    search never makes one so; retaliation after that enemy hits them can (claim 6, question 2).
  - Leaving the cells is neither established nor ruled out. S-0215 claim 27 keeps a hunter that
    finds nothing where it stands; what a unit on such a cell does after gaining a target by
    retaliation that is out of its weapon's range is not covered here.

  Which of these ends the recorded games needs an oracle run (backlog). How the units came to
  stand on the cells (*Q-1711 question 5*, above) does not decide it: claim 6's answers to
  questions 2 and 3 hold however they were placed.
- **A failed placement.** What a placing step does with a unit whose placement fails (claims 2
  and 5): whether it tries another cell, keeps the unit off the map or removes it. This
  belongs to the step, not to the placement rule; PT2 parts d and e and PT5 record what the
  measurement harness does.
- **Tile animations started by the placement.** Where the creation draws of a tile animation that
  the land recompute of a placement starts (S-0211 claim 5, S-0010 claim 19) fall relative to the
  placement draws of claim 3. PT3's map has no cell that could start one.
- **Infantry facing.** Which facing an infantry unit placed at play time starts with (claim 1
  covers vehicles only).
- **Above the ground.** Which standing position an infantry unit gets when it is placed at play
  time above the ground of its cell, for example on a bridge deck (claim 2).
