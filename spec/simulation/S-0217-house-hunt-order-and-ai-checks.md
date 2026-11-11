---
id: S-0217
title: House-wide hunt order and the computer player's periodic checks (original only)
topic: simulation
revision: 22
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-11
measurements: []
requests: [Q-2951, Q-2986]
supersedes: []
---

# S-0217: House-wide hunt order and the computer player's periodic checks (original only)

## Summary

What the house-wide hunt order does to a house's units and garrisons, and when a computer-controlled
player house of the original gives that order to itself in a skirmish: a check every 106 to 112
ticks, one delay draw each. A check gives the order, after giving the house's buildings sell orders,
when the house has no factory building and is not held (claim 3: past tick 900, with no building of
it hit in the last 900 ticks), and it gives the order at every check to a house with no building of
its own (occupied civilian buildings aside) and no construction vehicle. The checks are the
original's computer player's; in our engine only test-only computer-controlled player houses
(S-0215) make them.

## Terms

- **House-wide hunt order.** One house-level event that sends every unit of one house hunting
  (claim 1). Map action code 6 (S-0011) gives it to the house its number slot resolves to; the
  checks of claims 2 to 4 give it to the checking house.
- **Check.** One periodic strategy step of a computer-controlled player house in a skirmish of the
  original (claim 2). It takes place in that house's house update (S-0203, row 7).
- **Last base hit tick (*L*).** Per house, the tick at which a hit that has an attacker last
  reached a building of the house (claim 3). It is 0 at game start.
- **Occupied civilian building.** A civilian garrison building that a house owns only because its
  own infantry occupy it (S-0508 claim 7).
- **Creation order:** as S-0309 Terms defines it (the object created first comes first).
- **Factory building, building factory, construction vehicle.** A *factory building* is a building
  whose type has any recognised factory kind other than the no-factory value (S-0003 claim 21), the
  non-producing kinds included. A *building factory* is a factory building whose type makes
  buildings, as S-0522 claim 5 uses the term. A *construction vehicle* is a vehicle whose type
  deploys into a building factory. This spec does not use the word "construction yard".
- **Order of ownership.** For one house, the order in which it came to own its buildings, the one
  owned first coming first.
- **Computer-controlled, test-only computer-controlled player house, team member, AI house:** as
  S-0215 Terms and the glossary define them.

## Behaviour

**Houses in our engine.** Claims 2 to 4 describe the original's computer player only. They are
**Original only (note for AI designers)**: our AI houses and human-controlled houses make no checks,
and our AI may reproduce their effect by ordinary orders. A test-only computer-controlled player
house (S-0215 Terms) makes the checks of claims 2 to 4, with the effects those claims state; the
headless arena marks both its houses this way. The original's money step of Open questions is not
required of it beyond what claim 4 states. The neutral and
special houses make no checks, in the original or in our engine. Claim 1 is a rule of our engine for
every house that receives the order.

1. **House-wide hunt order.** When a house receives the house-wide hunt order, in the same tick,
   each of its actors that is on the map (not inside a building or a transport) is reached once:
   - a mobile actor (infantry unit, vehicle or aircraft) leaves its team, if it is a team member,
     and has *hunt* queued without an immediate start (S-0215 claim 7, which also says what that
     changes for a unit that has *hunt* current). A queued *hunt* becomes current as S-0215 claim
     11 gives (at a per-tick start or the late start),
   - a building with occupants has them evicted, with hunting and without inside fallback (S-0508
     claims 11 and 13): each occupant that can be placed comes out with *hunt* as its next
     behaviour, and one that cannot be placed is removed (S-0508 claims 11 and 12, which also
     cover a building with no exit cell),
   - any other building is not changed.

   This list leaves out the original's destruction of some units (Open questions). Units inside a
   transport are not reached, and occupants of a building are reached only through that building
   (second item), and only when the house owns that building at that moment (S-0508 claim 7 gives
   the owner). Its buildings with occupants are reached in reverse creation order, the one created
   last first, and the order shows through the evictions, whose scatter requests take draws (S-0508
   claim 11, S-0208 claims 11 to 13): with two occupied buildings, the occupants of the one created
   later are put out first and take the earlier draws. This spec states no order among the other
   actors, or between them and those buildings. Queueing *hunt* takes no draw, and the order could
   show only where the *hunt* replaces a walker's queued *enter* (S-0215 claim 7, S-0208 claim 6)
   next to an eviction's scatter, which is not established here (Open questions).
   - Backing: `measurable` (scenario H1 and S-0508 scenario G13 for the *hunt* and the evictions, H3
     part b for the order among buildings). For a vehicle or an aircraft, and for whether a mobile
     actor's *hunt* is queued or started at once, the branch is `unmeasured`: H1 records only the
     occupier side's infantry and accepts *hunt* current or queued, and S-0508 scenario G13 sees
     only evicted occupants. The team departure is `unmeasured`: the arena's houses have no teams
     (S-0215 Terms), and no scenario here gives the order to a team member. That units inside a
     transport are not reached is `unmeasured`: no scenario here has a transport.
   - Confidence: medium; low for the team departure (as S-0508 claim 11 rates its own)

