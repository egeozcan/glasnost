---
id: M-0019
title: Unit arena, the records' placements at tick 0 (standing positions, raw values, refused cells)
topic: simulation
specs: [S-0216, S-0208, S-0202, S-0506, S-0402]
requests: [Q-2277]
verdict: new-finding
value-visibility: committed
measured-on: 2026-11-02
method: harness
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-02
---

# M-0019: Unit arena, the records' placements at tick 0

This record reads the state at tick 0 of the stock replays of M-0016 and M-0017, which stand as
the stock games of M-0001 to M-0003. It answers where the records' infantry stood in their cells
at tick 0 and which of the scenario stream's raw values (S-0202) their placement took. In the
all-infantry games every placed infantry unit stood at a side position, never at the centre or top
position, and a model in which each object takes the raw values of S-0216 claim 3's draws for it,
in creation order, fits every position and the count of raw values: each position is the one
S-0216 claim 2 gives for the draw at the last of that object's raw values (for several infantry
units on one cell the record gives no finding of its own). Over all 1860 games the raw values
taken before tick 0, after the first *N*₀, are as many as that model gives the objects created,
*N*₀ being a number that, in the all-infantry games, depends on the seed only. Some games hold
spawned objects (Setup). For every unit, the number of the set-up's objects left off the map
before it equals the place *p* of its cell in S-0216 claim 8's order (0 for its stated cell), and
each of those objects is of that unit's type. That each refused cell made a new object is the
inferred reading of that count.

## Setup

- Game build: stock Yuri's Revenge 1.001, executable hash verified; run under the oracle harness
  (PROVENANCE §7).
- Data: stock, no mods, no loose rules or art files.
- Games: the 1860 replays of M-0016 (276 games) and M-0017 (1584 games), each replayed once from
  its archived setup and seed. Map, players, game settings, budget, stated cells, facings, the
  *hunt* mission and the end rule are those of M-0001, as M-0016 and M-0017 give them. Seeds 0 to
  5 are the seeds of M-0001's naming (side A with seeds 0 to 5, side B with seeds 0 to 5).
- The five gameplay changes of the harness that M-0001 to M-0003's archived games ran with (a
  siege rule for long-range artillery, a change to Magnetron carrying, changes to the computer
  houses' production, extra defences, and an expansion and factory queue) are all off, as in
  M-0016 and M-0017. The house director is off for every house.
- What is not stock, in these replays: M-0016's list, with the recorder below added. The second
  item's neighbouring cell is worded as in M-0006 and in M-0016's *Placement* bullet. M-0001's,
  M-0003's and M-0017's Setup, and M-0002 after M-0017, say "free neighbouring cell", which M-0006
  finds does not hold.
  - no menus: the skirmish setup comes from a settings file
  - units are created on their stated cells before tick 0, and a unit that its stated cell does
    not take is placed on a neighbouring cell, which can already hold a unit (M-0006, M-0016)
  - the map's four bunkers, owned by a neutral house, are in play from tick 0
  - the computer players' starting construction vehicles are removed. That this record's
    *starting objects* (below) are those vehicles is a reading (*Placement raw values*)
  - the two computer players are not allied with each other
  - the idle observer house's objects cannot be targeted or damaged
  - the run ends by M-0001's rule
  - a periodic sampler reads state only
  - a state recorder reads state only. M-0016 and M-0017 do not describe it: the harness ran it
    during their replays, and this record is the first to use it (*Tick 0* below, Evidence)
- The all-infantry subset: the 192 games of M-0002 in which both sides are infantry, 16 pairings
  (BRUTE, INIT, VIRUS and YURI against E1, E2, GGI and SHK), 12 games each.