2. **Checks (original only).** In a skirmish of the original, every computer-controlled player house
   makes a check in its house update of tick 0 and then at each tick at which its check delay has
   run out. A check at tick *t* sets the next check to tick *t* + 105 + *R*, where *R* is one draw
   R(1, 7) on the scenario stream (S-0202), taken at the end of the check. So consecutive checks are
   106 to 112 ticks apart. This delay draw is the check's only draw of its own: the effects of an
   order the check gives take their own draws (claim 1). This spec does not fix where a check falls
   among the other draws of its house update. Within row 7 the houses' checks and draws come in the
   row's house order. A human-controlled house, the neutral house and the special house make no
   checks. In our engine, when a test-only computer-controlled player house's check and the raise of
   its tags (S-0011 claim 35) come in one house update, the check comes after the raise.
   - Backing: `measurable` (scenario H1 for the spacing, H3 part a for the single draw of a check
     whose order evicts no occupant, in runs whose house updates without a check take no raw values,
     and H3 part c for the house order: the stream's states at the start and end of each house's
     update show which update came first). That human-controlled, neutral and special houses make no
     checks is `unmeasured`: H1's and H3's player houses are both computer-controlled, and the
     neutral house there has no units. The draw's place at the end of the check, after the draws of
     the order's evictions, is `unmeasured`: no scenario here tells the delay draw from the eviction
     draws within one house update. Where the check falls among the update's other draws is not
     stated, and so not measured: H3 part a judges only runs in which the house's update takes no
     other raw values. The check's place after the tags' raise is a rule of our engine, `unmeasured`
     (the original's order of the two is an open question).
   - Confidence: medium

3. **No factory and no recent hit (original only).** At a check at tick *t*, the house is *held*
   when *t* ≤ *L* + 900. *L* becomes the current tick whenever a hit that has an attacker reaches a
   building of the house, whatever the attacker's house: an enemy, an ally or the house itself. *L*
   is set whatever the hit then does (S-0302): a hit whose final amount is 0, a healing hit, at full
   health or not, and a hit that a row of S-0302 claim 8 stops, at any stage, all set it, except in
   the two cases next. The building's own damage hits it, and so changes *L*, only when the
   building's type is marked to damage itself. A building whose type can undeploy into a vehicle and
   covers one cell does not change *L*. A hit that its source does not deliver sets nothing: a
   detonation leaves out a building with no health (S-0303 claim 9), so the no-health row of S-0302
   claim 8 sets *L* only for a source that delivers such a hit (S-0302 claim 8, *No-health
   targets*). When the house is not held and owns no factory building of any kind on the map
   (building factories included), whatever construction vehicles it owns, the check does both of the
   following, in this order, in the same house update:
   - every building the house owns on the map that has health left, occupied civilian buildings
     included, receives a sell order (S-0408 claim 4, which decides whether it is sold), one after
     another in the house's order of ownership (Terms), which is not the order in which claim 1
     reaches occupied buildings,
   - the house receives the house-wide hunt order (claim 1).

   A house none of whose buildings has been hit is therefore held at every check up to tick 900,
   and the first check that can give this sale and order is the first one after tick 900. A hit
   at tick *h*, by any attacker, moves that to the first check after tick *h* + 900.
   - Backing: `measurable` (scenario H2: that the sale comes, by the end of the first check after
     the hold, for an enemy's hit in part b and an ally's in part c; a run whose first check after
     the hold comes within 6 ticks of the hold's end does not tell that check from the next one,
     since both can fall in H2's window). That the sale starts at a check, and not between checks,
     is `unmeasured`: H2 records only the sale's tick, and its window holds ticks between checks.
     The parts listed next are `unmeasured`. That a house that owns a factory building gets neither
     the sell orders nor the order from this claim: no scenario here gives a computer-controlled
     house a factory building. That the house then receives the house-wide hunt order: H2 does not
     observe *C*'s units, since the money step of Open questions may already send them hunting, and
     H1's houses own no building, so their checks give the order by claim 4. That no sale comes
     from this claim while the house is held: a sale before the hold ends cannot be told from one
     by the money step of Open questions, so H2 leaves such runs inconclusive. The boundary tick
     *L* + 900, for the same reason. That the sell orders come before the order: both act in one
     house update, and the order would show in the stream only when a building removed at its sell
     order (S-0408 claim 4, a firestorm-wall type) has holders whose drops take draws (S-0309 claim
     27) against the evictions' scatter draws (S-0508 claim 11), which no scenario here has. The
     sell order to occupied civilian buildings: a computer-controlled house cannot be ordered to
     occupy one, and whether its units occupy one by their own search, and when, is not predicted
     here (S-0309), so no scenario is sure to have one at a check that is not held; H1 records such
     sales without judging them. Hits from the house's own units or from the building itself, and a
     building that can undeploy into a vehicle, are `unmeasured` too: no H2 part has one. So are a
     house that owns a construction vehicle (H2's house owns none) and the order of the sell orders
     among the buildings, which shows in the same way (H2's house has one building). Hits that
     leave the building's health unchanged are `unmeasured`: a hit whose final amount is 0, a
     healing hit at full health, and a hit that any row of S-0302 claim 8 stops. H2 would show
     whether such a hit sets *L* only through the sale's tick, and a sale at the first check after
     tick 900, the one that a hit not setting *L* would give, cannot be told from a sale by the
     money step of Open questions. A healing hit on a damaged building is `unmeasured` too: no H2
     part has one.
   - Confidence: medium; low for the order of the sell orders

4. **A house with no building (original only).** A house that owns no building other than
   occupied civilian buildings, and no construction vehicle, receives the house-wide hunt order at
   every check, held or not. At a check at which it is not held and owns no building of a factory
   type (claim 3, occupied civilian buildings included), claim 3 also applies to it, so its
   occupied civilian buildings receive the sell order first. In the headless arena, where both
   houses start with neither, every check of each house gives the order, and the occupiers of
   occupied civilian buildings come out with *hunt* at each check of their house (claim 1).

   Between those checks, the arena's occupier-type infantry follow S-0508's rules unchanged. Which
   target a unit's search picks is S-0309's (claims 17 and 25 part e). A unit that has as target a
   civilian garrison building it may occupy (S-0508 claim 1, which leaves out a full one) goes for
   it in *capture* (S-0215 claims 23 and 27). Those rules force no *capture* toward a full building,
   such as the target a refused unit keeps (S-0508 claim 4), though S-0215 claim 28's rows can still
   give such a unit that building as destination. Units enter it in order of arrival until it is
   full, and those that arrive after that are refused (S-0508 claims 1 and 4). So in the arena a
   unit enters only after its *capture* toward the building has begun, and an occupant comes out at
   the first check of the building's owner at which that house owns the building (claim 1), or is
   removed if it cannot be placed. A refused unit has *hunt* queued at its house's next check
   (S-0215 claim 7 says what that changes for a unit that has *hunt* current), and S-0215 decides
   what it then does (claims 11, 27 and 37), which can leave it standing while the building is in
   range.
   - Backing: `measurable` (scenario H1 for checks at which the house is held and for the occupiers'
     entries and refusals). H1 judges the order only through a check's evictions, so a held check
     after tick 0 at which the house owns no occupied building is `unmeasured`: H1 records the
     behaviours there but does not judge them. What a refused unit does after its *hunt* is S-0215's
     and is not tested here. Checks at which it is not held are `unmeasured` for the same reason as
     claim 3's occupied civilian buildings. That a house that owns a construction vehicle does not
     get the order at every check is `unmeasured`: no scenario here gives a computer-controlled
     house one. The order given at the check of tick 0 is `unmeasured`: in the arena every unit is
     placed with *hunt* already, so that order has no effect that H1 can see.
   - Confidence: medium; low for the check of tick 0

## Formulas

> next check tick = *t* + 105 + R(1, 7), integer ticks; *t* is the tick of the current check.

> held at tick *t* ⇔ *t* ≤ *L* + 900; integer ticks.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Check base delay | 105 | ticks | added to the draw of claim 2 |
| Check draw range | 1 to 7 | ticks | one R(1, 7) on the scenario stream per check |
| Base hit hold | 900 | ticks | claim 3 |
| Last base hit tick at game start | 0 | tick | claim 3 |

## State and ordering

| State | Event | Result |
|---|---|---|
| any computer-controlled player house in a skirmish | its check delay has run out, in its house update | check (claims 3 and 4), then one draw sets the next check (claim 2) |
| not held, no factory building on the map | check | sell order to every own building on the map that has health left, then the house-wide hunt order |
| no building except occupied civilian buildings, no construction vehicle | check | the house-wide hunt order (after the sell orders of claim 3 when claim 3 applies) |
| occupied civilian building with occupants of the house | house-wide hunt order | occupants out with *hunt* next, building no longer occupied (S-0508 claims 11 and 13) |
| mobile actor on the map | house-wide hunt order | leaves its team; *hunt* queued without an immediate start |

Ordering as observable consequences:
- The order's evictions happen in the house update of the check's tick, after that tick's turn pass
  (S-0203 claim 8), so at the end of that tick the occupants that could be placed are on the map and
  those that could not have been removed (claim 1), and the building returns to the civilian house
  at its next turn when S-0508 claims 7 and 8 give that: it is of a tech level that changes owner
  through occupation, and has no occupants at that turn.
- A unit on its way to a garrison in *capture* when the order comes is not ready for the queued
  *hunt* while it moves (S-0215 claim 11), so *capture* stays current and the unit can still enter
  the garrison with *hunt* queued (S-0215 claims 7 and 9; S-0508 claim 5). If it has entered, the
  next check of its house that gives the order (claims 3 and 4) puts it out with *hunt* as its next
  behaviour (claim 1).

## Edge cases and quirks

- A hit sets *L* whatever the attacker's house, and whatever the hit does, so damage from an ally
  or from the house's own units, a hit that does no damage and a healing hit all hold the sale back
  as an enemy's damaging hit does. No switch: this is the original's computer player,
  which our AI does not copy.
- A house that owns only occupied civilian buildings, none of a factory type, counts as having no
  factory building, so claim 3 gives those buildings a sell order at each check that is not held.
  Whether an occupied civilian building is then sold is S-0408's (claim 4). No switch: this is
  the original's computer player, which our AI does not copy.

## Import mapping

The check delay, the draw range and the hold are constants of the original. The building types
that do not change *L* (claim 3) are those whose undeploy-into field (`UndeploysInto`, a unit type
name; absent: none) names a vehicle type and whose foundation (`Foundation`, building art) is one
cell. A building's own damage changes *L* only when its type's self-damage flag is set
(`DamageSelf`, a boolean, absent: no).

## Measurement scenarios

**H1. Arena checks, entries and garrison evictions (claims 1, 2 and 4).** Setup: the arena set-up
of M-0001, which M-0002 shares (both as M-0017's Setup gives it), side *B*, for the pairing that
Q-2951 names first, stock data, both houses computer-controlled with no buildings and no
construction vehicles, units placed with *hunt*. Record each tick: for each unit of the occupier
side, its current and queued behaviours, whether it is on the map, its position, whether it is
moving, and its target; for each civilian garrison building, its owner, its occupant count, its
health and whether it starts a sale. Procedure: play one seed to the end; repeat for the other
two pairings that Q-2951 names. Observable: for each unit of the occupier side, the tick at which
its behaviour first becomes *capture* and the building it goes for, and the tick at which it
enters a building or is refused; the ticks at which a building occupied by the occupier side has
all its occupants come out in one tick (an eviction tick); and the ticks at which the building's
owner becomes the occupier side's house and returns to the civilian house. Expected: units that
arrive at a full building are refused; every entry comes at or after the tick at which the
entering unit's behaviour became *capture* toward that building, in the pairing of Q-2951
question 2 too; whether and when each unit first goes for a building is recorded for Q-2951
questions 1 to 3 (claim 4 does not predict it); when the occupier side owns an occupied building
at the end of tick 105, the first eviction tick is at most 112; each later eviction tick of the
same house is 106 to 112 ticks after a previous one, or an exact sum of such steps when the house
owned no occupied building at a check; at every eviction tick every occupant that can be placed
comes out with *hunt* as its next behaviour, and every refused unit of that house on the map has
*hunt* current or queued at the end of that tick; the building's owner returns to the civilian
house at the building's next turn when it has no occupants then (S-0508 claims 7 and 8). Three
kinds of eviction tick are recorded and left out of the expectations, the spacing and the
falsifiers: one in the tick at which the building is destroyed (S-0508 claim 13), one after tick
900 at which the building starts a sale or is sold (claim 3's sell order, judged by S-0408 claim
4), and one at the building's own turn while its health is low enough for S-0508 claim 9's
eviction without hunting. A run in which no building of the occupier side becomes occupied tests
only the entries and refusals; play further seeds of that pairing, up to 5, until one has an
eviction tick. Falsified by an entry with no *capture* toward that building begun before it in
the same or an earlier tick, by an occupant still inside 113 or more ticks after the later of its
own entry and the latest tick at which its building's owner became its house, by an eviction
whose spacing from the previous one is not such a sum, or by a refused unit on the map with
neither *hunt* current nor *hunt* queued at the end of an eviction tick of its house.

**H2. Sale and hunt without a factory (claim 3).** Setup: a tiny skirmish map; computer-controlled
house *C* with one completed building of a type that has a build-up sprite, is not a factory and
cannot undeploy into a vehicle, no factory building and no construction vehicle, and a few armed
units placed in *guard*; a human-controlled enemy house *D* whose single armed unit stands within
its weapon range of *C*'s building and out of sight of *C*'s units. Parts: (a) nothing damages
*C*'s building; (b) at a fixed tick between 100 and 200 the harness orders *D*'s unit to attack
*C*'s building, with force-fire held (S-0213), and orders it away once the building's health first
drops; *h* is the tick of that first drop, the tick of the hit; (c) as (b), but *D* is allied to
*C*. A run in which the building's health does not drop at the hit, or *D*'s unit hits it again, is
void, and so is a run in which the building's health drops at any other time: in (a) at all, and in
(b) and (c) before the harness's order. Observable: the tick at which *C*'s building starts its
sale. The movement of *C*'s units is not observed: the step of Open questions may already send them
hunting before tick 900. Expected, with *S*(*t*) the tick of the first processing of a sell
behaviour ordered in the house update of tick *t* (S-0408 claim 4): (a) the sale starts from
*S*(901) to *S*(1012); (b) and (c) the sale starts from *S*(*h* + 901) to *S*(*h* + 1012). A sale
that starts before *S*(901) in (a), or before *S*(*h* + 901) in (b) or (c), is recorded for the
open question on the money step and leaves that part inconclusive, since that step's sales are not
told apart from claim 3's. Falsified by no sale, or a sale that starts after *S*(1012), in (a), or
by no sale, or a sale that starts after *S*(*h* + 1012), in (b) or (c).

**H3. One draw per check, and the order of evictions (claims 1 and 2).** Setup: as H1. Record the
scenario stream's state at the start and end of each house's house update, and, at each eviction
tick, the stream's state before and after each building's occupants are put out and the cell each
occupant comes out in. Raw values are counted by replaying the stream between the recorded states
(S-0202). A ranged draw can take none, one or several, so part (a) replays the delay draw by
S-0202's rule instead of counting it. Parts: (a) checks at which the order evicts no occupant; (b) a
check at which the order evicts the occupants of two buildings of the house; play further seeds of
H1's pairings, up to 10, until one run has such a check, and with none part (b) is inconclusive; (c)
the checks at tick 0, which both houses make (claim 2). For two buildings placed by the map, this
scenario takes the one whose map entry comes later as the one created later: the map's buildings
enter play in the order of their entries (S-0203 claim 3), and nothing creates them at another
moment. Expected: (a) at such a check, the state at the end of the house's update is the one reached
from its start by one R(1, 7) draw replayed by S-0202's rule, and that draw's value *R* gives the
next check's tick as *t* + 105 + *R*; (b) the occupants of the building created later are put out
first and take the earlier draws; (c) the update of the house that comes first in row 7's house
order (S-0203) ends before the other's starts, and its delay draw is the earlier one. Part (a)
judges only runs in which the house's updates at ticks without a check take no raw values, and is
inconclusive in any other run, since claim 2 does not fix where the check falls among the update's
other draws. Falsified by a check in (a) whose end state is not that replay's, by a next check tick
other than *t* + 105 + *R*, or in (b) by the building created earlier having its occupants put out
first, or in (c) by the other house's update or delay draw coming first.

## Open questions

- Q-2951 is answered in part, by this spec's claims 1, 2 and 4 together with S-0508 claim 4 (r39).
  Question 1, second part: the checks of their house give refused units *hunt* again and put the
  occupiers out (claims 1, 2 and 4). Question 1, first part, and questions 2 and 3: which target
  each unit's search picks first is S-0309's (claims 17 and 25 part e), and this spec does not
  predict it for the named pairings. Claim 4 gives what follows: a unit that goes for a building in
  *capture* enters only after that *capture* has begun, or is refused at a full building, and comes
  out at the first check of the building's owner at which that house owns the building, or is
  removed (claim 4). Scenario H1 records, for the three pairings, whether and when each unit goes
  for a building, enters or is refused. What a unit with *hunt* then does, and so whether it goes
  for the other group (question 1), is S-0215's (its *hunt* rules), which this spec does not
  restate.
- In the original, a check also gives the order, through a step that sells the house's planned
  base buildings to raise money, in conditions this spec does not state (claim 4 states the result
  only for a house with no building and no construction vehicle). Which houses with buildings or a
  construction vehicle that step reaches is not established here.
- Where a check falls in its house's update against the raise of the house's tags (S-0011 claim 35),
  when both come in one tick, is not established here for the original. Claim 2 fixes it for our
  engine's test-only houses.
- Where a check falls among the other draws of its house's update is not established here for the
  original, and this spec leaves it free for our engine.
- Whether the order reaches a walker whose queued *enter* its *hunt* replaces (S-0208 claim 6)
  before or after an eviction of the same order whose scatter can reach that walker is not
  established here.
- The original's order also destroys some units of a computer-controlled house that another house
  controls permanently; which ones, and how, is not specified here.

## Changes for other specs

- **S-0508 claim 13:** its house-wide hunt row names this spec's claim 1 and the checks of claims 3
  and 4 as causes (S-0508 r39).
- **S-0011:** the action-6 table row can point to this spec's claim 1 for the order's effect on
  units outside garrisons.
- **S-0203 row 7:** its statement that the computer player's strategy work is original-only
  context no longer covers test-only computer-controlled player houses: for them the checks of
  claim 2 and their draws are engine requirements, taken in that house's update.
- **S-0414 claim 18:** it owns the timing within a house's update (S-0011 claim 35), so it should
  place a test-only computer-controlled player house's check after the raise of that house's tags
  (claim 2).