- *Tick 0* is the first tick of play, in the committed records' numbering: units are created
  before tick 0 (M-0016, M-0017). For this record the state recorder gives, for every unit,
  aircraft and building of every house at tick 0 (after tick 0's update): its identifier, its
  type, its kind (infantry, vehicle, aircraft or building), its house, whether it is on the map
  and its position in leptons, and the same at the next tick, the only other tick it gives. It
  also gives the scenario stream's state at the start of tick 0.
- The set-up input lists the unit entries in order. In every game it lists the entries of the
  record's row type first, on whichever side the row type plays: a fact of the input.
- *Creation order* here means identifier order: the order of the numbers the recording gives the
  objects. That this is the order in which they were created is a reading. For every placed unit
  of every game identifier order matches the order of the set-up's unit entries, which puts the
  row type's units first. Step 5 tests one other order against the fit.
- *Raw value* and the stream sequence x₀, x₁, x₂, … are S-0202's. A raw value's *place* is its
  number in that sequence, counted from the seeding (x₀ is place 0). A ranged draw may take more
  than one raw value (S-0202 claim 4), so a count of raw values is not in general a count of
  draws. This record counts raw values throughout. It gives no count of draws.
- *The model's count* *c* of an object is the number of raw values that the draws S-0216 claim 3
  gives it take: its creation draw (S-0202 claim 10), the creation draws of the objects its
  creation makes (S-0506 claim 1, S-0402 claim 19), each one raw value, and its placement draw
  if any, R(0, 3) for an infantry object (one raw value) or R(0, 29) for a vehicle of a type with
  either visceroid flag (as many raw values as S-0202 claim 4 takes at its place in creation
  order). An object of the set-up off the map at tick 0 is counted as S-0216 claim 8 counts a
  refused try: its creation draws and, for infantry, one R(0, 3). The type data the model reads
  are those of the stock rules the games loaded, kept local.
- *The two houses* are the houses of the game's two players, as the settings file sets them up.
- *The set-up's objects* are the objects of the two houses, on the map or off it, of a type that
  one of the set-up's unit entries names.
- Every other object of the two houses at tick 0 is classed by its creation order alone. One
  created before the set-up's first object is a *starting object*. One created after it is a
  *spawned object*. Both classes are left out of every step below except step 7. That the
  spawned objects are objects that a set-up object's creation makes (S-0506 claim 1's slot
  objects, or an initial slave group of S-0402 claim 19) is a reading: they are made during the
  set-up and of a type no entry names. The model's count takes those objects' creation draws from
  the set-up objects' types, not from this class, so the record classes them by creation order
  only and relies on that reading for no count.

## Procedure

1. For every game, take the set-up's objects present at tick 0, in creation order (identifier
   order). Those on the map give a cell (position ÷ 256, truncated) and a position within the
   cell (position mod 256), named by S-0208 table P.
2. For every game, count the raw values of the scenario stream taken from the seeding to the
   state at the start of tick 0: replay the stream's generator (S-0202) from the game's seed until
   its state equals the recorded one. Call the count *N*.
3. For the all-infantry games, take the *N* raw values taken before tick 0, at places 0 to
   *N* − 1, and for each place the result R(0, 3) of a draw made at that raw value. Such a draw
   takes exactly that one raw value (S-0202 claim 4). Number the set-up's objects in creation
   order from 0, objects off the map included. From a start *a*, object 0 is given the *c* places
   *a* to *a* + *c* − 1, *c* being its model's count (Setup), and each later object the next *c*
   places after the previous object's, with its own *c*. An object's position is read from the
   last of its places. Each object on the map must stand where S-0216 claim 2 puts it for that
   result: the first free position of row R*k* of S-0208 table O, with *k* the result, given the
   units already standing on the cell. An object off the map only takes up its places: it sets no
   condition.
4. Try every start *a* from 0 upward for which every place given to an object lies from 0 to
   *N* − 1. Repeat the search two ways, each with the position read from an object's last place:
   - every object given one place fewer than its model's count (*c* − 1)
   - every object given one place more than its model's count (*c* + 1)
5. For the 96 side-B all-infantry games, repeat steps 3 and 4 with a second order: side A's
   set-up objects first, then side B's, each side's in identifier order. An object off the map
   keeps its place just before the unit of its side that follows it. (In the side-A games this
   order is identifier order itself.)
6. For every game and every unit of its set-up in order, take the set-up's objects off the map
   created after the previous unit and before that unit (for the first unit, those created
   before it): their number, their types and their houses, and whether any of their identifiers
   is among the objects the recorder gives at the next tick, of any house. Spawned objects are not
   among them. Find the place
   *p* of the unit's cell in S-0216 claim 8's order: 0 for its stated cell, then 1 to 8 for its
   eight neighbours in that claim's order.
7. For every game, sum the model's count *c* over the set-up's objects, on the map or off it, and
   compare the sum with *N* − *N*₀, taking for every game the *N*₀ that the fit of steps 3 and 4
   finds for its seed in the all-infantry games (*Placement raw values* below). Count the starting
   objects and the spawned objects of the two houses at tick 0. For the starting objects,
   take their houses and kinds, whether each is on the map at tick 0, and whether the recorder
   gives its identifier at the next tick. For the spawned objects, take whether each is on the map
   at tick 0.

## Result

**Positions read at tick 0.**

| Quantity | Games | Value |
|---|---|---|
| Games in which a placed infantry unit stands off table P's positions at tick 0 | 576 | none |
| Games in which a placed infantry unit stands at the centre position at tick 0 | 576 | none |

The 576 games are those of the 1860 whose set-up names an infantry type: M-0002's 288 games and
the 288 of M-0001 against E1, E2, GGI or SHK. Each of them holds placed infantry at tick 0.
The top position is reported for the 192 all-infantry games only (below).

In the all-infantry games the fit below gives each placed infantry unit the position its own last
place gives, so there the positions read after tick 0's update are the placement positions. In
the other games that hold infantry this is a reading from the exact table P positions: no unit
stands between them. A vehicle may begin to move in tick 0's update. That the cell a vehicle is
read on at tick 0 is the one it was placed on is a reading. The record uses a vehicle's cell only
in step 6.

**Standing positions (Q-2277 q1).**

| Quantity | Games | Value |
|---|---|---|
| Placed infantry at the centre or top position at tick 0 | 192 | none |
| Placed infantry at the right, left or bottom position | 192 | all |
| Games with exactly one start *a* fitting every object, the model's count per object | 192 | 192 |
| Games with any start fitting, one raw value fewer per object | 192 | 0 |
| Games with any start fitting, one raw value more per object | 192 | 0 |

- Two facts support the fit. Within the counts tested (the model's count per object, and one raw
  value fewer or more for every object) and over the whole range of candidate starts, exactly one
  start fits in each game, with the model's count. And that start also gives
  the game's recorded count *N* (*Placement raw values*, below): the last object's position place
  is the last raw value taken before tick 0.

- Several infantry units on one cell (q1's case): this record gives no finding of its own for it.
  Step 3 applies S-0216 claim 2's usable-cell rule with the units already standing on the cell,
  but whether any game of the subset has such a cell is not given, since with M-0006's moves it
  would bound the groups' sizes. So the fit does not test that case on its own. S-0216's scenario
  PT2 part b tests it.

**Placement raw values (Q-2277 q2).**

| Quantity | Games | Value |
|---|---|---|
| *N* = *N*₀ + the sum of *c* over the set-up's objects, all-infantry games, *N*₀ the game's own fit | 192 | 192 |
| *N* − *N*₀ = the sum of *c* over the set-up's objects, all games, *N*₀ by seed from the all-infantry fits | 1860 | 1860 |
| Games in which both houses hold starting objects, all vehicles, off the map at tick 0 and none given at the next tick | 1860 | 1860 |
| Games in which a spawned object is on the map at tick 0 | 1860 | 0 |

- *N*₀, the fitted start *a*, is the number of raw values taken before the first object's first
  place. In the all-infantry games it is found by each game's own fit and depends on the seed
  only, the same in every pairing:

  | Seed | 0 | 1 | 2 | 3 | 4 | 5 |
  |---|---|---|---|---|---|---|
  | *N*₀ (raw values) | 706 | 709 | 711 | 718 | 706 | 701 |

- *N* and *N*₀ count raw values, not draws. Before the first object's first place a ranged draw
  may take more than one raw value (S-0202 claim 4), so the record gives no count of the draws
  taken there. From that place on, the fit reads each position from an R(0, 3) draw, which takes
  exactly one raw value (S-0202 claim 4), and a creation draw takes one (S-0202 claim 10). So if
  an infantry object's places before its last are creation draws (inferred, below), each place
  from the first object's first place on is one draw, in order, in the all-infantry games.
- In the games with vehicles *N*₀ is not found on its own: it is taken by seed from the
  all-infantry fits, and the totals of step 7 are consistent with it in every one of those games.
- The starting objects are created before the set-up's first object, so any raw values they take
  fall within the first *N*₀. Which of them they take, if any, is not identified. That they are
  the players' starting construction vehicles the harness removes (Setup) is a reading: they
  come before every set-up object, are vehicles of no type the set-up names, and are gone at the
  next tick.
- The model's count takes the creation draws of the objects that a set-up object's creation makes
  from that object's type, so the spawned objects enter no count by their class. The all-infantry
  rows and the fit are checked against each game's own recorded count *N* and hold, as stated, in
  every one of the 192 games. The refusal count (*Set-up objects off the map before a unit*) holds
  in all 1860 games, since it counts the set-up's objects only.
- Creation order (Setup) puts the row type's units first, side A's in the side-A games and side
  B's in the side-B games. Step 5 tests the other order in the 96 side-B all-infantry games, where
  the two differ: identifier order fits in all 96, and with side A first no start fits in
  any of them.
- So, in the all-infantry games, the model fits: each infantry object's model count of the
  scenario stream's raw values, consecutive and in creation order (identifier order), the standing
  position given by the R(0, 3) result at the last, and the last object's last place the last raw
  value taken before tick 0, any objects off the map counted as in step 3. Within the counts
  tested, exactly one start fits in each game. The recording does not show which raw value each
  creation or placement took. The per-object places are the fitted model's.
- In the games with vehicles only the count is compared. In all 1860 games the raw values taken
  after the first *N*₀ are as many as the sum of the model's count over the set-up's objects, those
  off the map at tick 0 included. Which object took which raw value is not identified in the games
  with vehicles.
- That an infantry object's raw values before its last are its creation draws (S-0202 claim 10,
  S-0506 claim 1, S-0402 claim 19) and the last its standing-position draw (S-0216 claims 2 and 3)
  is consistent with the counts and the fit. The last is the one the fit reads positions from.
  The others are inferred.

**Set-up objects off the map before a unit.**

- In all 1860 games, for every unit of the set-up, the number of the set-up's objects off the map
  created after the previous unit and before it (for the first unit, before it) equals the place
  *p* of its cell in claim 8's order. No game has an exception. Spawned objects are not counted:
  they are not of a type the set-up names, and counting them would break the equality in the
  games that hold them.
- In all 1860 games each of those objects is of the type and house of the unit created after it.
- No identifier of the set-up's objects off the map at tick 0 is among the objects the recorder
  gives at the next tick, of any house.
- Reading (inferred, not a finding): each cell of claim 8's order that a unit did not stand on,
  before the cell it stands on, left one object of the unit's type off the map at tick 0, gone by
  the next tick, each try a new object. The recording gives the count only, equal to *p*. It does
  not show which cells were tried, nor that each object belongs to one tried cell.
- The model counts any such object as S-0216 claim 8 counts a refused try (Setup), its places in
  creation order before the unit's own, and step 7's equality holds with it so counted in all
  1860 games. In which games such objects occur, which places occur, and how often, is not given:
  read with M-0006's moves it would bound the groups' sizes.
- The verdict is *new-finding*: the off-map objects before each unit, their count in the raw
  values and the creation order with which the fit starts (step 5: with side A's set-up objects
  first, no start fits in any of the 96 side-B all-infantry games) are findings that the specs
  this record tests state only by citing it (S-0216 claims 3, 7 and 8).

**Tolerance for tests.** Exact, per game (n = 1 for each of the 1860 seeded games; there is no
spread, since each game replays from its seed). What a test may require depends on what it runs:

- A test of S-0216 claim 2's position rule may require, with no tolerance, that each placed
  infantry unit of the 192 all-infantry games stands at the position claim 2 gives for the R(0, 3)
  result at the last of that object's raw values in the fitted model. It requires that match of
  position to draw, not the draw's place in the stream: that place follows the records' creation
  order, and with side A's set-up objects first no start fits in the 96 side-B all-infantry games
  (step 5).
- A replay of these set-ups through the records' harness (Setup) may require, with no tolerance:
  in every game *N* − *N*₀ equal to step 7's sum (*N*₀ the seed's value in the table above),
  before each unit the number of the set-up's objects off the map equal to *p*, and in the 192
  all-infantry games the fitted model's places, each object's model count in identifier order.
- A test of S-0216 claims 3 and 8 in claim 8's arena may require, in the 192 all-infantry games,
  the records' creation order (the row type's units first). It cannot require the off-map
  counts: claim 8 keeps no object for a refused try. The sums and the fitted model's places it
  may require only where claim 8's take test refuses the cells the records' harness refused,
  which this record does not compare: S-0216 claim 8 rates that its test refuses the same cells,
  and its rounding, `unmeasured`.
- Of the 192 all-infantry games a test may require no placed infantry unit at the centre or top
  position at placement (by the fit). Of the 384 other games with placed infantry the record gives
  only a tick-0 reading, after that tick's update, of no unit at the centre position. A test that
  reads positions after that update may require it, but it is not a placement position there.

## Evidence

- Recordings: those of the state recorder (Setup), made during the replays of M-0016 and M-0017
  (2026-10-28) and kept local. M-0016 and M-0017 do not use or describe them. The replays were
  not run again for this record: its readings come from those recordings, read on 2026-11-02.
  Per game: per-object state at tick 0 and the next tick, and the scenario stream's state at the
  start of tick 0. SHA-256 of the sorted list of SHA-256 sums of the 1860 games'
  object-state and stream-state files:
  `88ace101541b91c644a848bfdab0871edd0bb255facaa7b692d700676839a6fd`.
- The stream model is the generator of S-0202. Replaying it from each game's seed reaches the
  recorded state at the start of tick 0 in every one of the 1860 games.
- Nothing went wrong in these readings.

## Notes

- Q-2277 q3 (whether moving the side-A YURI units between the centre and another standing
  position changes the winner) is not answered here. It needs a run whose arms put those units at
  named standing positions (S-0208 claim 5) and differ in that position. This record has no such
  run.
