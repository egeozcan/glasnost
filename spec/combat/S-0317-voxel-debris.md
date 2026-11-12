---
id: S-0317
title: Debris pieces with voxel models (creation, flight, bounces, end of life, removal)
topic: combat
revision: 59
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-12
measurements: []
requests: [Q-1702, Q-1834, Q-1835, Q-1836, Q-2997]
supersedes: []
---

# S-0317: Debris pieces with voxel models (creation, flight, bounces, end of life, removal)

## Summary

S-0303 (claim 22, F6) and S-0312 (claim 4, F1) say how many voxel-model debris pieces a detonation
or a destruction throws, and of which types. This spec says what one piece does once it exists. A
piece is an object of play: its creation takes draws from the scenario random stream, it flies
under a fixed gravity, it can damage objects where it bounces and where it ends, and it is removed
after a lifetime counted in piece turns. A piece of a type with the *meteor* flag flies a straight line
instead and can throw further pieces when it ends. A move can meet the ground, a high bridge's deck, a
building or a cliff face (claim 10). Claims 17 to 19 give the rebound on a sloped cell, the turn
off a cliff face and the cells that count as a building's. A piece is never removed for leaving
the map: off the legal map it meets S-0009 claim 44's stand-in cell, which our engine reads as a
blank cell at height 0 (claims 20 and 21). Claims 22 to
28 say how a piece is drawn: its pose and spin, the section drawn, its light, palette, depth,
shadow and visibility. The drawn model turns once in each flying turn, and the sense of its turn
reverses at every contact (claim 22). Claim 10's flat rebound narrows the elasticity to single
precision before it scales (F8). The slope rebound, the cliff turn and the building cells answer
Q-1834, the drawing Q-1835, and a piece off the map Q-1836. The two faults of the cliff turn share
one switch, as claim 18 states. A meteor type whose spawn type leads back to it makes pieces
with no known limit in the original; our engine bounds such a loop by a departure (claim 29),
which answers Q-2997.

## Terms

Shared terms are in the glossary: *scenario random stream*, *raw value*, *ranged draw R(a, b)*
(S-0202 Terms), *object*, *tick*, *tick number*, *height level*, *land type*, *resource cell*,
*departure*, *importer warning*, *random-state recorder*. In addition:

- **Object of play.** An object that is part of the game state: its creation, its changes and its
  removal can change later play. The opposite is a *presentation effect*, which changes nothing in
  play.
- **Piece.** One voxel-model animation in play, made from one *piece type* (a voxel-animation type
  of the rules file). The voxel-model debris of S-0303 claim 22 and S-0312 F1 consists of pieces.
- **Creation point.** The point the producer names when it makes a piece (for warhead debris,
  S-0303 claim 22's *Q*). The *start position* is where the piece's flight begins (claims 3 and 4).
- **Meteor piece.** A piece whose type has the meteor flag. Every other piece is a *falling piece*.
- **Spawn link, spawn loop.** A meteor piece type that names a spawn type and has a positive
  spawn count (claim 12) has a *spawn link* to that spawn type; no other type has one. A *spawn
  loop* is a set of piece types, each with a spawn link to the next and the last with one to the
  first; a type whose spawn link names itself is a spawn loop of one type. A spawn link *lies on*
  a spawn loop when both its types belong to that loop. A type has at most one spawn link, so it
  belongs to at most one spawn loop. Whether a link lies on a loop is a property of the types
  alone.
- **Lifetime counter.** A signed integer count of piece turns that each piece holds, set at
  creation from the type's duration field (claim 2) and counted down in its turns (claim 6).
- **Ground impact.** A piece turn whose move makes a *contact* of claim 10: it reaches the ground,
  crosses a high bridge's deck, or ends just above the ground in a cell that holds a building; a contact
  that a cliff then undoes is still one (claim 10). The move reaches the ground when the height of
  the piece's flight position after the move is below the ground height at that position truncated
  toward zero in each axis. A height exactly at the ground does not reach it; in a cell that holds
  a building such a height is a building contact instead (claim 10, which gives the boundary its
  backing).
- **RZ₂₄, trunc₂₄.** Both denote a real number rounded toward zero to single precision: RZ₂₄
  as S-0205 Terms defines it, trunc₂₄ as S-0204 Terms does. Other subscripts give other
  precisions in the same way. Every store of an arithmetic result in single precision in this
  spec (the formulas and the start positions of F1 and F2 included) is RZ₂₄ under S-0205's
  `quirks.chop_rounding`; with that switch off, each such store rounds to nearest single
  precision (RN₂₄) instead. Conversions of imported values follow S-0205, and the importer warns for a type whose imported
fields make any conversion of this spec out of range (S-0205).
- **Stopping.** The test of claim 9 that ends a piece's flight when it has almost no motion left.
- **Piece turn.** The processing one piece receives once per tick (claim 6).
- **Speed field, spin field.** The type's fields of claim 3, as Formulas F1 names them: *M* (the
  horizontal speed limit), *Z*ₘᵢₙ and *Z*ₘₐₓ (the vertical speed limits), *A*ₘᵢₙ and *A*ₘₐₓ (the
  spin limits), *e* (elasticity).
- **Cell of a position.** The cell coordinates (*c*ₓ, *c*ᵧ) = (trunc(*x*/256), trunc(*y*/256)) of
  a position (*x*, *y*, *z*) in leptons, as S-0306 Terms has it: a coordinate between −255 and 0
  falls in cell 0.
- **Cell read for a position.** The cell that every reading of this spec takes for a position
  (claim 20): the cell of the position when that is a legal cell (glossary, S-0009 claim 1), and
  the off-map cell otherwise.
- **Off the legal map.** A position is off the legal map when its cell read is the off-map cell.
- **Off-map cell.** The one cell read for every position off the legal map: S-0009 claim 44's
  stand-in cell (claim 20).
- **High cell.** In the cliff turn (claim 18, F9), a cell whose height level is at least the level
  of the cell of the old position plus 2.
- **Orientation.** The rotation with which a piece's model is drawn (claim 22, F10).

## Behaviour

Confidence ratings follow one convention. A claim is rated high, before its scenario has run,
where its backing is `measurable` and the scenario named in it would clearly discriminate the
claim from its alternatives. It is rated medium until that scenario runs where it departs from a
committed spec as that spec is now written, or where the scenario discriminates it only in part.
An `unmeasured` part is rated medium at most.

1. **A piece is an object of play.** Each piece takes part in the game state as every other object
   of play does. Its creation and some of its end effects take draws from the scenario random
   stream, so a piece that one implementation makes and another does not shifts every later random
   outcome of the game (S-0202 Terms, draw allocation order). It can damage objects (claims 7 and
   11). No limit on the number of pieces is known: a producer that throws *n* pieces makes
   all *n*, however many are in play, at least up to the 4000 pieces that V9
   tests. Whatever house its producer gives it (S-0312 claim 4 gives the pieces of a
   destroyed object that object's house; a detonation's pieces are taken to have none, which S-0303
   claim 22 does not state and V5 part f tests), every
   hit a piece deals (claims 7 and 11) is dealt by no attacker and on behalf of no house, its
   further pieces have no house (claim 12) and its attached system no owner house (claim 13). So
   the ownership of S-0312 claim 4 changes no outcome in play of a piece (it does choose the
   palette the piece is drawn with, claim 25). A hit on behalf of a house would
   show: a destruction by
   a hit with an attacking house and no attacker is credited to that house (S-0302 claim 14), and
   so counts among that house's kills in the score-screen tally of S-0414 claim 21. That tally is
   the sense of credit that V5 parts f and g observe: part f for the thrower's house, part g for
   the piece's own house. Both parts tell a hit with a house from one without only once V5 part
   f's check of that crediting shows it in the original.

   | Part of the claim | Backing | Confidence |
   |---|---|---|
   | A piece is an object of play whose creation takes draws | `measurable` (V1) | high |
   | Every creation draw is taken for up to 4000 pieces | `measurable` (V9, read with the per-piece count that V1 establishes, claim 3) | medium until V1 and V9 run |
   | Every one of those pieces is made | `unmeasured`: V9's draws cannot tell it from a limit that takes the draws and makes no piece | medium |
   | No piece made is later removed for the number in play; no limit for larger counts | `unmeasured`: V9 does not record removal, and no finite scenario shows every count | medium |
   | The hits have no attacker | `unmeasured`: V5 part f judges house credit, not an attacker object, and judges no retaliation | medium |
   | The hits have no house, so neither the thrower's house nor the piece's own house is credited | `measurable` (V5 part f for the thrower's house, V5 part g for the piece's own house; V8's matching parts for the expiry damage, claim 11), once V5 part f's check of S-0414 claim 21's crediting has run | medium until V5 parts f and g and that check run |
   | A detonation's pieces have no house | `measurable` together with the hits' lack of a house only (V5 part f: a credit to the thrower would show that the pieces hold the thrower's house and that their hits carry it). Once V5 part g shows that a piece's hits do not carry its own house, the house a detonation's pieces hold changes no outcome in play. Also `measurable` on its own through the drawn palette, which shows the house a piece holds (V15 part c, claim 25) | medium until V15 part c runs |

2. **Lifetime at creation.** A piece's lifetime counter starts at its type's duration field. A
   meteor piece then loses *L* of it, 0 to 19 turns (claim 4, F2). For a meteor piece whose
   duration is −2147483648 + *j*, with 0 ≤ *j* ≤ 19, the loss can wrap the counter (S-0205): when
   *L* > *j* it becomes 2147483648 + *j* − *L*, a value from 2147483629 to 2147483647; when
   *L* = *j* it is exactly −2147483648, which claim 6 wraps at the piece's first turn; when
   *L* < *j* it stays negative and the piece ends in its first turn (claim 6). The duration
   −2147483648 itself (*j* = 0) therefore always wraps, at creation or at the first turn. In all
   three cases F2's start lies some 2³¹ turns of flight back along the velocity (ahead of the
   creation point when the counter is negative): each coordinate beyond the integer range is
   converted by S-0205 claim 7, so the start can lie anywhere, on or off the legal map. The
   piece is placed there (claim 21) and then flies and ends as any other piece, off the legal map
   as claim 20 says.
   - Backing: `measurable` (scenario V2 for the duration field; V4 for a meteor piece's loss,
     through its start position, which F2 moves back by the shortened lifetime)
   - Backing for the start of a meteor piece with such a counter: `measurable` (V4 part f for
     *j* = 0, V4 part g for *j* = 10, which gives all three cases of the loss).
   - Confidence: high for the duration field and a meteor piece's loss; medium for a counter
     of −2147483648 + *j* and the start it gives until V4 parts f and g run. The results for the
     other values of *j* follow from the same arithmetic and are medium.

3. **Creation of a falling piece.** In this order, a falling piece's creation takes four raw values
   from the scenario random stream (for its spin rate, its vertical speed, its *y* speed and its
   *x* speed, F1), then three ranged draws R(−65535, 65535) on the same stream that choose the
   direction of its spin axis (F5). Making its attached particle system, when its type names one,
   takes no draw (claim 13). Its position is the creation point until its first turn. Its flight
   starts 10 leptons higher (its *start position*): before its first flight its flight position
   is its start position, and after each turn in which it flies its position is its flight
   point (claim 11). A falling piece that ends in its first turn, with no flight (claim 6), ends
   with its position still the creation point, while the effects that claim 11 places at the flight
   point use its start position. Its first velocity is that of F1, in leptons per tick. S-0202
   claim 10's row for a bouncing type (S-0109 Terms), "a group of three raw values and a
   subsequent group of three ranged draws", covers the bouncing sprite animations of S-0109's
   animation types, not pieces: a piece type is a voxel-animation type, a list of its own, and
   a piece's creation takes the four raw values above. Changes for other specs limits that row to
   sprite animations accordingly, and the reading stays medium until V1 runs. This reading was
   decided at the delegated sign-off, 2026-10-25. The count rests on that amendment. V1 decides it:
   four raw values before the first ranged draw confirm it, and three falsify it.
   - Backing: `measurable` (scenario V1 for the draw count and the order of raw and ranged
     draws, four raw values before the three ranged draws; V3 for the velocity from the logged
     raw values, which tells which raw value gives which speed). `unmeasured` for the first raw
     value's use as the spin rate, as in claim 4: V15 part b's two spin limits are equal, so its
     rate does not depend on the draw.
   - Backing for the one raw value whose signed reading is −2147483648 (F1, under
     `quirks.random_magnitude_overflow`): `unmeasured`, as the setup cannot select the seed that
     makes that one value in 2³² occur (S-0202 claim 6).
   - Confidence: medium for the draw count until V1 runs, since it departs from S-0202 claim
     10's bouncing-type row as that row reads; it does not rise to high before V1 has run. Medium
     also for the four raw values coming before the three ranged draws, until V1 runs, and for
     which raw value gives which quantity: the speeds' until V3 runs, the spin rate's because
     it is `unmeasured`. High that they are taken from the scenario random stream. Medium for
     the velocity's precision (F1). Medium for the case of that one raw value: it rests on F1's
     form, not on a run.

4. **Creation of a meteor piece.** A meteor piece's creation takes, in this order: a raw value for
   its *y* speed and one for its *x* speed (F2), a raw value that shortens its lifetime (F2), a raw
   value for its spin rate and the three ranged draws of F5; making its attached system takes no
   draw (claim 13). That is four raw values and three ranged draws, the same count as claim 3 gives
   a falling piece (S-0202 claim 10's bouncing-type row does not cover pieces: claim 3). Its vertical speed is *Z*ₘᵢₙ stored in single precision (F2). If its *x* speed is less than minus
   its *y* speed, both horizontal speeds change sign, so that their sum is never negative. Its
   start position is the creation point moved back along its velocity by its shortened lifetime
   (F2). It then flies in the direction of the creation point, but its path passes below it: each
   step falls 1.4 leptons further than the velocity used for the move back (claim 5). So a meteor
   piece aimed at the ground normally reaches it, and has a ground impact (claims 7 to 10) before
   its end. With a shortened lifetime of *T* turns (F2) and a negative *Z*ₘᵢₙ, it reaches the creation point's
   height after about *T* · |*Z*ₘᵢₙ| / (|*Z*ₘᵢₙ| + 1.4) turns, so about *T* · 1.4 / (|*Z*ₘᵢₙ| + 1.4)
   turns before its end: the gap grows in proportion to the lifetime (F2, F3; claim 5's shortfall
   changes this only slightly). With *Z*ₘᵢₙ = −10 the impact comes at about 0.877 of the lifetime:
   some 5 turns before the end for a lifetime of 40, some 120 for one of 1000.
   - Backing: `measurable` (scenario V1 for the draw count and the order of raw and ranged
     draws, and V4 part a for the speeds, the sign change and the start position, and for which of the first
     three raw values gives the *y* speed, the *x* speed and the shortened lifetime, which it
     tells apart from the logged raw values)
   - Backing: `measurable` (V15 part b, which logs the draws and compares the drawn turn with F5's
     axis and rate) for which ranged draw gives which component of the spin axis; medium until V15
     part b runs. `unmeasured` for the fourth raw value's use as the spin rate: V15 part b's two
     spin limits are equal, so its rate does not depend on the draw; medium
   - Confidence: medium for the draw count until V1 runs, as in claim 3 (not high before V1 has
     run), and medium for the order of the draws this claim names among themselves until V1 runs
     as in claim 3. High for the sign change and the start position
     (F2). Medium for the
     ground impact before the end, which follows from F2 and F3.

5. **Flight.** In each piece turn that does not end the piece (claim 6), the piece first loses the
   gravity *g* of 1.4 leptons per tick from its vertical speed, then moves by its whole velocity
   (F3). A meteor piece has *g* added back to its vertical speed after the move, and after any
   rebound of that turn (claim 10). Its vertical speed is single precision and the subtraction and
   the addition are each rounded to it (F3), so they need not cancel. Under S-0205's
   `quirks.chop_rounding` both round toward zero, so the vertical speed's magnitude can shrink by about
   its last bit in a turn, for a vertical speed whose two roundings do not cancel, and the piece's
   height then falls slightly short of the straight line: where that holds in every turn, after
   *n* turns the shortfall is about *n*²⁄2 times that last bit, so with a long lifetime its
   whole-lepton heights can differ from those of an exactly kept vertical speed (V4 part e). With the
   switch off the two roundings are to nearest and the shortfall need not build up. Apart from that, a meteor
   piece's velocity changes only at a ground impact: until its first ground impact it flies a straight line
   whose vertical step is *Z*ₘᵢₙ − 1.4 leptons per tick, and after a rebound it again moves in a
   straight line, with no fall. Ground impacts, stops and rebounds (claims 7 to 10) apply to both kinds
   of piece. The spin turns the drawn model only.
   - Backing: `measurable` (scenario V3 for the falling arc, V4 for the meteor line, V4 part e for
     the meteor's shortfall in whole-lepton heights)
   - *g* is fixed for pieces: the game-wide gravity value of the rules (S-0306 Terms) does not
     change it. Backing: `measurable` (V3 part c)
   - Confidence: high for *g*, its independence of the game-wide gravity value and the order
     (gravity, then move, then the meteor's addition). Medium for the meteor's shortfall until V4
     part e runs

6. **Piece turn.** Each piece in play takes at most one turn per tick, in the update order (below).
   In its turn its lifetime counter is reduced by 1 unless it is 0. A counter of 0 or less ends
   the piece either way (below); the reduction of a negative counter shows only in the wrap of
   −2147483648 below. A start sound named by its type follows the piece from turn to turn
   (claim 14); where in the turn it is moved has no observable effect.
   Then:
   - If the counter is now positive: on a tick with an even tick number, a type that names a
      trailer animation first makes it, at the point that F7 gives from the flight position
      before the move. Then the piece flies (claim 5). A ground impact then has claim 7's effects, and a
      stop claim 9's; a move that is both is a stop only (claim 9). The trailer is made before the move, in time as well as in place. It is
      made with an initial loop delay of 1 update, repeat factor 1 and no reverse request, the
      settings S-0109 claim 29 gives a projectile's trailer.
   - Otherwise the piece ends (claim 11) and leaves play in that turn.

   So a piece with lifetime counter *T* ≥ 1 and no ground impact on water and no stop flies in *T* − 1
   turns and ends in its *T*-th turn. The lifetime counts the turns the piece receives, not ticks:
   a removal earlier in the update order can cost the piece a turn (S-0203 claim 10), and then its
   end comes a tick later. A piece's own end is such a removal: under
   `quirks.update_order_removal_skip` of S-0203 (on by default), at least the object that follows
   the piece in the update order misses its turn in that tick, and S-0203 claim 10 gives the count
   when the end removes further objects. With the switch off, no removal costs a turn. A counter of
   0 or less at creation ends the piece in its first turn, with no flight, with one exception: a
   counter of −2147483648 is reduced to 2147483647 (wrapping, S-0205), so that piece flies on as
   one with that lifetime. A ground impact on water (claim 8) or a stop (claim 9) sets the counter to 0,
   so the piece ends in its next turn. Piece turns take place in the update order of S-0203 (claims
   2 and 3). A piece whose placement succeeds enters play during its creation, before any other
   object's next turn (a placement fails only at one point, claim 21); until its first turn a falling
   piece is at the creation point and a meteor piece at its start position. Claim 11 says which of
   the two points a falling piece's end uses when it ends in its first turn. Where a piece's
   attached particle system (claim 13) stands in the update order is not stated here (Open
   questions).
   - Backing: `measurable` (scenario V2 for the lifetime and the counter of −2147483648, V1 part b
     for one of two pieces placed one after the other missing its turn in the tick of the
     detonation; which of the two misses it is `unmeasured`, since V1 part b's two types take the
     same draws and its record does not tell A's from B's, V3 part b for the
     trailer's cadence and point)
   - Backing for the trailer's creation settings: `measurable` (V3 part b's comparison with a
     projectile's trailer of the same animation, whose test animation shows each setting)
   - Backing for F7's factor *k*: `measurable` (V3 part b) to the bracket that F7 states. V3
     part b's heights establish it: about 8/*Z* wide from whole-lepton heights alone, and at most
     about 1/*z* wide from the exact heights that F1 and F3 give for trailers before the first
     ground impact, at the highest logged height. Matching the original's trailer height for
     every piece is not claimed inside the bracket (F7). Our engine's value of 6.967, and the
     value it takes when the bracket excludes 6.967, are design (F7). Confidence: medium.
   - Confidence: high for the count-down of the lifetime counter, the end in the turn whose
     counter is not positive, the lifetime, and the trailer's *x* and *y* and its height of about
     seven times the piece's (F7). The effects that a turn's flight leads to keep their own
     ratings: a ground impact's in claims 7 and 8, a stop's in claim 9, the start sound's in claim
     14. Medium for *k*'s bracket until V3 part b runs; our engine's value is design (F7). Medium for the
     trailer cadence. Medium for the trailer's creation settings, until V3 part b
     runs. Medium for the
     placement in the update order, which S-0203 claim 2 leaves `unmeasured` for voxel animations,
     and for the missed turn after a piece's end, which follows from S-0203 claim 10.

7. **Ground impact on land.** At each ground impact of a piece whose position before the move is not in a water
   cell, unless the same move is a stop (claim 9), the effects come in this order: first the
   bounce animation, then the hits.
   - the type's bounce animation, when it names one, appears at the piece's position before the
     move, which after any flight is the previous turn's flight point (claim 11). It is made, with
     any draws its creation takes from the scenario random stream (S-0109 claim 3's random
     delay), before the first hit. So every draw of its creation comes before every draw that a
     hit takes (S-0302 claim 13's damage effect and building fires, and the draws of a death that
     a hit causes, S-0312).
   - every object listed in the ground layer of the one cell that holds the piece's new position
     receives a hit when its horizontal distance from
     that position, |Δ*x*| + |Δ*y*| in leptons, is at most the type's damage radius. The hits form
     a set, whose order matters only where one hit's result can affect another object (for
     example a destroyed object whose own death damages another object of the cell before that
     object's hit), or where more than one hit takes draws from the scenario random stream (which
     draw each hit receives): there they come in the ground layer's listing order (S-0209 claim 13; for a
     listing that the hits themselves change, see Open questions). Each hit is
     ordinary damage (S-0302) of the type's damage amount with the type's warhead. It has no
     attacker and no attacking house (S-0302 claim 1, after the amendment that Changes for other
     specs lists), it does not ignore defences, and it carries
     no no-escape marker. Its distance is F4's *D*. Objects of other cells are never hit by a
     ground impact, even when they lie within the radius.
   - the hits take the ground layer whatever the height of the impact. A ground impact on a
     high-bridge deck (claim 10, a move that the deck test puts at the deck height) hits the
     objects of the cell's ground layer, those under the deck, and never an object standing on the
     deck, which belongs to the deck layer (S-0209 Terms); an impact under the deck hits the same
     ground layer. This holds for a deck over ground that is not water; over water, claim 8
     applies at the deck height as under it, and the impact hits nothing. A detonation, by contrast, chooses its layer by height (S-0303 claim 4). This
     is a bug of the original (Edge cases, `quirks.voxel_debris_deck_impact_layer`).
   - no object listed in the cell is skipped for having no health left, unlike S-0613 claim 13's
     gas hits: such an object receives its hit under S-0302's rules for its target kind, as S-0319
     claim 7 states for a sprite animation's hits. For the one such
     object known to stay listed in a cell, a building awaiting removal with 0 health (S-0312
     claim 21), a skip and a hit give the same result: the hit is stopped with no side effect by
     S-0302 claim 8's no-health row for a building.
   - a type with no warhead (the absent-key default, Import mapping) still deals these hits: each
     is a hit without a warhead, which S-0302 decides (claim 6 reduces it to 0 for a hit that does
     not ignore defences, claim 20 stops it on a terrain object, and its Edge cases give the
     departure for a prone infantry unit).

   A ground impact deals its hits whatever the damage amount; S-0302 decides what a hit of 0 does.

   | Part of the claim | Backing | Confidence |
   |---|---|---|
   | Hits in the one cell of the new position only | `measurable` (V5 for one cell; V6's land-to-water shore throw for the cell of the new position, not the old) | high |
   | The ground layer for an impact on a high-bridge deck | `measurable` (V7 part b, its run with objects) | medium until it runs |
   | The radius inside the impact cell | `unmeasured`: V5 places no object in the impact cell beyond it | medium |
   | The distance *D* (F4) | `measurable` (V5 part b) | medium, as it uses the lift of S-0409 F2 as S-0613 F7 does |
   | The hit does not ignore defences | `measurable` (V5 part c) | medium until it runs |
   | No no-escape marker | `measurable` (V5 part d) | medium until it runs |
   | No attacker | `unmeasured`: V5 part f judges house credit, not an attacker object (claim 1) | medium |
   | No house | `measurable` (V5 parts f and g, claim 1) | medium until they run |
   | A hit of 0 | `measurable` (V5 part e) where S-0302 gives an amount of 0 a visible result; otherwise `unmeasured`, as a hit with no visible result cannot be told from no hit | medium |
   | The listing order, where one hit's result can affect another object or more than one hit takes draws | `unmeasured`: no scenario here gives an object hit by the impact a death that affects another object of the cell, or two hit objects that both take draws | low (the same reading as S-0613 claim 13's gas hits) |
   | The bounce animation, with its creation draws, before the hits and their draws | `measurable` (V5 part h, random-state recorder) | medium until V5 part h runs |
   | The bounce animation's position | `measurable` (V3, which records where each bounce animation is made) | medium until V3 runs |
   | A type with no warhead | `unmeasured`: no scenario here isolates its hit consequences | medium, resting on S-0302 |
   | No skip for an object with no health left | `unmeasured`: no scenario here holds an object with no health in the impact cell, and for a building awaiting removal a skip and a hit look alike | medium, as S-0319 claim 7 rates the same rule for a sprite animation |

8. **Ground impact on water.** A ground impact whose position before the move is in a cell of the water land
   type, and whose move is not a stop (claim 9), makes no animation and no hit. It sets the lifetime counter to 0 (claim 6). Only the land type of
   that cell is tested, not the height: a high-bridge cell over water has its ground's land type
   (S-0211 claim 9), so a ground impact there whose position before the move is in that cell is a
   ground impact on water whether claim 10's deck test puts it on the deck or under it. An impact
   on the deck over water therefore makes no bounce animation and hits neither the objects on the
   deck nor those under it, and the piece ends in its next turn at the deck height, so claim 11
   gives it the first row (on water at deck height): the land case, not a wake or splash. This is
   a bug of the original (Edge cases, `quirks.voxel_debris_deck_over_water`). The rebound
   of claim 10 still applies to it: its position and velocity change as claim 10 specifies, so the
   end in its next turn (claim 11) takes place at the position claim 10 leaves, and the wake and splash heights are counted from it.
   - Backing: `measurable` (scenario V6; V7 part b's water runs for a high-bridge cell over water)
   - Confidence: high. Medium that the cell tested is the one before the move, not after it, and
     for the rebound at a ground impact on water. Medium for a deck over water until V7 part b's
     water runs run.

9. **Stopping.** After each move a piece is stopped when its motion measure *S* of F6 is
   less than 2.5. A stop sets the lifetime counter to 0 (claim 6). The measure counts the piece's
   height above its cell's ground height (F6) as motion, even while its vertical speed is near zero. It is
   taken once the move and any rebound of that turn (claim 10) are complete, from the height and
   speeds they leave, and before a meteor piece has *g* added back (claim 5). So for a meteor
   piece it reads the vertical speed with *g* subtracted, not the stored value after the
   addition.
   A ground impact whose move also leaves *S* below 2.5 has only the stop's effect: the rebound of
   claim 10 is still made (with claim 10's position and velocity changes),
   but claims 7 and 8 have no effect in that turn: no bounce animation, no hits and no water test.
   The counter is set to 0 either way.
   - Backing: `measurable` (scenario V3; part d for the absent bounce animation in a stopping turn
     whose move reaches the ground)
   - Backing for the place of the test after the rebound and before a meteor piece's addition:
     `unmeasured`. The place shows only in a piece whose measure lies below 2.5 at this place and
     not at another: a meteor piece whose measure with *g* subtracted is below 2.5 while the one
     with *g* added back is not, or a rebounding piece whose measure after the rebound is below
     2.5 while the one before it is not (or the reverse). Such a piece stops on a turn on which
     the other place would let it move on. No scenario here builds one. Confidence: medium, the
     same place for both kinds of piece.
   - Backing for F6's form: `unmeasured`. V3 can check only the turn in which a piece stops,
     which follows from F6's form but does not show its terms one by one: a different measure that
     falls below the threshold in the same turns would pass V3 too. Confidence: medium on a flat
     cell; low on a sloped cell, where F6's ground height is S-0106's surface height (claim 10) but
     no run here checks the stop there.
   - Confidence: medium that a slow piece near the ground ends in the turn after it comes almost
     to rest, before its lifetime runs out, and for a stop taking the place of a ground impact's
     effects, until V3 runs.

10. **Contacts and rebound.** At a contact a piece makes a ground
    impact (claims 7 and 8) and rebounds. After the move, with the old position before the move and the new one after
    it, the tests below are made in this order. The ground height and the deck height (the ground
    height plus 416 leptons, 4 height levels) are taken at the new position truncated toward zero
    in each axis. The ground height is the terrain surface height of S-0106 claim 2 at that
    position, so on a sloped cell it is the slope's height at the piece's place in the cell, not the
    cell's centre height (medium: `measurable` on a sloped cell by V7 part e's positions; flat
    cells are covered by V3).
    - **High-bridge deck.** When the cell of the new position or of the old position is a
      high-bridge cell (a cell that an intact high bridge marks, S-0211 claim 15): a
      move from at or above the deck height to below it puts the height of the flight position at
      the deck height (on the deck); a move from below the deck height to at or above it puts it
      20 leptons below the deck height (under the deck). A low bridge has no deck: a low-bridge
      cell is not a high-bridge cell, so a move there is tested as on any other cell, by the
      ground test below at the cell's ground height, with the land type of its bridge overlay
      (S-0211 claim 9). The cells of a destroyed high bridge lose their mark (S-0211 claim 15)
      and are tested in the same way.
    - **Ground.** Otherwise (no deck is crossed, whether or not the cell is a
      high-bridge cell: Terms, *ground impact*), when the new height is below the ground height, the move reaches the
      ground. The height is set to the ground height when it is less than 100 leptons below it, and
      is otherwise left where it is, below the ground (Edge cases).
    - **Building cell.** Otherwise, when the new height is at least the ground height and less
      than 150 leptons above it, and the cell of the new position holds a building, the move is a
      contact and the height is set to the ground height. Claim 19 says which cells count as holding
      a building: not every building does, and a wall overlay does.
    - **No contact.** Otherwise the move has no contact: no ground impact and no rebound (the stop
      of claim 9 can still end the piece).

    At every contact that the cliff test below does not undo, the velocity rebounds off the surface of the cell of the new position and is
    scaled by the type's elasticity *e*. On a flat cell the vertical speed is reversed and the
    whole velocity is scaled by *e*: *v*′ = *e*(*v*ₓ, *v*ᵧ, −*v*_z). On a sloped cell the velocity
    is reflected in a tilted plane instead (claim 17, F8), of which the flat rule is the case of a
    flat cell; in both, *e* is narrowed to single precision before it scales (F8; `measurable` by V3
    part e, medium until it runs). The **cliff** test is made
    for a contact whose new position is in a cell at least 2 height levels above the cell of the old
    position. Let *u* be the piece's vertical speed at the start of the turn, before gravity, *z*₀
    the single-precision height of its flight position before the move (F3), and *z*₁ the height the
    contact has just set. *z*₀ is the height of the flight position, not of the position (claims 3
    and 11), which is the flight position truncated to whole leptons and, in a falling piece's first
    turn, the creation point: there the flight position stands 10 leptons higher, at the start
    position. After the first turn the two heights differ by less than 1 lepton, unless the height's
    conversion is out of the integer range (S-0205 claim 7). The move is undone when:
    - *u* is below −0.0002 leptons per tick (the piece is descending) and *z*₁ > *z*₀: the contact
      would put the piece higher than it was before the move, or
    - *u* is any other value, 0 included, and *z*₁ > *z*₀ + *u* + 1: the contact would lift the
      piece more than 1 lepton above the height its own vertical speed, without gravity, would
      have carried it to.

    The comparison with −0.0002 is made in double precision, with *u* (stored in single
    precision) widened exactly, and our engine draws the line at the same place. So a piece whose
    *u* is exactly 0 is undone when its contact lifts it more than 1 lepton above its height
    before the move. A meteor piece with a lower vertical limit of 0 has *u* exactly 0 at the
    start of every turn until its first contact, since the gravity taken off and added back
    cancel exactly for a vertical speed of 0 (F3).

    When the move is undone, the flight position and the velocity become those at the start of
    the turn, the velocity is then turned off the cliff face and scaled by *e* as imported, not
    narrowed to single precision as at a rebound (claim 18, F9),
    and the ground impact's position (claim 7) is the position before the move. So a piece that
    the contact would lift onto a higher cell, by more than its own motion carried it, bounces off
    the cliff face instead, while a rising piece that the contact lifts by little enough climbs
    onto the higher cell.
    - Backing: `measurable` (scenario V3 for flat ground; V7 part a for the cliff test as far as its
      throws reach, the deep move and the undoing of a rising piece's contact; V7 parts b to d for
      the high-bridge deck and the low bridge, the building cell and the 100-lepton limit; V7 part f
      for the set-back of a descending piece less than 100 leptons below the higher ground; V7 part
      i for a rising piece that climbs onto the higher cell in its first turn). That a piece whose
      *u* is 0 is undone when its contact lifts it 1.4 leptons or more, the lifts V7 part h makes,
      is `measurable` (V7 part h); for a lift above 1 lepton and below 1.4 it is `unmeasured`, since
      no run here makes one. The place of the line at −0.0002 rather than at 0 is `unmeasured`: only
      a vertical speed from −0.0002 up to 0 would show it, which F1 and F3 make only by chance, and
      no scenario builds one. Confidence: medium. That *z*₀ is the flight position's height is
      `measurable` in a falling piece's first turn (V7 part i): there the two heights differ by 10
      leptons, and part i's first moves meet the cliff in the band where that difference decides the
      result. Its pieces rise, so the rising condition (the second) is the one that applies: from
      the flight position's height it lets their moves stand, and from the position's height it
      would undo them. A test that undid every contact putting a piece higher than before its move,
      whatever its vertical speed, would undo them from either height, so part i also tells the
      rising condition from that rule. After the first turn it is `unmeasured`: the two heights
      differ there by less than 1 lepton unless the height's conversion is out of the integer range
      (S-0205 claim 7), and no scenario here resolves a lift that close to the line (V7 part h's
      lifts exceed 1 from either height). Medium. The boundary of the ground test (a height exactly
      at the ground height is no contact outside a building cell) is `unmeasured`: F3's
      single-precision steps make an exact landing on the ground height rare, and no run here
      arranges one; medium.
    - Confidence: medium for the flat rebound, the high-bridge rules, the low bridge, the 100-lepton limit and the
      150-lepton building band, until V3 and V7 run. The rebound on a sloped cell, the turn off a
      cliff face and the cells that count as holding a building keep the ratings of claims 17 to
      19. Medium for the cliff test's
      two conditions (until V7 parts a, h and i run, and `unmeasured` for the cases above that
      they do not reach), and for the flight position's height as *z*₀ until V7 part i runs.

11. **End of a piece.** When a piece ends (claim 6), the effects depend on two tests made at its
    position: *on water* (its cell has the water land type) and *at deck height* (its height is at least
    416 leptons above its cell's ground height, 4 height levels). The tests and the wake and splash
    animations of the water rows use the piece's position. The expiry animation, the area damage,
    the light flash, the further pieces and the new resource all use its *flight point*: the
    flight position (F3) truncated toward zero in each axis, a point in whole leptons. After each
    flying turn the piece's position is set to that same point (F3), so after any flight the
    position and the flight point coincide. Before any flight the flight point is the start
    position (claims 3 and 4). The two points therefore differ only for a falling piece that ends
    in its first turn with no flight (claim 6): its position is then the creation point, and its
    flight point its start position, 10 leptons higher (claim 3). So such a piece's tests and
    water animations are at the creation point, and its expiry animation and area damage 10
    leptons above it (V2 part b). A meteor piece is placed at its start position (claim 6), so for
    it the two points coincide before its first flight too.

    | Case | Effects, in this order |
    |---|---|
    | Not on water, or on water at deck height | When the type names an expiry animation: that animation at the piece's flight point (above), then a detonation's area damage at that point (S-0303 claims 1 to 20) with the type's damage amount and warhead (also when that amount is 0 or the type has no warhead, which S-0303 claim 2 then decides: no effect), no source object and no house, resources affected. The warhead's drawing-only light flash, when it has one (none with no warhead), is made at the same point, before or after the area damage: the flash changes nothing in play, so either order conforms. The area damage's own effects follow their rules there: the warhead's particle system when it names one (S-0303 claim 20), owned by no house, and an exploding overlay in the centre cell (S-0303 claim 19), whose barrel debris is a piece of this spec. The projectile-only effects of S-0303 claims 21 and 22 are not made: no explosion animation from the warhead's animation list, no warhead debris and no nullify animation, as for the disc laser of S-0303 claim 23, from which a piece's end differs in making the warhead's light flash (above), which the disc laser does not. With no expiry animation, none of these. |
    | On water and not at deck height, falling piece | The game-wide wake animation at the position, and the first entry of the game-wide splash list 10 leptons higher |
    | On water and not at deck height, meteor piece | The last entry of the splash list 5 leptons higher |
    | Meteor piece, not on water, or on water at deck height | After the first row's effects: the further pieces of claim 12 |
    | Resource flag, not on water and not at deck height | After the effects above: new resource (claim 12) |

    Then the piece leaves play, and its attached system is marked (claim 13).

    With an empty splash list the two water rows are a **departure** (no switch), as for S-0312's
    dropped vehicle in water: with an empty splash list the original's result is undefined. Our
    engine makes no splash animation in the two water rows, the falling piece's wake is still made,
    and the importer warns about an empty splash list. This matches S-0303 F5's result for an
    explosion animation's empty list (none when *n*ₛ = 0).
    - Backing for the empty list: `measurable` for our engine (V6's engine run); the departure's
      result is a design rule, not an oracle value.
    - The cases, the order of each row and the expiry-animation condition: `measurable`
      (scenario V6 for water with a falling piece, V8 for the expiry damage, its absence without
      an expiry animation, the warhead's particle system and the missing projectile-only effects). Confidence: high. The order of the
      expiry animation and the area damage in the first row is `unmeasured`: V8 records the
      damage, the flash and the particle system, not which of the two comes first. Confidence:
      medium for that order. The place of a meteor piece's further pieces and of new resource
      after an area damage that takes draws is `unmeasured`: no scenario puts an expiry animation,
      area damage that takes draws and further pieces or a resource flag in one run. Confidence:
      medium for that order. The area damage affecting resources is `unmeasured`: V8 has no
      resource in the damage area. Confidence: medium.
    - The deck threshold of 416 leptons and its being a height test: `measurable` (V8's runs
      without a bridge, at 415 and 416 leptons). Confidence: high for a height test; medium for
      the exact value 416 until V8's two runs end at exactly 415 and 416 leptons.
    - The two points of a falling piece that ends in its first turn: `measurable` (scenario V2
      part b). Confidence: medium until V2 part b runs, which shows its area damage's point only
      through the expiry animation made at the same point.
    - The area damage's having no house: `measurable` (V8's part with no source object and its
      own-house part, as V5 parts f and g, claim 1). Confidence: medium until those parts run.
    - The area damage's having no source object: `unmeasured`, as for claim 1's hits, which those
      parts judge by house credit only. Confidence: medium.
    - The meteor piece's water row: `unmeasured`, as V6 throws only a fixed-path test type, which
      is a falling type. Confidence: medium.
    - A type with no warhead: `unmeasured`, as its area damage has no effect (S-0303 claim 2),
      which no run can tell from an area damage that hits nothing. Confidence: medium.
    - The light flash for a damage amount of 0: `unmeasured`, as V8 uses a positive amount and no
      scenario here ends a piece whose type has an amount of 0 and a warhead with a light flash.
      Confidence: medium.

12. **Meteor spawns and resources.** A meteor piece that ends not on water, or on water at deck height, and whose
    type names a spawn type with a positive spawn count *c*, makes *n* = R(0, *c*) + R(0, *c*)
    further pieces of the spawn type at its flight point (claim 11), with no house: two ranged draws on the
    scenario random stream, the first one first. Each further piece is created as claims 3 and 4
    say, in turn. With no spawn type, or a spawn count of 0 or less, there are no spawn draws.
    The sum *n* is a signed 32-bit integer that wraps: when *c* > 1073741823 the two draws can
    add up to more than 2147483647, and the wrapped sum is then negative, so no further piece is
    made (the two draws are still taken).
    A spawn count of 2147483647 gives each draw the span of S-0202 claim 5, and the importer
    warns for it as that claim requires. A spawn type that leads back to the piece's own type is
    claim 29's case.

    A type with the resource flag, at an end that is not on water and not at deck height, adds new
    resource: a falling piece to its own cell, a meteor piece to each of the 8 neighbouring cells
    in step-direction order 0 to 7 (for an end off the legal map, claim 20 says how they are
    found). Each such cell that is germinable (S-0401 claim 10) becomes a
    new resource cell as S-0401 claim 11 says, with its overlay draw and then its growth-list
    offer, cell by cell in that order. Its stage is 0. S-0401 claim 11's limits apply: no cell
    becomes a resource cell at stage 12 or more, which never bars one here, as the stage is 0; and
    a tiberium never given an image number makes no new cell (S-0112 claim 12). Cells that are not
    germinable are left as they are and take no draw.

    The tiberium of the new resource is the tiberium of the overlay type at position 127 of the
    overlay-type list, counting from 0 (S-0112, Terms): the first tiberium whose image run, flat or
    sloped, holds that position, subject to S-0112 claim 12 (a tiberium without an image number
    earlier in the list). The position is fixed, whatever the list holds: it is the start of
    the image run that S-0112 builds for image number 3, not a value of the user's data. If that
    overlay type is a resource overlay that no tiberium's run holds, it is the first tiberium of
    the list. If it is not a resource overlay, or the list has fewer than 128 entries, the
    original's result is undefined, and this case is a **departure** (no switch): our engine makes
    no new resource cell, takes no draws and the importer warns (design). Where the tiberium so chosen
    has never been given an image number, there is no new cell; our engine takes no draws there
    (design), and the original's draws in that case are not established.
    - Backing: `measurable` (scenario V4 parts c and d for the spawns; V10 for the resources).
      `unmeasured` for which tiberium appears: V10 sets up no list in which the rule picks a
      tiberium other than the first. `unmeasured` for the fall-backs of the tiberium choice:
      an overlay type at position 127 that no run holds, one that is not a resource overlay, a
      shorter list, and a tiberium without an image number.
      `unmeasured` for the water and deck parts of the spawn condition: V4's meteors end on flat
      clear ground. `unmeasured` for the further pieces' hits having no attacker and no house: V4's
      further pieces carry no damage, and V5 and V8 throw a piece directly, not a further piece,
      though a further piece is a piece, and claim 1 holds for every piece. `unmeasured` for
      the wrapped sum at a spawn count above 1073741823: V4 uses counts of 2 and 0.
    - Confidence: high for the spawn count and its draws, medium for the wrapped sum; medium for the water and deck parts of
      the spawn condition; medium for the further pieces' hits having no attacker and no house. Germinability
      follows S-0401 claim 10's confidence. High for the draw order among the further pieces, and among the resource draws; medium for the
      resource draws coming after the further pieces' draws (`unmeasured`: no scenario runs one
      meteor piece that both spawns and adds resource); medium for their place after the area
      damage's draws (`unmeasured`, claim 11). Medium for the stage. Medium for the tiberium of the new
      resource (`unmeasured`); low for its fall-backs (`unmeasured`).

13. **Attached particle system.** A type that names a particle system makes it when the piece is
    created, after the piece's own draws, at the creation point (not the start position), with the
    piece as its owner object and no owner house. Making it takes no draw (S-0616 claim 1); the
    system's draws come in its own later turns. It then follows S-0616 for its kind with the
    piece as owner: for example a smoke system keeps the displacement from the piece that it had at
    creation (S-0616 claim 5). For a falling piece that displacement is zero, since the piece is at
    the creation point when the system is made (claim 3): the system then follows the piece's
    position, which is the creation point until the piece's first flying turn and its flight point
    (claim 11) after each one; the flight began 10 leptons above the creation point. For a meteor
    piece, which is placed at its start position, the displacement is the whole flight, from the
    start position to the creation point (F2). When the piece leaves play the system is marked, as
    for the removal of an owner object (S-0616 claim 4).
    - Backing: `measurable` (scenario V11) for the creation and the owner object. `unmeasured`
      for the mark: V11 records the tick the system's emission stops, which S-0616 claim 9 and the
      system's own lifetime can also end.
      `unmeasured` for no owner house: V11 records only the system's position and the end of its
      emission, which do not show the owner house
    - Confidence: high for the creation and the owner object, except that the system's place at
      the creation point is medium until V11 runs, since it departs from S-0616 claim 1 as now
      written (Changes for other specs); medium for the mark and for no owner house.
      Medium for both displacements,
      which follow from S-0616 claim 5 and the placements of claims 3 and 4.

14. **Sounds.** A type that names a start sound plays it at the creation point when the piece is
    created; the sound then moves with the piece, in each of its turns (claim 6). The piece itself plays no sound when it ends.
    The stop-sound setting has no effect.
    - Backing: `measurable` (scenario V12)
    - Confidence: medium

15. **Spin.** The spin rate is formed by F5 and turns the drawn model about the spin axis in each
    flying turn, in a sense that reverses at each contact (claim 22). It has no effect on play. Its raw value and the three axis draws are taken at every creation
    that completes (claims 3 and 4; claim 16 for a span of 0).
    - Backing: `measurable` (scenario V1 for the draws). The drawn rotation is claim 22's, with
      its own backing (V15 part b).
    - Confidence: high for the draws, apart from their count, which is rated as in claim 3
      (medium until V1 runs). Medium for the spin rate's value (F5).

16. **Zero spans in the creation formulas.** F1, F2 and F5 divide by a span made from the type's
    fields. A span of 0 makes the original stop with a program error (S-0205 claim 3). For fields
    whose span value before truncation lies within the signed 32-bit range, the spans are 0 exactly
    when:
    - The horizontal limit *M* satisfies −0.5 < *M* < 0.5 (both ends excluded), for *S*ₓᵧ.
    - The vertical limits satisfy −2 < *Z*ₘₐₓ − *Z*ₘᵢₙ < 0 (both ends excluded), for *S*_z of a
      falling piece.
    - The spin limits, in radians, satisfy −2 < *A*ₘₐₓ − *A*ₘᵢₙ < 0 (both ends excluded), for *S*ₐ.

    These intervals are exact in exact arithmetic. The spans are formed in floating point (F1, F2
    and F5, under S-0205), so a difference so close to an end of its interval that the floating-point sum
    rounds onto that end gives that end's span: the intervals hold up to that rounding.

    A negative span is not 0 and does not stop the original: the remainder then takes the span's
    size, so that, for example, *M* = −1 gives *S*ₓᵧ = −2 and horizontal speeds of 1 or 2. A span
    of −1 meets S-0205 claim 3's overflowing division only for the one raw value whose magnitude
    stays −2147483648 (F1's preamble). A falling piece's vertical span is not used by a meteor
    piece. A span of 0 is a **departure**: our engine uses S-0205 claim 3's remainder for that
    division (the remainder equals the dividend), and the importer warns for such a type.
    - Backing: `measurable` for the original's stop with a horizontal, vertical or spin span of 0
      (scenario V13). The departure's result is a design rule, not an oracle value.
    - Confidence: high for the horizontal span that stops the original; medium for the vertical
      and spin spans until V13's runs for them are made.

17. **Rebound on a sloped cell.** At every contact of claim 10 that the cliff test does not undo,
    the rebound depends on the slope shape (S-0106) of the cell of the new position. The velocity
    is reflected in a plane fixed by that shape's original tilt transform (S-0106 table C4), and
    then scaled by *e* (F8). The plane's normal is the up axis of that transform: in exact
    arithmetic, *v*″ = *e*(*v* − 2(*v* · *n*)*n*), where *n* is the unit normal of F8. The
    reflection reverses the velocity's component along *n* and keeps the two components in the
    plane.
    - For the flat shape and the saddle shapes, which C4 gives the identity, *n* is vertical, and
      this is claim 10's flat rule.
    - For the other shapes *n* leans toward the downhill side, as the gas deflection of S-0613 claim
      10 does with the same transforms. So with a positive *e* a piece that comes straight down
      rebounds with a horizontal speed toward the downhill side.
    - The tilt of *n* from the vertical is that of C4, not the terrain as drawn. It is about 29.31°
      for the edge and steep shapes and about 21.36° for the corner and valley shapes. S-0106
      claim 4 corrects this mismatch for drawing only, and the rebound keeps C4's values, as the
      gas deflection does (S-0106 claim 4, S-0613 claim 10).

    The surface the piece meets need not be the cell's ground. At a contact on a high bridge's
    deck (claim 10's deck test), the rebound still takes the slope shape of the ground cell under
    the deck. On a deck above sloped ground a piece therefore rebounds as off the slope, not off
    the flat deck. This is a bug of the original (Edge cases,
    `quirks.voxel_debris_deck_rebound_slope`).
    - Backing: `measurable` (V7 part e for each slope shape; its deck run for the deck over a
      slope, where a test map can hold one). `unmeasured` for F8's single-precision stores on a
      sloped cell: another placement of the stores differs from F8 only in the last place, and
      V7 part e does not log its velocities as exact single-precision values, so no scenario
      here tells the placements apart. Medium.
    - Confidence: medium for the reflection in C4's planes on the edge, corner and valley shapes
      (1 to 12) until V7 part e runs. Medium for C4's angle rather than the drawn one on the
      steep shapes (13 to 16),
      which V7 part e cannot tell apart, and medium for the flat rule on the flat and
      saddle shapes, as claim 10 rates the flat rebound, until V7 part e runs. Medium for the
      narrowing of *e* until V3 part e runs, and medium for
      the deck over a slope (low if no test map can hold one).

18. **Turn off a cliff face.** When the cliff test undoes a move (claim 10), the velocity *u* of
    the start of the turn (before gravity) is turned off a face and scaled by *e* (F9).
    - The face depends on the step direction *d* (glossary) from the cell of the old position to
      that of the new.
    - For a straight step (*d* even) the face lies across the step. The horizontal component
      along the step reverses, and the other component and the vertical speed are kept.
    - For a diagonal step two further neighbours of the old cell are examined (F9's table):
      - When exactly one of them is a high cell (Terms), the face is the straight face that F9
        names for it, and one horizontal component reverses.
      - Otherwise the face is diagonal: the two horizontal components exchange places, and for a
        step to the south-east or north-west both also change sign.
    - Apart from the north-west fault below, the turn keeps the sign of the vertical speed, and
      the whole velocity is then scaled by *e* as imported. So with a positive *e* a piece that was
      falling keeps falling; with *e* = 0 its vertical speed becomes 0, and a negative *e*
      reverses it. A meteor piece then has *g* added back (claim 5, F3).

    Two faults of the original affect the diagonal steps. One switch covers both
    (`quirks.voxel_debris_cliff_step_faults`, Edge cases):
    - **North-east and south-west steps.** These examine the two cells on the far side of the old
      cell, behind the move, instead of the two cells beside the corner the piece crosses, and
      they map each of them to the face the other would give. The south-east and north-west steps
      examine the cells beside the corner.
    - **North-west steps.** When only the northern of the two cells is high, the original gives a
      velocity that matches no face (F9): the *x* speed becomes a fixed 2.3771236, the *y*
      speed takes the old *x* speed, and the vertical speed takes the old *y* speed, all then
      scaled by *e*.

    With the switch off, north-east and south-west steps examine the cells beside the corner, as
    the other two diagonal steps do, and the north-west case turns across *y*: the *y* speed
    reverses and the other components are kept.

    A move whose cell changes by more than one in *x* or in *y* needs a horizontal speed of 256
    leptons per tick or more, unless a coordinate's conversion is out of the integer range
    (S-0205 claim 7), which can change the cell by more than one at any speed. For it the
    original's face is undefined, and this case is a **departure** (no switch): our engine takes
    the step direction from the signs of the two changes and turns the velocity as for that step
    (design). The importer warns for a type
    whose horizontal limit *M* is 256 or more. That warning does not cover every case: a rebound
    off a slope or the north-west turn can raise the horizontal speed to 256 or more from a
    smaller *M*.
    - Backing: `measurable` (V7 part f for each straight step, each diagonal case of F9's table,
      the mirrored cells and the north-west turn). `unmeasured` for the multi-cell move, which
      is a departure, and for F9's scaling by *e* as imported, not narrowed first: part f's
      elasticity of 0.5 is exact in single precision, so it scales alike either way, and no
      cliff turn in the scenarios here uses an elasticity that single precision does not hold.
      Medium.
    - Confidence: medium until V7 part f runs. Its throws reach a cliff face only from below,
      and the diagonal cases need a move that crosses a corner, which part f must arrange.

19. **Cells that count as holding a building.** For claim 10's building contact, the cell of the
    new position counts as holding a building in either of these cases:
    - **A building is listed there.** The first building listed in the cell's ground layer
      (S-0209 claim 13's listing order) counts, unless it is an inactive section of a laser fence
      (S-0211 Terms) or a vehicle-like building (S-0214 Terms). When the first building listed is
      one of those two kinds, the cell does not count, even if a building listed after it would,
      and even if the cell also holds a wall overlay.
    - **No building is listed there.** The cell counts when its overlay is one of the three wall
      overlays of S-0306's constants.

    Every other building counts: one in the construction state (glossary), whether in its
    build-up just after it is placed or being sold, one awaiting removal (S-0312 claim 21), one
    of any house or of no owner, a gate whether open or closed, and an active laser-fence
    section. No
    other overlay counts. Only the ground layer is examined, whatever the piece's height: an
    object on a high bridge's deck makes no building contact. S-0613 claim 10 names the same two
    excluded kinds for a gas particle's obstacles; it does not state the first-listed rule.
    - Backing: `measurable` (V7 part g for the kinds of building and overlay it sets up).
      `unmeasured` for a building awaiting removal and one of no owner, which V7 part g does not
      set up (no map can place a building of no owner: S-0009 claim 27 drops a map object whose
      owner matches no house, and no scenario here makes one in play); for an active laser-fence section, since no spec gives a route by which a scenario
      can make one (S-0211), so V7 part g records it only where a run happens to hold one; and
      for the first building listed: no scenario here builds a cell that lists two buildings,
      which stock play rarely makes.
    - Confidence: medium until V7 part g runs; medium for a building awaiting removal, one of no
      owner and an active laser-fence section (unmeasured, as above); low for the first-listed
      rule.

20. **Off the legal map.** A piece is never removed for leaving the map, at once or later. It
    takes its turns wherever it is, on the legal map or off it, and ends only
    as claim 6 says: after its lifetime, or in the turn after a stop or a ground impact on water.
    Every reading of this spec that takes the cell of a position (its ground height, height
    level, slope shape, land type, high-bridge mark, buildings, overlay and ground light) takes
    the cell read for it (Terms). For a position off the legal map that is the off-map cell, S-0009
    claim 44's stand-in. Our engine reads it as a blank cell at every position and at every time:
    flat (slope shape 0), at height level 0 and ground height 0, of the clear land type, with no
    overlay, no object listed in any layer and no high bridge. This is S-0009 claim 44's
    departure, with no switch. In the original the stand-in is one shared cell whose state can
    change: a low bridge whose end row or pieces reach outside the map gives it that overlay at
    map load, and every later read of any off-map cell, in the same game or in a later game of the
    same program run, sees that state (S-0009 claim 44). On a map with no such low bridge, in a
    fresh program run or one with no earlier game whose low bridge reached off its map, the
    original's stand-in reads as the same blank cell, and the rest of this claim describes that
    blank cell. Its unscaled ground light level is
    1000, which claim 25 reads; S-0104 leaves the light of a position off the legal map open, and
    this value is stated here for a piece only. So off the legal map a piece:
    - reaches the ground at height 0, rebounds as on a flat cell there (claims 10 and 17), and
      can stop there (claim 9, with *h* counted from 0),
    - meets no water in the off-map cell, which is of the clear land type. Its ground contacts
      follow claims 7 to 9: claim 8's water test still looks at the position before the move, so
      a move from a legal water cell that ends off the map is a ground impact on water; any other
      ground impact makes its bounce animation and deals its hits (claim 7), which find no object,
      and a stopping turn has only the stop's effect (claim 9),
    - makes no building contact there, since the off-map cell does not count as holding a
      building (claim 19). The off-map cell has no high-bridge mark, so off the legal map claim
      10's deck test applies only through the cell of the old position: a move from a high-bridge
      cell into the off-map cell is tested for the deck as any move from that cell is. Its cliff
      test compares height levels with level 0 (claim 10), in both directions across the edge of
      the legal map,
    - at its end, makes the first row of claim 11 there: its expiry animation, an area damage in
      which cells off the map take part in nothing (S-0303), the light flash, and for a meteor
      piece the further pieces of claim 12 at its flight point off the map,
    - adds new resource as claim 12 says, with each cell tested as usual. A falling piece adds
      it to its own cell, the off-map cell, which is never germinable: it lies in no usable area
      (S-0401 claim 10). So a falling piece adds no resource and takes no resource draw. A meteor
      piece adds it to each of the 8 neighbouring cells of its flight point, found as below, and
      each neighbour that is a legal, germinable cell becomes a resource cell as on the map. The
      end off the map gates nothing. Whether a neighbour of a position off the legal map can be
      germinable depends on whether the map's usable area reaches its outermost legal cells.

    A position outside the usable area but on a legal cell is on the map. Its cell is a real cell
    with its own height, land type, overlay and contents, and every claim applies to it unchanged.

    A neighbouring cell is found in the same way. A step, one of claim 18's cliff steps or one of
    claim 12's eight resource cells in step-direction order, is added to the cell coordinates of
    the old position (for claim 18's cliff steps) or of the end's flight point (for claim 12). When the sum names a legal cell, that
    cell is the neighbour; otherwise the neighbour is the off-map cell. So across the edge of the
    legal map a neighbour one step away is a legal cell or the off-map cell as the sum says, in
    both directions.

    Our engine reads the off-map cell for every position off the legal map, however far away, and
    for every neighbour that names no legal cell (design rule, decided at the delegated sign-off,
    2026-10-25). The original departs from it only off the legal map, and near its edge only on
    maps whose legal cells reach the highest columns: there a position just beyond the lowest or
    highest legal column can read a legal cell near the opposite edge of the map, one row over.
    Far off the map, a position about 2³¹ leptons away in *y* can read a legal cell (V4 part f),
    and a neighbour of a position tens of thousands of cells off the map can be a legal cell. Reading the off-map cell there keeps a piece that has flown far away free of
    terrain it is nowhere near. There is no switch: see Edge cases.
    - Backing: `measurable` (V14: no removal, the contact at height 0, the expiry animation at a
      falling piece's end off the map, no resource from a falling piece's end, a meteor piece's
      end tested on its legal neighbours, and the ordinary behaviour on legal cells outside the
      usable area; V15 part c's off-map run for the ground light of 1000, inconclusive if no view
      can show a position off the legal map, when that light stays `unmeasured`). `unmeasured` for
      the end's area damage and light flash off the map and for a meteor piece's further pieces
      made off it: V14 records neither the damage nor a flash, its *Meteor resource run*'s type
      names no spawn type, and its *Meteor run* does not arrange a piece that ends off the map.
      `unmeasured` for the original's
      off-map cell being the same blank cell at every position and every time on a map with no
      low bridge reaching off it, in a program run with no earlier such map: V14 samples only some, and S-0009 claim 44 leaves changes in
      play open. Our engine's blank cell is S-0009 claim 44's departure, not a measured fact. The reading of
      the off-map cell far from the map is a design rule and is not measured. Of the original's
      far readings it replaces, V4 part f shows the one in *y*, whose raised test map tells the
      legal cell's ground height from the off-map cell's height 0; the column reading and the far
      neighbours are `unmeasured`, since no scenario here reaches them.
    - Confidence: medium until V14 runs, the meteor neighbours' test included. Medium for the
      end's area damage, light flash and further pieces off the map (`unmeasured`). Medium for the
      design rule far from the map: a choice, which no measurement of the original confirms.
      Medium for the original's far reading in *y* until V4 part f runs; low for its column
      reading and far neighbours (`unmeasured`, as above). Medium that nothing in play changes the
      original's off-map cell on such a map (S-0009 claim 44 did not look for changes in play).
      Medium for the off-map ground light of 1000 until V15 part c's off-map run shows it, and
      medium if it stays `unmeasured`, as no view may show a position off the legal map.

21. **Placement at creation.** A piece is placed (claim 6) at its creation point (a falling piece)
    or its start position (a meteor piece, F2). The placement makes no test against the map's
    bounds, so it succeeds on or off the legal map, at every point but one: a piece whose
    placement point is exactly (0, 0, 0) is not placed, and never takes a turn. Such a piece has
    still taken all its creation draws (claims 3 and 4), made its attached particle system (claim 13) and played its start sound (claim 14). It
    never enters the update order: it never takes a turn, so it never moves, never ends, has
    none of claim 11's effects and is never drawn. Its system keeps it as an owner that never
    moves and never leaves play. No switch: our engine does the same (Edge cases).
    - Backing: `measurable` for a start off the legal map (V14's meteor run). `unmeasured` for
      the failure at (0, 0, 0): no producer here can be made to name exactly that point.
    - Confidence: medium for placement anywhere until V14 runs; low for the failed placement.

22. **Pose and spin of the drawn model.** A piece's model is drawn with its orientation (F10)
    alone. Unlike a body, turret or projectile, no view transform of S-0105 claim 1 is applied. Its
    model frame is mapped straight onto the screen frame of S-0105 F5 and F6: model *x* to the
    right, model *y* up the screen after F5's flip, and model *z* toward the viewer.
    - **At creation.** The orientation is the identity, so the model is seen straight along its
      own *z* axis, from the side its *z* axis points to, at a fixed screen orientation that does not depend on
      the throw.
    - **Flying turns.** In each flying turn (claim 6), contact or not, the orientation turns
      about the spin axis of F5 by the spin rate (F10), a right-handed turn in the model's frame
      before F5's flip. The turn builds up smoothly, not in facing steps, and the axis stays
      fixed.
    - **Contacts.** At every contact of claim 10, also one that the cliff test undoes, the sense
      of the turn reverses before that flying turn's rotation is applied: the contact turn itself
      already turns in the reversed sense, and so does every later turn until the next contact
      reverses the sense again. So after *n* flying turns the model is turned about the axis by
      the spin rate times (*n*₊ − *n*₋), where *n*₊ counts the turns flown in the first sense
      and *n*₋ those flown in the reversed sense, each contact turn counted in the sense that
      its own contact gives.
    - **End.** The pose changes only in flying turns, so not in the turn in which a piece ends
      (claim 6).

    No switch: the missing view tilt is presentation only and not plainly a fault, since the
    model tumbles from its first flying turn (Edge cases).
    - Backing: `measurable` (V15 part a for the pose at creation, V15 part b for the turn, its
      smoothness and its reversal at the ground contacts on its flat ground that make a bounce
      animation, the contact turn included). `unmeasured` for the reversal at the other contacts.
      A contact that the cliff test undoes makes a bounce animation (V7 part a), but V7 records
      positions, not frames, and V15 part b has no cliff, so no scenario here captures the turn
      at such a contact. A stop and a ground impact on water make no bounce animation (claims 8
      and 9) and end the piece in its next turn, in which the pose does not change (claim 6), so
      only the contact turn itself shows the sense there, and V15 part b sets up neither. Medium.
    - Confidence: medium for the pose at creation with no view transform until V15 part a runs.
      Medium for the turn's sense and its reversal, the contact turn's own sense included, and
      for F10's rule, until V15 part b runs.

23. **What is drawn of the model.** A piece draws one section of its model (S-0007 claim 30: its
    own model or a shared part), whose index is the type's section field (Import mapping,
    default 0).
    - The section is drawn with claim 22's transform only. No motion-file transform is applied
      (S-0105 claims 13 and 14 do not apply). For an own model, which has no motion file, that
      changes nothing. For a shared part, the part's motion file plays no part in the piece's
      drawing: the section is drawn neither in frame 0 of that file nor with the file played
      over the piece's life.
    - A section index outside the model's sections is a **departure** (no switch): the
      original's result is undefined. Our engine draws nothing for such a piece, and the
      importer warns (design).
    - Backing: `measurable` (V15 part d), except that the section is not drawn in frame 0 of a
      shared part's motion file, which is `unmeasured`: part d's motion file need not move the
      section in frame 0.
    - Confidence: medium until V15 part d runs; medium for frame 0 (`unmeasured`).

24. **Light.** Before the section is drawn, each normal index of the model's normal table gets
    the shade of S-0306 F16's one diffuse term, with *L*′ = *R*ᵀ*L*.
    - *R* is the rotation of the orientation (claim 22, F10), with no view part, and *L* is the
      fixed light of S-0105 claim 17. The light is therefore fixed in the frame the model is
      drawn in, and the model turns under it: a face of the model brightens and darkens as the
      piece spins.
    - There is no specular term.
    - The normal table is the one that the normal mode of the model's first section selects,
      also when the section drawn is another one. This is S-0105 claim 18's rule, under its
      `quirks.voxel_light_first_section_mode`.
    - Normal indices 253 to 255 get shade 16, and indices at or above the table's size but below
      253 keep the shade an earlier draw gave them (S-0105 claim 20).
    - Each pixel is then looked up in the voxel lighting table (S-0105 claim 19) and rasterised
      as S-0105 claims 21 to 23 say, outside the depth-buffered mode of S-0105 claim 24.
    - All of the above holds for a drawn section of normal mode 1 to 4. A drawn section of normal
      mode 0 is drawn as S-0105 claim 35 says: unlit, with its column data read under
      `quirks.voxel_mode0_misread`, and without the shades of this claim. Whether it is lit
      follows the drawn section's own normal mode, not the first section's. A drawn section of
      normal mode 1 to 4 whose first section has normal mode 0 is lit through a normal table with
      no entries: normal indices below 253 keep the shade an earlier draw gave them, and indices
      253 to 255 get shade 16.
    - Every drawn frame shows the piece in its current orientation (claim 22) and with the light
      of that orientation.
    - Backing: `measurable` (V15 part b, from the shades of the turning test model) for the one
      diffuse term and for the absence of a specular term. The normal
      table of the first section, the shades of indices 253 and above and the drawing of a
      section of normal mode 0 rest on S-0105 claims 18, 20 and 35 and their backing: V15 part b
      does not test them here, and no scenario here draws a debris model with a section of normal
      mode 0. `unmeasured` for a drawn section of normal mode 1 to 4 whose first section has
      normal mode 0: S-0105 does not state this case, and the empty table is inferred from S-0105
      claim 18 (the table follows the first section's mode), claim 35 (a section of mode 0 has no
      table) and claim 20 (the shades of indices outside a table); no scenario here draws such a
      model. `unmeasured` for the current orientation in every drawn frame: an image kept from
      an earlier frame differs from a fresh one only where its pose or light has since changed,
      and no scenario here can tell that from a fresh drawing of the same pose.
    - Confidence: medium until V15 part b runs; medium for the first-section table, the indices
      of 253 and above and a section of normal mode 0, which follow S-0105; medium for the empty
      table under a first section of normal mode 0 (`unmeasured` and inferred, as above); medium
      for the current orientation in every drawn frame (`unmeasured`, as above).

25. **Palette, brightness and translucency.** The drawn image is copied to the screen with the
    first of these that applies:
    - a piece that holds a house (S-0312 claim 4): that house's 53-shade house-colour palette
      (S-0103 claim 5), at the brightness *U* of the cell of the piece's position (S-0104 claim
      13's unscaled ground level), with no extra light, no height term and no bridge-deck term,
    - a piece with no house whose type has the resource flag: the start-up mine table (S-0112
      Terms) at that *U*,
    - any other piece with no house: the start-up mine table at brightness 1000.

    The copy goes through the light-alpha values (S-0104 claim 15). It is 50 % translucent when
    the type's translucent flag is set (Import mapping), and opaque otherwise. A translucent
    piece's pixel blends by S-0102 F8's 50 % row: each output channel is ⌊*s*/2⌋ + ⌊*d*/2⌋, where
    *s* is the channel of the piece's selected, lit source pixel and *d* the channel already on
    the screen, each halved and rounded down before the sum, in integers. This is a rule of a
    piece's own, with no fade by progress. Off the legal map, *U* is the off-map cell's 1000, with
    its own backing in claim 20. So a destroyed
    object's pieces show their house's colour and follow the cell's light, while a detonation's
    pieces, which hold no house (claim 1), show the untinted colours of the start-up mine table
    and do not follow the light unless their type has the resource flag.
    - Backing: `measurable` (V15 part c for the palettes and the brightness *U*; V15 part d for
      the translucency). `unmeasured` for the blend's channel arithmetic: part d's pieces spin
      and move, so no frame shows the same pixel opaque and translucent over a known background.
      S-0102 D41 measures the same arithmetic for animations. `unmeasured` for the absence of an
      extra light, of a height term and of a bridge-deck term: part c lights its pieces with no
      extra light, over ground at one height and with no high bridge, so it cannot tell them from
      their alternatives, and no scenario here sets them up.
    - Confidence: medium until V15 parts c and d run; medium for the blend's channel arithmetic
      and for the three absent terms (`unmeasured`, as above).

26. **Draw point and depth.** The model is placed around the ground projection of the
    piece's position, lifted once by the height lift *L*(*z*) of S-0101 claim 3, where *z* is the
    position's world height, as S-0105 F6 places a part around its draw point. It is drawn in the
    air layer (S-0102 claim 3).
    - **Depth test.** It reads depth with the flat profile and the strict test of S-0102 claims
      10 and 12, with a final bias of −2 − *L*(*z*), where *z* is the world height of its
      position (S-0102 claim 13's terms).
    - **No depth write.** It never writes depth.
    - **Consequence.** The bias cancels the height lift (S-0102 claim 11), so the piece is
      compared as if it lay at the ground point below it. However high it flies, terrain,
      overlays and buildings hide it at a pixel where, and only where, the depth they stored there
      wins the strict test against the flat profile's depth at that pixel (S-0102 claims 10 and 12).
    - Backing: `measurable` (V15 part e) for the depth test, its second run for the flat
      profile rather than the upright one. `unmeasured` for the absence of a
      depth write: no drawing after a piece is placed in the scenarios here to show one.
    - Confidence: medium until V15 part e runs; medium for the absence of a depth write

27. **Shadow.** Every piece that is drawn also draws a shadow, before its body.
    - **Shape.** The shadow is the footprint of the drawn section, two pixels for each present
      column as S-0105 claim 30 says. It is positioned by F11: S-0105 F10 with the orientation's
      rotation applied to the corners of the section's bottom face after their height is set to
      0, with no view transform, no shadow transform of a cell's tilt, a scale of 1 and the
      3-pixel shift of S-0105 claim 31. So the footprint turns as the model spins, and it
      narrows as the model's bottom face turns toward the vertical.
    - **Draw point.** The shadow's draw point is the ground projection of (*x*, *y*), lifted once
      by *L*(*f*) (S-0101 claim 3), where *f* is the ground height at the piece's position (0 off
      the legal map). In a high-bridge cell, when the piece's height is at least *f* + 416, it is
      lifted by *L*(*f* + 417) instead, one lepton above the deck.
    - **Depth.** The shadow reads depth with the flat profile and a final bias of
      −4 − *L*(*h*ₛ), where *h*ₛ is *f*, or *f* + 416 in the case over a deck.
    - **No depth write.** It never writes depth.
    - **Treatment.** It is drawn with the ordinary shadow treatment (S-0102 claim 26, S-0105
      claim 31), not through the light-alpha values, and in every drawn frame for the piece's
      current orientation, as its body is (claim 24).
    - Backing: `measurable` (V15 part f) for the shape over flat ground and the draw point.
      `unmeasured` for the absence of a shadow transform of a cell's tilt: part f throws only
      over flat ground and a deck, where a tilt transform changes nothing. `unmeasured` for the
      treatment: part f records no contrast between the ordinary shadow treatment and the
      light-alpha values. `unmeasured` for the depth reading and the absence of a depth write:
      V15 part f compares the footprint's shape and placement, and no scenario here draws
      anything over the shadow afterwards that would show a depth reading or write (as for claim
      26's depth write).
    - Confidence: medium until V15 part f runs; medium for the tilt and the treatment, and for
      the depth reading and the absence of a depth write

28. **Visibility.** A piece within the view is drawn in every drawn frame, with its shadow,
    whatever the shroud over it: the shroud does not hide it. Its body takes the light-alpha
    values that the shroud pieces set (S-0110 claim 5), so under the shroud it darkens as the
    ground does, to black where the shroud is complete. With fog of war in force, neither the
    body nor the shadow is drawn while the piece's position is fogged for the player. Fog of war
    is off in a skirmish (S-0409 claim 19), so there this rule changes nothing.
    - Backing: `measurable` for the shroud (V15 part g). `unmeasured` for fog of war, as the
      original's skirmish cannot turn it on (as for S-0102 claim 10's remembered images).
    - Confidence: medium

29. **Spawn loops.** The original takes a spawn type as named, also when it is the piece's own
    type or leads back to it through other spawn links (Terms, spawn loop): it neither refuses
    nor changes such data when it loads the rules, and it sets no limit on the pieces a loop
    makes (claim 1). A further piece of a loop is a piece like any other. It enters play during
    its maker's end and takes its first turn later in the same turn pass (S-0203 claim 4), unless
    a removal costs it that turn (S-0203 claim 10: under that claim's switch the ending piece's
    own removal costs the object directly after it its turn, and that object can be one of the
    ending piece's further pieces). It ends in that first turn when its lifetime counter at
    creation is at most 1 and not −2147483648 (claims 2 and 6): always for a duration of at most
    1 (apart from the wrapped durations of claim 2), and for a duration *D* from 2 to 20 when its
    loss *L* is at least *D* − 1. Each end of a piece of the loop that is not on water below deck
    height makes *n* = R(0, *c*) + R(0, *c*) further pieces (claim 12), *c* on average for a
    spawn count up to 1073741823; above it the sum can wrap, and a wrapped sum makes none.

    So in the original no limit is known on the number of pieces that one piece of a loop leads
    to. The chain goes on, within one tick while its pieces end in their first turns and from
    tick to tick otherwise, until every piece of it has ended without further pieces (both draws
    0, a wrapped sum, or an end on water below deck height). With a spawn count of 2 or more the count can grow without
    end; where it grows within one tick, that tick does not end and the game stops responding.

    Our engine departs from this, with no switch. A piece made through a spawn link that lies on
    a spawn loop carries a *loop mark*, which is part of its saved state. Its own type belongs to
    that loop, so its own spawn link lies on the loop too: a marked piece makes no further pieces
    at its end and takes neither spawn-count draw. Its other end effects (claim 11) and its new
    resource (claim 12) are those of any piece. A piece made in any other way (by a producer of
    S-0303 or S-0312, or through a spawn link that lies on no spawn loop) carries no mark, and
    its spawns follow claim 12 unchanged. So one unmarked piece of a loop type leads to at most
    2*c* further pieces of the loop, all made at its end, and none of them makes more. Spawn
    links that lie on no loop, and the pieces made through them, are unchanged. The importer
    warns once for each spawn loop.
    - Backing for the acceptance at load and for every end of a piece of the loop making the
      pieces its two draws give: `measurable` (scenario V16 parts a and c). Confidence: medium
      until V16 runs. Backing for the absence of a limit: `unmeasured`, as in claim 1: no finite
      scenario shows every count, V16 included. Confidence: medium.
    - Backing for the end in the first turn and the chain within one tick: `measurable` (V16 part
      b). Confidence: high for the end in the first turn of a piece whose lifetime counter at
      creation is at most 1, medium where that counter comes from claim 2's wrapped durations,
      and medium for a growing chain stopping the game within one tick until V16 part b runs.
    - Backing for the departure: a design rule, not an oracle value; `measurable` for our engine
      (V16's engine runs). Confidence: high for the departure, a design rule.

## Formulas

All raw values below are read as signed 32-bit numbers (S-0202 claim 6). For a raw value *r*,
|*r*| is its magnitude in S-0202's "magnitude first" signed remainder (claim 6): its absolute
value, except that the value −2147483648 keeps its sign while `quirks.random_magnitude_overflow` is
on (the default) and is taken as 2147483648 when it is off. "rem" is the integer
remainder truncated toward zero, which has the sign of the dividend. trunc is truncation toward
zero (S-0205 claim 7). Each floating-point expression is formed in double precision, as S-0205
claim 4 has it, and then stored in single precision, except the spin rate, which is kept in double
precision.

**F1. Falling piece, first velocity.** With the type's horizontal speed limit *M*, its vertical
limits *Z*ₘᵢₙ and *Z*ₘₐₓ, and the raw values *r*₁ … *r*₄ of claim 3 in the order taken:

> *S*ₓᵧ = trunc(2*M*),  *S*_z = trunc(*Z*ₘₐₓ − *Z*ₘᵢₙ + 1)
>
> *v*ₓ = (|*r*₄| rem *S*ₓᵧ) − *M*,  *v*ᵧ = (|*r*₃| rem *S*ₓᵧ) − *M*
>
> *v*_z = (|*r*₂| rem *S*_z) + *Z*ₘᵢₙ

Each remainder is an integer converted exactly to a floating-point number. The field (*M* or
*Z*ₘᵢₙ, a double-precision value of the type) is subtracted or added in double precision, and only
the result is stored in single precision. Units: leptons per tick. The start position is the creation
point plus (0, 0, 10) leptons, kept in single precision. *r*₁ is the spin raw value *u* of F5.

**F2. Meteor piece.** With the raw values *q*₁ … *q*₄ of claim 4 in the order taken:

> *v*ᵧ = (|*q*₁| rem *S*ₓᵧ) − *M*,  *v*ₓ = (|*q*₂| rem *S*ₓᵧ) − *M*,  *v*_z = *Z*ₘᵢₙ
>
> When *v*ₓ < −*v*ᵧ, both horizontal components change sign.
>
> the lifetime counter *T* is reduced by |*q*₃ rem 20|
>
> start = (trunc(*c*ₓ − *T* *v*ₓ), trunc(*c*ᵧ − *T* *v*ᵧ), trunc(*c*_z − *T* *v*_z))

where *T* is the reduced lifetime counter and (*c*ₓ, *c*ᵧ, *c*_z) the creation point in leptons.
The speeds are narrowed as in F1: each horizontal speed is formed in double precision from the
remainder and *M* and stored in single precision, and *v*_z is *Z*ₘᵢₙ stored in single precision.
The start is formed in double precision from the integer *T*, the stored single-precision speeds
and the creation point, and then truncated.
The remainder by 20 is taken before the absolute value, so 0 to 19 turns are removed. *q*₄ is the
spin raw value *u* of F5.

**F3. Flight step.** A piece's flight position is its start position (claims 3 and 4) until its
first flying turn. In a flying turn, let *v* = (*v*ₓ, *v*ᵧ, *v*_z) and *p* be the velocity and
the flight position at the start of the turn, both in single precision. Then

> *v*′ = (*v*ₓ, *v*ᵧ, *v*_z − *g*),  *p*′ = *p* + *v*′

where *p*′ is the flight position after the move. Each result is stored in single precision, a
narrowing (S-0205), so *v*′_z = RZ₂₄(*v*_z − *g*) and each axis of *p*′ is RZ₂₄ of the exact sum. A
falling piece's velocity after the turn is *v*′, before any rebound of claim 10. A meteor piece's
vertical speed after the turn is then RZ₂₄(*w*_z + *g*), where *w*_z is the vertical speed the turn
has left: *v*′_z, or its rebound value when the turn rebounds (claim 10). Its horizontal speeds are
those the turn has left. So a meteor piece's vertical speed is not *v*_z unchanged: it can differ
from it in the last bit, and the difference builds up over the flight. The first turn's *p* is the
start position. Here *g* = 1.4 leptons per tick per tick, the single-precision 1.4 widened to
a double (slightly less than 1.4, by about 2 × 10⁻⁸). The piece's position in play after
the move is *p*′ truncated toward zero in each axis (claim 11). Claims 9 and 10 give the timing of
the contact and stop tests.

**F4. Impact distance.** For an object at (*x*ₒ, *y*ₒ) and the piece's new position (*x*, *y*):

> *D* = *L*(|*x*ₒ − *x*| + |*y*ₒ − *y*|)

where *L* is the simulation lift of S-0409 F2, applied to this horizontal sum as if it were a
height, exactly as in S-0613 F7: its integer pixel result is used numerically as a lepton distance,
without unit conversion. The objects compared are those whose sum, before the lift, is at
most the damage radius.

**F5. Spin.** With the type's spin limits *A*ₘᵢₙ and *A*ₘₐₓ in radians per tick (Import mapping)
and the spin raw value *u*:

> *S*ₐ = trunc(*A*ₘₐₓ − *A*ₘᵢₙ + 1),  spin rate = (|*u*| rem *S*ₐ) + *A*ₘᵢₙ  (floating point; the result only turns the drawn model)

When the upper limit is at least the lower one and less than 1 radian above it, *S*ₐ is 1 and the
spin rate is *A*ₘᵢₙ whatever the draw. The axis is the direction of the vector (*a*₃, *a*₂, *a*₁),
with *a*₁, *a*₂, *a*₃ the three ranged draws R(−65535, 65535) in the order taken, normalised to
unit length. Which draw gives which component, and the precision in which the spin rate is kept,
show only in drawing, but there they are observable. The first decides the axis the model turns
about, which a capture of the drawn model could compare with V1's logged draws. The second decides
the angle reached after many turns. V15 part b reads the drawn orientation only to the precision
of captured frames (claim 22).
When all three draws are 0 the axis has no direction
and cannot be normalised. Our engine then draws the piece without spin; the axis affects drawing
only, so this changes nothing in play (the original's drawing in that case, about one creation in
2 × 10¹⁵, is not established).

**F6. Motion measure** (`unmeasured` as a form; claim 9). After a move and any rebound of the same turn, and
before a meteor piece's addition of *g* (claim 9), with *v* the velocity at that point and *h* the
piece's height above its cell's ground height in leptons (on a flat cell, its height above the
terrain), an integer: *h* = trunc(*z* − *f*), with *z* the
single-precision height of the flight position and *f* the ground height at the flight point
(claim 11), as claim 10 takes it. In a high-bridge cell (claim 10), when *h* is at least 416 the
deck height is counted from instead: *h* is reduced by 416 (medium). A low-bridge cell is not a
high-bridge cell, so there *h* is counted from the ground height. A piece left 100 leptons or more
below the ground (claim 10) has a negative *h*. On a sloped cell, *f* is the ground height at the
point, which is the terrain surface height of S-0106 claim 2 at the truncated flight point, as in
claim 10's contact test (medium, `unmeasured` on a sloped cell).

> *S* = trunc(*ρ*(trunc(*v*ₓ)² + trunc(*v*ᵧ)² + trunc(*h* *g* + *v*_z)²))

The term *h* *g* + *v*_z adds a height times an acceleration (leptons times leptons per tick per
tick) to a speed (leptons per tick): each term is taken as a plain number, mixed units and all,
and *S* is a plain number. The observable consequence: a piece's height above its cell's ground
counts as motion in proportion to *g*, so a piece that comes almost to rest while still some
leptons above the ground is not stopped until its height and speeds together bring *S* below the
threshold, where a measure of speed alone would stop it at once (claim 9). The form is `unmeasured`: scenario
V3 checks only the turn in which a piece stops, not the form (claim 9). *ρ* is the table square root of S-0204 F1 (S-0204 claim 5, under
`quirks.table_root_distance`), not the exact root, taken of the sum of the three squared integers,
which is exact while the sum is below 2⁵³. Larger sums, which need a component of magnitude
of at least about 5.5 × 10⁷, are not covered here (Open questions). The inner truncations and the
outer one are S-0205 claim 7's conversions. The
piece is stopped when *S* < 2.5, so exactly when *S* ≤ 2.

**F7. Trailer point** (claim 6). Let (*x*, *y*, *z*) be the flight position at the start of the
turn, before the move. The trailer animation is made at (trunc *x*, trunc *y*, *h*), with

> *h* = conv( *k* × *z* + 0.5 )

formed in double precision, where conv is S-0205's common conversion (claim 7), truncation toward
zero, and *k* is a fixed factor of about 6.967. So the trailer is made at the piece's *x* and *y*
but about seven times as high. The *x*, the *y* and the roughly seven-fold height are rated high.
A run establishes *k* only to a bracket. Each trailer made at the whole-lepton height *h* from a
flight position whose logged whole-lepton height is *Z* > 0 (so *Z* ≤ *z* < *Z* + 1) gives

> (*h* − 0.5) / (*Z* + 1) < *k* < (*h* + 0.5) / *Z*

For a trailer made before the piece's first ground impact, F1 and F3 give the exact
single-precision height *z* of that flight position from the logged raw values, and the bound
narrows to

> (*h* − 0.5) / *z* ≤ *k* < (*h* + 0.5) / *z*

V3 part b establishes the intersection of these bounds over its trailers. With whole-lepton
heights alone the bracket is about (*k* + 1)/*Z*ₘₐₓ, so about 8/*Z*ₘₐₓ, wide at the highest such
height *Z*ₘₐₓ (about 0.008 at 1000 leptons). With the exact heights it is at most about
1/*z*ₘₐₓ wide (about 0.001 at 1000 leptons), and each further trailer can narrow it. Matching
the original's trailer height for every piece is not claimed inside the bracket. Two values of
*k* inside it give the same height for every logged trailer, but they can give heights 1 lepton
apart for another piece whose *k* × *z* + 0.5 lies close to a whole number. *k* is rated medium
(claim 6). Our engine takes *k* = 6.967 (design): the factor this spec finds, given to three
decimals. If V3 part b's bracket does not contain 6.967, our engine takes instead the value
inside the bracket with the fewest decimals, and of those the one nearest the bracket's
midpoint; of two values equally near it, the smaller. This rule also covers a bracket narrower than 0.001: the value then simply has more
decimals.
This is a bug of the original (Edge cases,
`quirks.voxel_debris_trailer_height`); with the switch off,
*h* = trunc *z*, so the trailer is made at the flight point (claim 11) before the move. Units:
leptons.


**F8. Rebound** (claims 10 and 17). Let *v* = (*v*ₓ, *v*ᵧ, *v*_z) be the velocity that the turn's
move leaves (*v*′ of F3), and *M* the matrix of S-0106 table C4 for the slope shape of the cell
of the new position (of the ground under the deck at a deck contact, claim 17). *M*'s rows are as
C4 lists them; shapes 0 and 17 to 20 give the identity. Write *S* = diag(1, −1, 1),
*Z* = diag(1, 1, −1), and *ε* for *e* stored in single precision. Then

> *w* = *M*ᵀ · *S* · *v*,  *w*′ = *Z* · (*ε* *w*),  *v*″ = *S* · *M* · *w*′

and *v*″ is the velocity after the rebound. Read the formula from right to left.
- **Stores.** Each completed vector component is stored in single precision: the three
  components of *M*ᵀ · *S* · *v*, each product *ε* *w*ᵢ, and the three components of
  *S* · *M* · *w*′. A
  change of sign is exact.
- **Sums.** A matrix-vector product groups its row sums as S-0613 F3 does: (*a*ᵧ + *a*_z) + *a*ₓ
  for *x*, and (*a*ᵧ + *a*ₓ) + *a*_z for *y* and *z*, with no fused operations.
- **Flat cell.** With *M* the identity, *v*″ = (*ε* *v*ₓ, *ε* *v*ᵧ, −*ε* *v*_z), each product
  stored: claim 10's flat rule.
- **Exact arithmetic.** *v*″ = *ε*(*v* − 2(*v* · *n*)*n*) with *n* = *S* *m*₃ = (*m*₁₃, −*m*₂₃,
  *m*₃₃), where *m*ᵢⱼ is *M*'s entry in row *i* and column *j*. This holds to C4's precision,
  whose rows are unit vectors only to single precision. The tilt of *n* from the vertical is
  arccos *m*₃₃.
- **Stored result.** The rebound velocity equals the composition of the three steps above with
  each listed component stored in single precision, which can differ from the exact form in its
  last place.

*ε* is the imported elasticity narrowed once to single precision: for the default it is the
single-precision 0.8, and for an elasticity of 0.5 it is 0.5 exactly. Units: leptons per tick.

**F9. Cliff turn** (claim 18). Let *u* = (*u*ₓ, *u*ᵧ, *u*_z) be the velocity at the start of the
turn, before gravity, (*x*, *y*) the cell of the old position, and *d* the step direction of the
cell of the new position from it (glossary). *High* is as in Terms. The face is:

| *d* | first cell *P* | second cell *Q* | only *P* high | only *Q* high | otherwise |
|---|---|---|---|---|---|
| 0, 4 | — | — | — | — | across *y* |
| 2, 6 | — | — | — | — | across *x* |
| 1 (north-east) | (*x* − 1, *y*) | (*x*, *y* + 1) | across *y* | across *x* | diagonal |
| 3 (south-east) | (*x* + 1, *y*) | (*x*, *y* + 1) | across *x* | across *y* | diagonal |
| 5 (south-west) | (*x* + 1, *y*) | (*x*, *y* − 1) | across *y* | across *x* | diagonal |
| 7 (north-west) | (*x* − 1, *y*) | (*x*, *y* − 1) | across *x* | north-west turn | diagonal |

with these results before scaling:

| Face | Velocity |
|---|---|
| across *x* | (−*u*ₓ, *u*ᵧ, *u*_z) |
| across *y* | (*u*ₓ, −*u*ᵧ, *u*_z) |
| diagonal, *d* = 1 or 5 | (*u*ᵧ, *u*ₓ, *u*_z) |
| diagonal, *d* = 3 or 7 | (−*u*ᵧ, −*u*ₓ, *u*_z) |
| north-west turn | (*c*, *u*ₓ, *u*ᵧ), *c* = 2.3771235942840576 |

- **Scaling.** Each component of the result is multiplied by *e* as imported, in double
  precision and not narrowed first (unlike F8), and stored in single precision.
- **North-west turn.** *c* is the single-precision number 2.3771236.
- **With `quirks.voxel_debris_cliff_step_faults` off.** The rows of *d* = 1 and 5
  examine the cells beside the corner and keep their faces, as the rows of *d* = 3 and 7 do:
  - *d* = 1: (*x* + 1, *y*) only high gives across *x*; (*x*, *y* − 1) only high gives across *y*.
  - *d* = 5: (*x* − 1, *y*) only high gives across *x*; (*x*, *y* + 1) only high gives across *y*.
  - The north-west turn is across *y*.

Units: leptons per tick.

**F10. Orientation** (claims 22, 24 and 27). **The rule.** After *n* flying turns the orientation
is the turn about F5's axis *a* by *ω*(*n*₊ − *n*₋) (claim 22), where *ω* is the spin rate, and the
drawing transform is that turn's matrix. That is all a run can check, to the precision of captured
frames (claim 22's backing).

**F11. Shadow footprint** (claim 27). S-0105 F10 applies with these changes:
- **Corners.** For each of the four corners *K* = (*K*ₓ, *K*ᵧ, *K*_z) of the drawn section's bottom
  face, take *K̄* = (*K*ₓ, *K*ᵧ, 0), and let *G K* be replaced by *R K̄* (*R* the rotation of claim
  24, F10), stored in single precision as S-0105 F10 stores *G K*.
- **Q.** *Q* = ((*R K̄*)ₓ + 2.9999964, −(*R K̄*)ᵧ), with *μ* = 1.
- **Columns.** The present columns are those of the drawn section (claim 23). The quantities
  *o*, Δᵢ, Δⱼ and the marking of two pixels per present column are S-0105 F10's.
- **Copy.** The copy to the screen follows S-0105 F6 around the shadow's draw point (claim 27).

Units: pixels.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| gravity *g* | 1.4 (the single-precision value, widened) | leptons per tick per tick | every piece type |
| start lift of a falling piece | 10 | leptons | claim 3 |
| meteor lifetime cut | 0 to 19 | piece turns | F2 |
| spin-axis draw bounds | −65535 to 65535 | — | three ranged draws, F5 |
| stop threshold | 2.5 | — (a plain number, F6's mixed-unit form) | F6 |
| deck threshold | 416 | leptons | 4 height levels of 104, claim 11 |
| water splash lift | 10 (falling), 5 (meteor) | leptons | claim 11 |
| trailer cadence | every tick with an even tick number | ticks | claim 6 |
| trailer height factor *k* | 6.967 (our engine's value, design) | — | claim 6, F7 (medium; F7's bracket; matching the original inside it is not claimed) |
| degrees to radians | 0.01745277777777778 (3.1415 / 180) | radians per degree | Import mapping |
| building band | 150 | leptons | claims 10 and 19 |
| high cell | 2 | height levels above the old cell | claim 18, F9 |
| cliff speed line | −0.0002 (double precision) | leptons per tick | claim 10: below it the descending condition |
| north-west turn speed factor *c* | 2.3771235942840576 (single precision) | leptons per tick, before scaling by *e* | F9, `quirks.voxel_debris_cliff_step_faults` |
| off-map cell (our engine; the original on a map with no low bridge reaching off it, in a program run with no earlier such map) | level 0, ground height 0, slope shape 0, clear, unscaled ground light 1000 | — | claim 20, S-0009 claim 44 |
| the one point where a piece is not placed | (0, 0, 0) | leptons | claim 21 |
| default section | 0 | section index | claim 23 |
| shade of normal indices 253 to 255 | 16 | shade index | claim 24 (S-0306 F16) |
| brightness of a piece with no house and no resource flag | 1000 | per-mille | claim 25 |
| body depth bias | −2 − *L*(*z*) | depth units | claim 26 |
| shadow depth bias | −4 − *L*(*h*ₛ) | depth units | claim 27 |
| shadow deck lift | 416 (test), 417 (draw point) | leptons above the ground height | claim 27 |
| shadow *x* shift | 2.9999964 | pixels | F11 |

## State and ordering

| State | Event | Result |
|---|---|---|
| Flying | piece turn, counter positive after the count-down | trailer on even ticks (F7, from the position before the move), then gravity, move, ground impact or stop effects |
| Flying | ground impact on land | rebound; bounce animation, then hits in one cell, only when the turn is not a stop (claims 7, 9, 10) |
| Flying | ground impact on water (the cell before the move has the water land type, a deck over water included) | counter set to 0 (claim 8) |
| Flying | stop | counter set to 0 (claim 9); a stop in a move that reaches the ground has no bounce animation, hits or water test |
| Flying | piece turn, counter 0 or less after the count-down | end (claim 11), then leaves play |
| Ended | — | attached system marked (claim 13) |

Within a creation: the piece's own draws (claims 3 and 4); making its system takes none (claim
13). Where the system enters the update order is open (claim 6, Open questions). Within an end:
the area damage (with the light flash before or after it, claim 11), then a meteor's spawn draws
and the creation draws of each spawned piece in turn (none for a piece with claim 29's loop mark
in our engine), then the resource draws.

## Edge cases and quirks

- **Expiry damage needs an expiry animation** (`quirks.voxel_debris_expiry_needs_animation`,
  default on). A piece's end deals area damage and makes the light flash only when its type names
  an expiry animation (also when the damage amount is 0, claim 11). With the switch off, the area
  damage and the flash happen at every end that is not on water, or on water at deck height,
  whenever the type names an expiry animation (as with the switch on, an amount of 0 included) or
  the damage amount is not 0.
- **The impact distance is lifted** (claim 7, F4). The distance passed to S-0302 is the lifted
  Manhattan sum, smaller than the lepton distance. This is the same rule as S-0613 F7. No switch,
  here or in S-0613 (decided at the delegated sign-off, 2026-10-25).
- **A deck impact hits the ground layer** (`quirks.voxel_debris_deck_impact_layer`, default on,
  claims 7 and 10). A piece whose ground impact puts it on a high-bridge deck hits the objects
  under the deck in that cell, not those on the deck. With the switch off, a ground impact that
  the deck test puts at the deck height hits the deck layer of the cell of the new position when
  that cell carries a high bridge, as a detonation on the deck does (S-0303 claim 4), and every
  other ground impact hits the ground layer. S-0607 has the same kind of fault for a dropped unit's
  landing, under a switch of its own (`quirks.lift_deck_impact_reads_ground`). The two switches
  stay separate, and this switch and `quirks.voxel_debris_deck_over_water` have the meanings given
  here (decided at the delegated sign-off, 2026-10-25).
- **A deck impact over water is a water impact** (`quirks.voxel_debris_deck_over_water`, default
  on, claims 8 and 11). A ground impact whose position before the move is in a high-bridge cell
  over water makes no bounce animation and no hit even when the deck test puts it on the deck, and
  the piece ends in its next turn on the deck with the first row of claim 11. With the switch off,
  a ground impact that the deck test puts at the deck height is treated as on land: claim 7 applies
  to it (its bounce animation, and its hits in the layer that
  `quirks.voxel_debris_deck_impact_layer` chooses), and the lifetime counter is not set to 0 by
  it. An impact under the deck over water stays a water impact either way.
- **An impact hits one cell only** (claim 7). Objects of neighbouring cells within the radius are
  not hit. There is no switch: the one-cell limit is the original's observable behaviour
  (V5), not a fault.
- **A deep move stays below the ground** (claim 10). A move that ends 100 leptons or more below the
  ground height is not put back on the ground: the piece rebounds from where it is, below the
  ground, and its next moves can reach the ground again from below. No switch; it changes nothing
  for a piece that falls less than 100 leptons in one turn.
- **The water test uses the position before the move** (claim 8). A piece that moves from land into
  water in a ground impact bounces once more on the water cell's ground and makes its bounce animation
  and hits; one that moves from water onto land ends. No switch.
- **The trailer is made too high** (`quirks.voxel_debris_trailer_height`, default on; claim 6,
  F7). The trailer animation's height is the piece's height multiplied by about 6.967, while its
  *x* and *y* are the piece's. Its drawing and any effect of its own (S-0109) take place there.
  The switch matters to play, not only to drawing, when the trailer animation's type deals damage
  over time (S-0109 claim 17): that damage is centred at the trailer's height, so objects near the
  piece's path can be missed. With the switch off, the trailer is made at the piece's flight point before the move.
- **Spin limits collapse** (F5). When the upper limit is at least the lower one and less
  than 1 radian above it, the spin rate equals its lower limit. This
  affects drawing only; the draw is still taken. No switch.
- **The stop sound is ignored** (claim 14). No switch: it changes no play.
- **Zero spans** are a departure (claim 16).
- **An empty splash list** is a departure (claim 11).
- **A deck rebound takes the ground's slope** (`quirks.voxel_debris_deck_rebound_slope`, default
  on; claim 17). A contact on a high bridge's deck over sloped ground rebounds as off that slope.
  With the switch off, a contact that the deck test puts at the deck height rebounds by the flat
  rule (F8 with the identity). It is a switch of its own beside the two deck switches above
  (decided at the delegated sign-off, 2026-10-25).
- **Faults of the diagonal cliff steps** (`quirks.voxel_debris_cliff_step_faults`, default on,
  claim 18, F9). The two faults share this one switch, as decided at the delegated sign-off,
  2026-10-25. North-east and south-west steps examine the two cells behind the move and swap
  their faces. With only the northern neighbour high, a north-west step gives (*c*, *u*ₓ, *u*ᵧ)
  scaled by *e*, which no face stands for. With the switch off, north-east and south-west steps
  examine the cells beside the corner the piece crosses, as F9's list under the switch says, and
  the north-west case turns across *y*.
- **A move of more than one cell** is a departure (claim 18): our engine turns it as for the step
  direction of the signs of its two cell changes, and the importer warns for a type whose
  horizontal limit is 256 or more.
- **The rebound uses the original tilt transforms** (claim 17). The tilt of the rebound plane is
  C4's, not the drawn terrain's. No switch: it is the same rule as S-0613 claim 10's gas
  deflection, which has none.
- **The off-map cell is always blank** in our engine (claim 20): S-0009 claim 44's departure,
  with no switch. The original's stand-in can take a low bridge's overlay at map load, and every
  later off-map read sees it, in later games of the same program run too.
- **A far cell read as a legal cell** is a departure (claim 20). In the original, some positions
  far off the legal map, and neighbours of positions very far off it, read a legal cell elsewhere
  on the map. Our engine reads the off-map cell for them (decided at the delegated sign-off,
  2026-10-25). No switch: such positions are reached only with extreme data, such as a wrapped
  duration, or beyond the edge of a map of the largest sizes, and a far cell's terrain there is a
  fault with no use in play.
- **No placement at the origin** (claim 21). A piece placed at exactly (0, 0, 0) takes its creation draws,
  makes its system and plays its sound, then never takes a turn. No switch: our engine does the
  same.
- **No view tilt on a piece** (claim 22). A piece's model is drawn without S-0105 claim 1's view
  transform. No switch: this is presentation only, and it is not plainly a fault, since the model
  tumbles from its first flying turn.
- **A section outside the model** is a departure (claim 23): our engine draws nothing for such a
  piece, and the importer warns.
- **A spawn loop** is a departure (claim 29): a piece made through a spawn link that lies on a
  spawn loop makes no further pieces and takes no spawn draws, and the importer warns for each
  loop. No switch: the original's chain has no bound and can stop the game.

## Import mapping

| Source | Concept | Parse form | Default when absent |
|---|---|---|---|
| rules, voxel-animation type section, `Duration` | lifetime counter at creation (claim 2) | integer, ticks | 30 |
| rules, same section, `IsMeteor` | meteor piece (claim 4) | boolean | no |
| rules, same section, `MaxXYVel` | horizontal speed limit *M* (F1, F2) | floating point, leptons per tick | 15.0 |
| rules, same section, `MinZVel`, `MaxZVel` | vertical limits *Z*ₘᵢₙ, *Z*ₘₐₓ (F1, F2) | floating point, leptons per tick | 3.5, 5.0 |
| rules, same section, `Elasticity` | elasticity *e* (claim 10) | floating point | 0.8 (single precision) |
| rules, same section, `MinAngularVelocity`, `MaxAngularVelocity` | spin limits *A*ₘᵢₙ, *A*ₘₐₓ (F5) | floating point in degrees per tick; a nonzero value is multiplied by the degrees-to-radians constant; a value of exactly 0 gives the creation default of the last column | absent: −1 degree each, about −0.01745 radians; an explicit 0 gives the creation defaults, 0 for the lower limit and 10 degrees (about 0.1745 radians) for the upper |
| rules, same section, `Damage`, `DamageRadius`, `Warhead` | impact and expiry damage (claims 7, 11) | integer; integer leptons; warhead name | 0, 0, none |
| rules, same section, `BounceAnim`, `ExpireAnim`, `TrailerAnim` | bounce, expiry and trailer animations (claims 6, 7, 11) | animation names | none |
| rules, same section, `Spawns`, `SpawnCount` | spawn type and count *c* (claim 12); a spawn type that leads back makes a spawn loop (claim 29), for which the importer warns | voxel-animation name; integer | none, 0 |
| rules, same section, `IsTiberium` | resource flag (claim 12) | boolean | no |
| rules, same section, `AttachedSystem` | attached particle system (claim 13) | particle-system name | none |
| rules, same section, `StartSound`, `StopSound` | start sound (claim 14); stop sound read, no effect | sound names | none |
| rules, same section, `VoxelIndex` | section drawn (claim 23) | integer section index | 0 |
| rules, same section, `Translucent` | 50 % translucent drawing (claim 25) | boolean | no |
| rules, the general section, `Wake` | wake animation (claim 11) | as S-0312's Import mapping | as there |
| rules, the combat-damage section, `SplashList` | splash list (claim 11) | as S-0303's Import mapping | as there |

The type's other drawing keys (the model it names) belong to S-0007 and do not change play.

Backing of the absent-key defaults: `measurable` for the duration (V2 run with the key absent), the
speed limits and the elasticity (V3 run with those keys absent: F1's ranges and the rebound factor,
and for the upper vertical limit only as far as F1's vertical span shows it, since any upper limit
from 4.5 up to but not including 5.5 gives the same span with the lower limit 3.5),
and the three animations (V3 run with those keys absent: no bounce, expiry or trailer animation
appears). The damage, radius and warhead defaults are `unmeasured`: with them a ground impact deals no
hit with a visible result, so no run tells them from other values that also give none. The
lower spin default is `measurable` through the drawn rotation only (a V15 part b run with those
keys absent), and the upper one only as far as F5 shows it: any upper limit at least the lower
one and less than 1 radian above it gives the same rotation, the lower limit's, whatever the
draw. The section and translucency defaults are `measurable` (V15 part d).
Confidence: high for the `measurable` defaults, except the exact upper vertical limit (medium,
as V3 shows only the span) and the section and translucency defaults (medium, as claims 23 and
25 rate them). Medium for the damage, radius, warhead and spin defaults, and for the spin
defaults' conversion to radians. The defaults of the meteor flag, the
resource flag, the attached system and the sounds are `measurable` (V3's
run with those keys absent: a falling piece, no new resource, no particle
system and no sound). Medium for them until that run is made. The spawn type and count defaults
are `unmeasured`, low: a falling piece makes no further pieces whatever they are (claim 12).

## Measurement scenarios

All scenarios use stock data with no add-ons or mods, plus named test rules files where a field
must take a chosen value, at a fixed game speed, with the random-state recorder logging every raw
value and ranged draw of the scenario random stream with its tick number. Where a scenario has
"the recorder" log positions, heights or the points at which animations are made, these come
from a read-only recorder of object state, as in other specs' scenarios (for example S-0603's).

A *fixed-path test type* is a named test falling type with a horizontal speed limit of 0.5, equal
vertical speed limits and equal spin limits. By F1 and F5 every span is then 1, so whatever the
draws, both horizontal speeds are −0.5 leptons per tick and the vertical speed is the lower
vertical limit: its path is known in advance. A limit of magnitude at least 0.5 gives no zero span:
0.5 gives a span of 1 and −0.5 a span of −1 (claim 16).

**V1. Creation draws.** *Setup:* a tiny flat map; a test warhead with one debris type and a minimum
of 1 and a maximum of 2 (S-0303 claim 22), so that S-0303 F6's count is R(1, 1), which takes no raw
value (S-0202 claim 4), and a per-type maximum of 1 for the one type, fired once at open ground.
Even with that per-type maximum, F6's allocation draws need not be the same in every run (each
takes one raw value until one assigns the piece), so the recorder's log is read with F6's table:
the piece's draws start after the raw value whose allocation assigns it. Run with a falling type that names no
particle system, a falling type with one, and a meteor type. *Record:* the draws in the
detonation's tick.
*Expected:* after S-0303 F6's draws, four raw values and three ranged draws R(−65535, 65535) per
falling piece (in that order), or for a meteor piece a raw value, a raw value, a raw value, a raw
value and the three ranged draws. The run with a system shows the same creation draws as the run
without one, and making the system adds no creation draw: it takes none (S-0616
claim 1); the system's own draws come only in its own turns (S-0616 claims 5 to 18).
*Falsified by:* a different count, order or stream, or extra draws after the piece's draws in
the run with a system that the run without one lacks before the next object's turn. *Part b (update order, claim 6):* a test warhead with no explosion animation throwing one
piece each of two named test meteor types, A and B, both with a duration of 0 and spawn count 1,
so that both end in their first turn. Their spawn type is a named test falling type with a
duration of 100 that names no expiry, bounce or trailer animation, no particle system and no
resource flag: as a falling type it makes no further pieces (claim 12), its creation takes claim
3's draws and nothing else, and it neither ends nor takes a draw within the observed ticks. In
the record, each pair of spawn draws is followed directly by the creation draws of the pieces it
spawns. The warhead's list holds A and B with
a per-type maximum of 1 each and a minimum of 2 and a maximum of 3 (S-0303 claim 22), so that
S-0303 F6's count is R(2, 2), which takes no raw value (S-0202 claim 4), and F6 makes two pieces,
of either type or one of each; its allocation draws are read with F6's table as in the main part. The test weapon names no firing
animation, its projectile no trailer, the firing unit stays idle after its one shot and nothing
else enters play after the projectile, so the projectile is last in the update order when it
detonates in tick *k*. *Expected:* the two pieces enter play one after the other directly after the
projectile (S-0203 claim 3). The projectile's leaving play in its own turn costs the first of
them its turn in tick *k* (S-0203 claim 10). The second takes its first turn in tick *k* and
ends, so one pair of spawn draws R(0, 1) comes in tick *k*; that piece's own end costs the piece
it spawned, if it spawned one, its turn, not the first piece, which stands before it. With spawn
count 1, a piece spawns no piece when both its draws are 0, one run in four on average (claim
12); the tested draw ticks are the same either way, and a spawned piece adds only its creation
draws, after its pair, in the same tick. The first piece takes its first turn in tick
*k* + 1 and ends, so the other pair of spawn draws comes in tick *k* + 1. The two types take the
same draws, so the record shows the ticks of the two pairs, not which type owns which pair (claim
6). *Falsified by:* both pairs in the same tick, or a pair in neither tick. The engine fixture
with `quirks.update_order_removal_skip` off expects both pairs in tick *k*.

**V2. Lifetime.** *Setup:* as V1 on flat clear land, with named fixed-path test types whose two
vertical speed limits are both 25, which name an expiry animation and have a duration of 1, 5 and
0, and a test warhead with no explosion animation. Nothing else that can leave play is in play
during the counted turns: no other projectile, animation or piece, and the firing unit stays idle
after its one shot. As in V1 part b, the test weapon names no firing animation, its projectile no
trailer, and nothing else enters play after the projectile, so the projectile is last in the
update order when it detonates in tick *k*, and the piece enters play directly after it (S-0203
claim 3). Such a piece rises at 25 leptons per tick less gravity and comes back to the
ground only after more than 30 turns, so no ground impact, ground impact on water or stop (it is high above the
ground) can happen within the counted turns. A further run leaves the duration key absent (the
default of 30), and one more sets the duration to −2147483648. *Record:* the tick *k* of the
detonation, the tick *m* of the piece's first movement (from a position log) and the tick of the
expiry animation, which marks the end. *Expected:* for a duration *T* ≥ 2 the end is in tick *m* +
*T* − 1. For a duration of 1 or 0 the piece never moves and ends in its first turn. With
`quirks.update_order_removal_skip` on, the projectile's own removal in tick *k* costs the piece,
directly after it, its turn in tick *k* (S-0203 claim 10), so the end is in tick *k* + 1; the
engine fixture with the switch off expects tick *k*. With −2147483648 the piece flies on and has not ended after the 30 counted turns.
*Falsified by:* any other end tick. *Part b (the end with no flight, claims 3 and 11):* the type
with a duration of 0, its expiry animation named, fired once at open flat ground and once at an
enemy ship on a water cell, with the recorder logging the detonation's point (the creation point,
S-0303 claim 22) and the position at which each animation is made. *Expected:* on the ground, the
expiry animation 10 leptons above the creation point, at the same *x* and *y*; on the water, the
wake at the creation point and the first splash 10 leptons above it, and no expiry animation.
*Falsified by:* an expiry animation at the creation point's height, or a wake 10 leptons above the
creation point (with a splash 20 above it), which would mean that the end uses a single point.

**V3. Falling arc and rebound on flat ground.** *Setup:* as V1 on flat clear ground with an expiry
animation and a bounce animation. A further run leaves the speed-limit, elasticity,
bounce-animation, expiry-animation and trailer-animation keys absent, and also the meteor flag,
spawn type and count, resource flag, attached-system and sound keys. *Record:* the piece's
position each tick (from the drawn model or the read-only recorder's position log), the tick and
position at which each bounce animation is made, and the logged raw values. *Expected:* the first velocity of F1, *g* = 1.4 per tick,
positions each tick as F3 predicts from the logged raw values (bounce heights fall by about *e*²
between bounces, not exactly, because gravity acts in whole ticks), each bounce animation at the
piece's position before that turn's move (claim 7), and an end in the piece turn after the first
stop of F6, or at the lifetime. When the stopping turn's move reaches the ground, that turn has no
bounce animation (claim 9). *Falsified by:* a first velocity that differs from
F1, a different gravity, a rebound that does not reverse and scale the velocity, a bounce
animation at the position after the move, a bounce animation in a stopping turn, or an end in a
different turn from the one expected (F6's form is `unmeasured` and V3 checks only its stop turn, so an end one
turn away from it weighs on F6, not on the rest). In the run with keys absent, *expected:* the defaults of Import
mapping: no bounce, expiry or trailer animation, a falling piece, no further pieces, no new
resource, no particle system and no sound. *Falsified by:* any of these appearing.
*Part b (the trailer, claim 6 and F7):* the falling type of the setup with a trailer animation
named (a test copy of a stock smoke animation), with the recorder logging the piece's flight
position each tick, and the tick and position at which each trailer animation is made. *Expected:*
a trailer in each flying turn on a tick with an even tick number and in no other turn; each at the
truncated *x* and *y* of the piece's flight position before that turn's move, and at the height
that F7 gives from that position's height, about seven times the piece's height (with whole-lepton
heights, and with the exact heights that F1 and F3 give from the logged raw values for the trailers
before the first ground impact, this establishes F7's bracket for *k*; the recorder also logs the
raw values of the piece's creation draws). *Falsified by:* a
trailer on an odd tick, a trailer at the piece's own height, or a trailer at the position after
the move. Creation settings: the trailer animation is a named test animation of at least
three frames that advances one frame on every update, has a loop count of 2 and is not marked to
play in reverse, so that an initial loop delay shows as a held first frame, a repeat factor
other than 1 as a number of passes other than 2, and a reverse request as reversed play
(S-0109). With the same test animation also named as the trailer of a test projectile, capture
every drawn frame from each trailer's creation. *Expected:* counted in the trailer's own updates
rather than in ticks, the piece's trailer shows the same frames for the same number of updates
from its creation as the projectile's trailer (S-0109 claim 29 gives the projectile's trailer these settings, and claim 6 gives the piece's trailer the same):
the same first advance, two passes, forward play. *Falsified by:* a first frame held longer or shorter, a reversed play or
a different number of passes.
*Part c (gravity, claim 5):* the main run again with a test rules file whose game-wide gravity
value is changed. *Expected:* each tick's position is the one F3 gives from this run's own logged
raw values, with the same *g* as the main run. *Falsified by:* a vertical step that changes with
the gravity value.
*Part d (a stop on the ground, claim 9):* a named test falling type with a duration of 400 and
an elasticity of 10 %, thrown on flat clear ground with a bounce animation named, so that its
pieces come almost to rest on the ground long before their lifetime ends. Repeat throws until at
least 5 pieces have a stopping turn whose move reaches the ground, found from F3's positions and
the logged raw values. *Expected:* none of those turns makes a bounce animation, and each piece
ends in the turn after it. *Falsified by:* a bounce animation in such a turn. A set of throws with
no such turn is inconclusive for this part.
*Part e (the narrowing of the elasticity, claim 10 and F8):* part d's throws, with the read-only recorder
logging the velocity each tick as its exact single-precision values. Single precision does not
hold part d's elasticity, 0.1, exactly. For each flat rebound, compute each component after it from the logged
velocity before it in two ways: scaled by the single-precision 0.1 (F8's *ε*) and stored, and
scaled by the double-precision 0.1 and stored. The two differ in the last place for some
components only. *Expected:* every component matches the first. *Falsified by:* a component that
matches only the second. Throws in which fewer than 10 components tell the two apart are
inconclusive for this part, and more are thrown.

**V4. Meteor line.** *Setup:* a named test meteor type with a duration of 40, equal vertical speed
limits of −10 (so that it descends 11.4 leptons per turn), a spawn type and spawn count 2, thrown
by a test warhead at flat clear ground, with a named bounce animation that marks each ground impact.
*Record:* the start position, the per-tick position, the logged raw values and each ground impact.
*Part a (the line):* *Expected:* the start of F2, speeds with *v*ₓ + *v*ᵧ ≥ 0 and a vertical step
*Z*ₘᵢₙ − 1.4 in whole-lepton heights, apart from claim 5's shortfall (part e), until the first
ground impact. The start and the speeds are computed from the logged raw values with the first
for *v*ᵧ, the second for *v*ₓ and the third for the shortened lifetime (F2). A start and speeds
that match only another assignment of those three raw values falsify the order; a run in which
two assignments give the same start and speeds is inconclusive for it and is repeated.
*Part b (the ground impact):* *Expected:* when the creation point is on the ground, a ground impact
about *T* × 1.4 / 11.4 turns before the end, with *T* the shortened lifetime of 21 to 40 (F2),
so some 3 to 5 turns (claim 4), and a straight path with no
fall after it.
*Part c (the spawns):* *Record:* also the number of further pieces and the two ranged draws
R(0, 2) at the end. *Expected:* R(0, 2) + R(0, 2) further pieces.
*Part d:* repeat with a spawn count of 0. *Expected:* no spawn draws and no further pieces.
*Falsified by (parts a to d):* any differing value.
*Part e (the meteor's shortfall, claim 5):* a named test meteor type with a duration of 1000, a
horizontal limit of 0.5 and vertical limits of −10, thrown at flat clear ground, with the
read-only recorder logging the piece's whole-lepton height each tick. Its start is then about
10000 leptons above the creation point and a few cells from it (F2). After the run, compute from
F3 the whole-lepton heights for the recorded start position under two readings: F3's, and one
whose vertical speed is kept exactly. By the first ground impact, some 880 turns in, the two
differ by about a third of a lepton, so their whole-lepton heights differ at many turns. *Expected:* F3's reading at every
turn. *Falsified by:* the other reading at a turn where they differ. A run in which no turn
differs is inconclusive. The placement cannot fail at that start (claim 21).
*Part f (a wrapped counter, claims 2, 20 and 21):* a named test meteor type with a duration of
−2147483648, a horizontal limit of 15, vertical limits of −10 and an expiry animation, thrown by a
test warhead at flat clear ground near the middle of a small test map, several times. Every
legal cell of the test map is flat clear ground at height level 4, so the ground height of any
legal cell differs from the off-map cell's height 0 (claim 20). *Record:* the logged raw values,
the lifetime counter, position and height of each piece from its creation (the read-only recorder,
which follows objects off the map too), the height of each contact, and any expiry animation.
*Expected:*
- each throw makes its pieces, with their creation draws (claim 4),
- each piece's start position is F2's with the shortened lifetime of claim 2 (the counter wrapped
  or −2147483648), computed from the logged raw values, each coordinate wrapped in its conversion
  (S-0205 claim 7). With the loss *r* = |*q*₃ rem 20| and whole-number speeds, this puts a
  horizontal coordinate whose speed is even *r* times that speed from the creation point's,
  one whose speed is odd about 2³¹ leptons away, and the start height 10*r* leptons below the
  creation point's,
- each piece then takes its turns as claim 6 says (one in each tick, except where a removal
  earlier in the update order costs it one), with its counter counting down from the wrapped
  value; a piece whose counter after the loss is −2147483648 wraps at its first turn (claim 6)
  and flies on,
- each move is tested as claim 10 says against the cell that the original reads for the new
  position (below). A start at or below the ground height there makes the first move a contact, so such a piece
  rebounds (claims 10 and 17) and does not fly its straight line; one that starts above the
  ground flies its line until it meets the ground,
- a piece whose *x* speed is odd reads the off-map cell (contacts tested against height 0). One
  whose *x* speed is even and *y* speed odd lies about 2³¹ leptons off the map in *y*: the
  original reads for it a legal cell, the one in the row after the row its *y* coordinate would
  give without the 2³¹ leptons (the same row when that coordinate is a whole number of cells), so
  claim 10's tests use that cell's ground height, the height of level 4, although the piece lies
  far off the map. The deep-move exception is included: a move that ends 100 leptons or more
  below that height is a contact left below the ground, not set to it, and every other contact is
  set to that height. Contacts tested against height 0 would show the off-map cell instead. Our
  engine reads the off-map cell there (claim 20's design rule), so this case records the
  original's reading that the rule replaces. One whose speeds are both even stays near the
  creation point.

*Falsified by:* a throw that makes no piece, a piece that never moves, a start that differs from
F2's, a piece that ends at once although its counter wrapped to a large positive value, or a
contact that does not follow claim 10 for the cell the original reads for the position (above):
a ground contact set to another height than that cell's ground height (for a piece about 2³¹
leptons off the map in *y*, a contact at height 0 instead of level 4's), or a piece moved by a
contact that claim 10 leaves where it is, such as one 100 leptons or more below that ground.
*Part g (a counter near the wrap, claim 2 for* j *≥ 1):* as part f, with a duration of
−2147483638 (*j* = 10), thrown many times. From each piece's logged third raw value its loss
*L* = |*q*₃ rem 20| is known (F2). *Expected:* a piece with *L* > 10 has the counter 2147483658 −
*L* after the loss and flies on; one with *L* = 10 has −2147483648, wraps at its first turn
(claim 6) and flies on; one with *L* < 10 ends in its first turn with no flight (claim 6). *Falsified by:* a counter after the loss other
than the one its *L* gives, or a piece that flies on although its *L* is less than 10, or ends at
once although its *L* is 10 or more. Throws are repeated until each of the three cases has
occurred; a case that never occurs leaves its result untested.
**V5. Impact hits.** *Setup:* a fixed-path test type with damage 50, a damage radius of 128,
a test warhead, vertical speed limits of −40 each, the default elasticity and duration (Import
mapping) and no expiry animation, so that its end deals no area damage (claim 11), thrown over a cell holding two infantry objects of the same house as the thrower,
and over a cell next to one holding an infantry object within 128 leptons of the impact point.
*Record:* health changes, any experience or attacker credit, and which house is blamed (for
example, whether a house's own objects retaliate). *Expected:* objects in the impact cell within
the Manhattan radius are hit; the neighbouring cell's object is not
hit. *Falsified by:* a hit outside the impact cell or a missing hit inside it.
The vertical limit keeps the first ground impact from being a stop (claim 9): the piece meets the
ground at more than 40 leptons per tick downward and rebounds at more than 30 upward, so after
that turn, with both horizontal speeds truncated to 0 and the height at the ground, F6's measure
is more than 30. The rebound then carries the piece through more turns than its remaining
lifetime, so it ends in the air before a second ground impact.
*Part b (D):* as the setup, with a test warhead whose spread makes S-0302's falloff (S-0302 F3)
depend on the distance, and one infantry object per run at a recorded horizontal displacement from
the impact point, at several displacements with sums up to 128 leptons. *Expected:* each health
loss is S-0302's for the distance *D* of F4. *Falsified by:* a loss that matches the unlifted sum
or the straight-line distance instead, where these give a different loss.
*Part c (defences):* as the setup, with a test warhead whose versus value against the hit
objects' armour is 50 %. *Expected:* each health loss is S-0302's defended amount for that versus
value at the object's distance *D* (F4; S-0302 claims 5 and 6). *Falsified by:* a loss of the full damage amount, which a hit that
ignores defences would give.
*Part d (no-escape marker):* a test rules file that sets the game-wide crew escape chance to
always succeed, and a crewed test vehicle with no passenger capacity and health below the damage
amount in the impact cell, with room around it for the crew to be placed. *Expected:* the
vehicle is destroyed and leaves a crew (S-0312 claim 13). *Falsified by:* no crew, which the no-escape marker would give.
*Part e (a hit of 0):* as the setup with a damage amount of 0, the random-state recorder running.
*Expected:* each object within the radius in the impact cell shows the result S-0302 gives a hit
of amount 0 from that warhead (claims 5 to 8), and objects outside it show none. The part backs
claim 7's hit of 0 only where that result is visible.
*Part f (no attacker):* as the setup, with short game off, but the impact cell holds one
infantry object of a computer-controlled house hostile to the thrower with health below the
damage amount, and one of that house with more health than the damage amount, set to the *guard*
behaviour. The pieces come from a shot of a unit of the thrower's house that stands, like every
other object of that house, beyond the survivor's sight and weapon range, so no live object of
the thrower's house is near the survivor. The creating shot strikes far enough from the impact
cell that its own hits reach neither infantry object, and no other piece it makes lands within
the damage radius of either, so only this piece's impact can hit them. The computer house is given no orders and builds
nothing during the run (a map with no base for it); short game off keeps it undefeated, and the
survivor keeps it in play. *Record:* each house's count of units destroyed on the score screen,
the kill tally of S-0414 claim 21. A hit with the thrower's house as its attacking house and no
attacker would raise the thrower's count (S-0302 claim 14), so the part tells an absent house from
the thrower's, as S-0414 claim 21 credits such a hit to its attacking house. A separate run
first checks that crediting, on the same map with the same two houses:
an infantry object of the computer house is destroyed by a hit with the thrower's house as attacking house and no attacker, for
example the gas particle damage of a particle system owned by the thrower's house (S-0302 claim
1, S-0613 claim 13). *Expected of the check:* the thrower's count rises by one. If it does not,
parts f and g cannot tell a hit with a house from one without, and their house results are
inconclusive. *Expected:* the first dies and the thrower's house's count of units destroyed does
not change, while the victim's house counts one loss (S-0414 claim 21); no object gains
experience. With every object of the thrower's house out of the survivor's sight and range, the
part cannot show retaliation and judges none. *Falsified by:* a credit to the thrower's house or
experience for any object. The part also tests claim 1's reading that a detonation's pieces
hold no house, together with the hits' lack of one: a credit to the thrower would show that
the pieces hold the thrower's house and that their hits carry it. An unchanged count does not show
which of the two holds, so the part does not show whether a piece's hits carry its own house.
Part g does.
*Part g (the piece's own house):* as part f, but the pieces come from the destruction of a test
vehicle of a third, computer-controlled house with no base, standing in the impact cell's
neighbour, whose type has a positive maximum debris count and throws a falling test type with
V5's damage, damage radius and test warhead and a horizontal limit large enough to carry some
pieces into the impact cell (S-0312 claim 4, which makes the pieces owned by the third house). The
vehicle's own death deals no damage, and a unit of the thrower's house destroys it with a weapon
whose detonation throws no debris, from beyond the survivor's sight and range. The thrown type
names no expiry animation, so that no piece's end deals area damage (claim 11). Runs in which no
piece makes a ground impact that is not a stop (claim 9) within the damage radius of the first
infantry object are repeated; the recorder's positions show the impact, and the piece's further
flight after it, which a stop's zeroed lifetime counter ends (claims 6 and 9), shows that the
turn was not a stop. *Expected:* the
first infantry object dies; the third house's count of units destroyed does not change, and the
thrower's house's count rises by one only, for the vehicle (S-0302 claim 14). *Falsified by:* a
rise in the third house's count, or a rise of more than one in the thrower's. As in part f, the
part tells the two apart only once part f's check of the S-0414 claim 21 crediting has shown a
rise.
*Part h (the order of a ground impact's effects, claim 7):* as the setup, with the random-state
recorder running, one piece thrown, and the type naming a test bounce animation whose random-rate
pair is descending, so that its creation takes one ranged draw from the scenario random stream
(S-0109 claim 3). The impact cell of the piece's first ground impact holds, within the radius, one
test vehicle and nothing else. The vehicle has no passengers, and its type's list of damage
particle systems holds two smoke entries. Its health is such that the hit takes its health
from at least half its strength (rounded down) to below it without destroying it, so that the
hit reports *now yellow* or *now red* (S-0302 claim 12). The named test rules file sets the
game-wide yellow ratio to one half, so that the health after the hit is also at or below that
ratio and the hit takes one ranged draw over 0 … 1 (S-0302 claim 13). *Record:* the stream's state at the start of
the impact turn, the bounce animation's stage delay (how long its first frame is held) and which
of the two smoke systems the vehicle gets. *Expected:* replaying the recorded stream with the
animation's draw first and the vehicle's draw after it gives both the recorded delay and the
recorded smoke system. *Falsified by:* a recorded pair that only the reverse order gives. A run
in which both orders give the same pair is inconclusive and is repeated.


**V6. Water.** *Setup:* a
fixed-path test type with V5's damage, damage radius, test warhead, vertical limits, elasticity
and duration (so that, as in V5, no ground impact it judges is a stop) and V4's named bounce animation, thrown over a water cell
holding a test vessel within that radius of the impact point, and from land to water.
*Crossing the shore:* the fixed-path type moves −0.5 leptons per tick in each axis (F1), so a flat
land cell and a flat water cell at the same height level (so that claim 10's cliff test does not
apply) are placed side by side, the water cell on the side of decreasing *x*, and the throw is
made in two steps. A first throw over the land cell alone records the turn *t* of the
first ground impact and the piece's flight position before and after that turn's move. The
creation point is then moved in *x* by the amount that puts the *x* before that move less than
0.5 leptons inside the land cell's edge, so that the move of turn *t*, and no earlier move,
crosses into the water cell, with *y* well inside the shared row. As the creation point is in
whole leptons and each move covers half a lepton, this needs a whole-lepton *x* before move *t*:
when that *x* ends in half a lepton, the throw height is changed until *t* moves by one turn, and
the shift is made after that. The test vessel stands in the water cell within the damage radius
of the landing point, and an infantry object of the same house stands in the land cell for the
mirrored throw. The recorder's positions confirm
that the crossing falls in the move of turn *t*; a throw in which it does not is reported, not
judged. A mirrored throw starts over the water cell, with the land cell on the side of
decreasing *x*, so that the move of the first ground impact crosses from water onto land.
*Record:*
bounce animations, the wake and splash animations, the end tick, and the vessel's and the infantry object's health per tick.
*Expected:* no bounce animation in a ground impact on water; the end in the next turn with the wake
at the piece's ground position after the rebound and the first splash entry 10 leptons above it
(claims 8 and 11).
The test rules file gives the game-wide splash list three distinct entries, so that the first,
the last and a random pick can be told apart, and the throw over water is made five times: a
random pick would show the first entry in all five with a chance of about 1 in 240.
In the shore throws, *expected:* from land to water, the ground impact of turn *t* follows the land
rules, with its bounce animation at the position before the move, on the land side, and a hit
on the vessel in the water cell, the cell of the new position (claim 7), and the piece flies on.
From water onto land: no bounce animation, no hit on the infantry object and the end in the next
turn (claim 8, Edge cases).
*Falsified by:* any other entry in any of the five throws, a bounce animation at a ground impact whose position before the move is in a water cell
(claim 8), the water rules (no bounce animation and no hit on the vessel) in the land-to-water
throw or the land rules in the water-to-land throw, a different end tick, or any health loss of
the vessel in the throw over water alone. *Engine run:* V6 on our engine with an empty splash list. *Expected:* the wake of a falling
piece and no splash animation, and nothing at all for a meteor piece (claim 11). *Falsified by:*
any splash animation or failure.

**V7. Cliff, bridge and building** (claim 10; parts e to g: claims 17 to 19). Named falling test types with chosen horizontal
speeds (a horizontal limit of 0.5 gives −0.5 per axis, F1), an elasticity of 0.5 and a bounce
animation that marks each ground impact. *Record:* the per-tick flight position (the read-only
recorder) and velocity, and each bounce animation.
*Part a (cliff):* a test type with a horizontal limit of 10 and an elasticity of 0.5, thrown
many times from the lower ground at the foot of a cliff 2 height levels high, so that some
pieces fly toward the face. For each move that would end in the higher cell, compute from the
recorded height before the move and claim 10 which case applies. *Expected:* a falling piece
whose height before the move is below the higher cell's ground height and whose new height is
less than 100 leptons below that ground height is set back to
the position before the move, a bounce animation is made, and its next turns move away from the
face more slowly; one that arrives 100 leptons or more below that ground is left in the higher
cell, below its ground (the deep move of claim 10), unless claim 10's cliff test, computed from
the record, undoes the move; one whose height before the move is at or
above that ground height lands on it (claim 10's descending condition does not hold). The throws include pieces that reach the face
while still rising, from the recorded vertical speed at the start of the turn. A rising piece
whose contact would set it more than 1 lepton above its height before the move plus that speed
is set back in the same way; one whose contact sets it no higher than that stays on the higher
cell's ground. *Falsified by:* a descending piece on the higher cell's ground after such a move,
a deep arrival that is set back although claim 10's cliff test, computed from the record, does
not undo it, or a rising piece whose result is the other one of the two. With the default
vertical limits (F1) and *g*, a piece thrown from the lower ground rises less than 20 leptons
above its creation point, so in this part no move ends less than 100 leptons below a ground 2
height levels higher: the set-back of a descending piece within 100 leptons of that ground, and
a rising piece that climbs onto it, are recorded if they occur but are not expected to arise in
this part. Part f arranges the first with its two types, and part i the second.
*Part b (bridge deck):* thrown from above a high bridge's deck so that it falls through the deck
height. *Expected:* the height becomes the deck height, a ground impact, and a rebound from there.
A second run starts below the deck (thrown from under the bridge) with a high vertical speed.
*Expected:* the height becomes 20 leptons below the deck height, a ground impact. A third run
throws the piece over a low bridge on flat ground. *Expected:* no contact at the height of a deck
above it, and the first ground impact at the cell's ground height, with a bounce animation (the
low-bridge cell's land type is not water). The test low bridge's overlay types have the
own-land-type flag and the land type *road*, so that S-0211 claim 9 gives its cells that land
type. *Falsified by:*
a fall through the deck to the ground, a rise through it, or a contact above the low bridge.
*Water runs (claim 8):* the first two runs repeated over a high bridge across water, with a test
vehicle on the deck and a test vessel in the water under it in the impact cell, and the type
given V5's damage and test warhead. *Expected:* falling onto the deck: the height becomes the
deck height, no bounce animation, no health change on either object, and the piece ends in its
next turn with no further ground impact, no wake and no splash. From under the deck: the height
becomes 20 leptons below the deck height, no bounce animation, no health change, and a wake and
splash at its end. *Falsified by:* a bounce animation or a health change at either impact, a
second ground impact on the deck, or a wake or splash after the impact on the deck.
*Run with objects (claim 7):* as the first run, with the type given V5's damage, test warhead and
vertical limits (so that, as in V5, the impact on the deck is not a stop) and a damage radius of
512, so that every object listed in the impact cell is within it (F4's sum
is at most 510 inside one cell). The piece is thrown, as in part d, by a test warhead's detonation
on a test aircraft flying over the deck cell above the deck, so that the detonation looks at the
deck layer only (S-0303 claim 4) and cannot hit the vehicle under the deck; the throwing warhead's
versus value against the deck vehicle's armour is 0 %, so that the detonation does not change
its health either (S-0302 claim 6). A first throw without objects records the cell of the first ground impact on the
deck; one test vehicle then stands on the deck in that cell and one on the ground under the deck
in the same cell, and the throw is repeated. *Record:* each vehicle's health per tick and the
tick of the ground impact. *Expected:* in the tick of the impact on the deck, the vehicle under
the deck loses health and the one on the deck does not. *Falsified by:* a hit on the vehicle on
the deck, or no hit on the one under it. A throw in which the first impact falls in another cell
than the recorded one is repeated.
*Part c (building cell):* thrown so that it falls onto a cell of a test building's foundation.
*Expected:* a ground impact when the height is less than 150 leptons above the ground height,
the height set to the ground height. *Falsified by:* a fall to the ground with no earlier impact,
or an impact at 150 leptons or more.
*Part d (deep move):* a fixed-path type with vertical limits of −150, thrown by a test warhead's
detonation on a test aircraft flying over flat ground (S-0303 claim 22), repeated at several heights until some move
ends more than 100 leptons below the ground (about one move in three does, as each move falls
about 150 leptons). *Expected:* the height after such a move is left below the ground height,
while a move that ends less than 100 leptons below it is set to the ground height. *Falsified
by:* the other result in either case.
*Part e (slopes, claim 17):* a test map holding one cell of each slope shape 1 to 20 (S-0106),
each with neighbours at height levels that keep claim 10's cliff test from applying. The pieces are
of a fixed-path test type with vertical limits of −40 each and an elasticity of 0.5. They are
thrown, as in part d, by a test warhead's detonation on a test aircraft flying over each sloped
cell, so that the first contact is on that cell. *Record:* the velocity in the turn before the
contact and in the contact turn, and the position after the contact.
*Expected:*
- The velocity after the contact is F8's for the cell's shape, with C4's matrix, computed from the
  velocity that the move leaves (F3, from the logged velocity before it). The run holds it to F8
  only to within the last place, so it does not test where F8 stores (claim 17).
- On shapes 17 to 20 that is the flat rule.
- A piece that comes almost straight down leaves with a horizontal speed toward the downhill side.

*Falsified by:* the flat rule on shapes 1 to 16, or a reflection in the drawn terrain's planes
(S-0106 claim 4's corrected angles) where the two differ. For a piece that comes almost straight
down at a vertical speed *V*, with *e* = 0.5, the horizontal speed after the rebound is about
0.5 *V* sin 2*θ* for a tilt *θ*. C4's angles and the corrected ones then give speeds that differ by
about 7.7 % of *V* on the edge shapes 1 to 4 (29.31° against 22.21°), some 3 leptons per tick at
*V* = 40, and by about 9.4 % on the corner and valley shapes 5 to 12 (21.36° against 30°), some 4
leptons per tick. On the steep shapes 13 to 16 (29.31° against 30°) they differ by only about 0.6 %,
some 0.24 leptons per tick at *V* = 40, which the run does not reliably tell apart: there it tests
the reflection against the flat rule only, and is inconclusive between C4's and the corrected
angle.
*Deck over a slope:* where a test map can hold a high bridge whose deck lies over a sloped cell,
a throw onto the deck there. *Expected:* F8 with the ground cell's shape. *Falsified by:* the flat
rule. Where no test map can hold one, this run is not made and claim 17's deck sentence stays
`unmeasured`.

*Part f (cliff faces, claim 18):* plateaus 2 height levels above the throwing ground, laid out so
that the old cell of a move undone by the cliff test faces:
- a straight face in each of the four straight step directions,
- for each diagonal step, the four layouts of F9's table: only the cell of the step high, also *P*
  high, also *Q* high, and both high.

Two types are used:
- the fixed-path test type, whose step is always north-west (−0.5 leptons per tick in each axis,
  F1). Its creation point lies equally far into its cell in *x* and in *y*, so that both cell
  coordinates change in the same move, and its creation height and vertical limits are chosen
  from F1 and F3 so that it reaches the plateau's edge less than 100 leptons below the plateau's
  ground, falling,
- a type with a horizontal limit of 1.5, whose speeds are −1.5, −0.5 or 0.5 leptons per tick in
  each axis (F1), and both vertical limits 0, so that its vertical speed starts at 0. It is
  thrown many times for the other directions, from creation points within 4 leptons of the face
  (of both faces, for a step that crosses a corner), at a creation height that puts its start
  (F1) between 5 and 10 leptons below the plateau's ground. At its slowest speed toward the face
  it reaches the face within 10 turns, by which *g* (claim 5) has lowered it by less than 80
  leptons, so it too reaches each face less than 100 leptons below the plateau's ground, falling.

*Record:* as the setup. A throw is judged only where the recorder's positions show a move undone
by the cliff test (claim 10); its step direction and the layout give the expected face. Every
throw of the fixed-path type is also judged at its move into the plateau's cell: that move is
expected to be undone (claim 10's descending condition), and a fixed-path piece left in the
plateau's cell after it falsifies claim 10.
*Expected:*
- the velocity after the turn is F9's, computed from the logged velocity at the start of that
  turn, with the vertical speed kept in sign and scaled by *e*, except in the north-west turn
  (claim 18, F9's north-west row), whose speeds the third item gives,
- in the north-east and south-west layouts, the faces of F9's table, which differ from those of
  the cells beside the corner,
- for the fixed-path type with only the northern cell high, an *x* speed of about 2.3771236 × 0.5
  ≈ 1.19, a *y* speed of about −0.25 and a vertical speed of about −0.25 leptons per tick
  (*u*ₓ = *u*ᵧ = −0.5).

*Falsified by:* any other face where the two differ: in particular the faces of the cells beside
the corner for a north-east or south-west step, or a reversed *y* speed in the north-west case.

*Part g (building cells, claim 19):* as part c, onto cells holding, in turn:
- an ordinary test building, and one under construction,
- a building of a neutral house,
- an open gate and a closed gate,
- an active section of a laser fence, and an inactive one,
- a vehicle-like building (S-0214 Terms),
- each of the three wall overlays, with no building,
- a resource overlay, with no building.

*Expected:* a building contact (part c's height set to the ground from less than 150 leptons above
it) on every cell except those with the inactive fence section, the vehicle-like building and the
resource overlay, where the first contact is with the ground. *Falsified by:* the other result on
any cell. No spec gives a route to an active laser-fence section, so that cell is set up only
where a run happens to hold one; without it, the active section stays `unmeasured` (claim 19).
*Part h (vertical speed 0 at a cliff, claim 10):* part a's cliff, with the lower ground lying
toward lower first-axis values and stretching at least three cells from the face. A named test
meteor type with vertical limits of 0 each, a horizontal limit of 8, a duration of 40 and an
elasticity of 0.5, thrown many times by a test warhead at a point on the higher ground next to
the face. Each move is then (*v*ₓ, *v*ᵧ, −1.4) and *u* is exactly 0 in every turn up to the first
contact (claim 10), and the start position lies at the creation point's height, the higher
ground's (F2). *Qualifying throws:* those whose *v*ₓ is positive and whose start position lies
over the lower ground at least one move short of the face, so that the move entering the higher
cell is not the first. After *k* ≥ 1 earlier moves that move ends 1.4(*k* + 1) leptons below the
higher ground, a ground contact set to that ground, which lifts the piece 1.4*k* leptons above its
height before the move, more than 1. *Expected:* for each qualifying throw, the move is undone:
a bounce animation at the position before the move, over the lower cell, at that position's
whole-lepton height (claims 7 and 11; the flight position before the move lies 1.4*k* leptons
below the higher ground), and next turns that move away from the face. *Falsified by:* for a qualifying
throw, the piece on the higher cell's ground after that move, which a test that undoes no move
with *u* = 0 would give. Both of claim 10's conditions undo these moves, so this part does not
tell which of them *u* = 0 takes.
*Part i (height before the move, claim 10):* part h's cliff. A named falling test type with a
horizontal limit of 255, vertical limits of 197.5 and 198.5, an elasticity of 0 and an expiry
animation, thrown many times by a test warhead detonating on the lower ground at the centre of
a cell next to the face, with the random-state recorder. By F1 each vertical speed is 197.5 or
198.5 and each horizontal speed a whole number from −255 to 254. *Qualifying pieces:* those whose
recorded creation point lies at the lower ground's height and whose logged raw values give, by
F1, a first move that ends in the higher cell next to the creation point's cell: an *x* speed
that carries the start position across the face and a *y* speed that keeps it in its row. With
the creation point at the cell's centre that is an *x* speed above 128 and a *y* speed of
magnitude below 128, about 1 piece in 8. The first move then ends 1.9 or 0.9 leptons below the
higher ground, which stands 208 leptons above the lower ground, so the move ends 206.1 or 207.1
leptons above the lower ground: a ground contact less than 100 leptons below the higher ground,
set to it. Measured from the flight position before the move, the start
position 10 leptons above the lower ground, that lifts the piece by 198 leptons, less than
*u* + 1, so the move is not undone; measured from the position before the move, the creation
point, it would lift it by 208, more than *u* + 1, and the move would be undone. The piece
rises (*u* is 197.5 or 198.5), so only claim 10's rising condition applies, and the run tells a
*z*₀ taken from the flight position's height from one taken from the position's. A test that
undid every contact putting the piece higher than before its move, whatever its vertical
speed, would undo it from either height. *Record:* the logged raw values, the per-tick flight
position, each bounce animation and the expiry animation. *Expected:* for each qualifying piece,
in its first turn a stop on the higher cell at the higher ground's height, with no bounce
animation (claim 9: the rebound scaled by 0 leaves it at rest on the ground, so *S* is 0), and
its end in its next turn, with the expiry animation on the higher cell at that height.
*Falsified by:* for a qualifying piece, a flight position over the lower cell at the end of its
first turn, or its expiry animation on the lower cell (claim 10: an undone move leaves the piece
at its start position, 10 leptons above the lower ground), which a *z*₀ taken from the position
(claim 3), or a test that undid every contact putting the piece higher whatever its vertical
speed, would give.

**V8. Expiry damage and the deck.** *Setup:* a fixed-path test type with damage 50 and a test
warhead that names a particle system (S-0303 claim 20), has an animation list of one test type
other than the piece's expiry animation and has a positive maximum debris count (S-0303 claim 22),
ending next to an infantry object, once with an expiry animation and once without; and once ending
on a high bridge deck above water. *Record:* health changes, the light flash, any particle system,
any animation of the warhead's list and any new piece. *Expected:* damage, flash and the warhead's
particle system only with the expiry animation, and never an animation of the warhead's list or a
new piece; on the deck, the land case (expiry animation and damage), not the water case.
*Falsified by:* damage or a particle system without an expiry animation, no particle system with
it, an animation of the warhead's list, a new piece, or a splash on the deck.
*Exploding overlay:* the same type with its expiry animation, ending in a cell that holds a stock
exploding overlay (barrel). *Expected:* the overlay is removed and S-0303 claim 19's effects
follow, any barrel debris being a piece that flies as this spec says. *Falsified by:* the overlay
remaining.
*Deck threshold, without a bridge:* the same type with its expiry animation, its path set so that
it ends in flight over a water cell with no bridge, once at 415 and once at 416 leptons above the
cell's ground height (the recorder gives the end height; if the path cannot end at those heights
exactly, use the nearest end heights below 416 and at or above it, and report them). *Expected:*
the water case (wake and splash) at 415, the land case (expiry animation and damage) at 416.
*Falsified by:* the land case below 416, or the water case at 416 or above, which would mean
another threshold or a test for an actual bridge. End heights other than 415 and 416 back the
threshold only to within the bracket they span.
*No source object:* as V5 part f, short game off, with the type's expiry animation and the two
infantry objects next to the piece's end point, within the warhead's area. The thrower's shot
comes from a unit of the thrower's house that, like every other object of that house, stands
beyond the survivor's sight and weapon range, as there. *Expected and falsified by:* as V5 part f
(claim 11), for the thrower's house.
*Own house:* as V5 part g, with the type's expiry animation and the two infantry objects next to
the end point of a piece that ends in the impact cell, within the warhead's area. *Expected and
falsified by:* as V5 part g (claim 11).
In both parts the tally must judge the expiry's area damage alone, so no ground impact may hit
the two infantry objects. A ground impact hits only objects in the one cell of its new position
(claim 7), so the two infantry objects stand in cells next to the end point's cell, not in it,
and the first has health below the area damage that S-0303 gives it at its distance from the end
point, the survivor more than that. The recorder's position log shows the cell of every ground
impact of every piece; a run in which any piece has a ground impact in either infantry object's
cell is repeated.

**V9. No cap.** *Setup:* a named test warhead with a minimum debris count of 200 and a maximum of
201 (so 200 pieces each time, F6), whose list holds one type, V1's falling type that names no
particle system, with a per-type maximum of 200, fired 20 times in one tick through several test
units. Each allocation then assigns between 0 and 200 pieces, so the allocation draws vary, and the
recorder's log is read with S-0303 F6's table as in V1: each allocation value is followed by the
creation draws of each piece it assigns, in the per-piece count and order that V1 establishes
(claim 3 predicts four raw values and three ranged draws); a ranged request can use more than one
raw value (S-0202 claim 4). V9 is read after V1 has run. *Record:* the number of pieces made, by draws.
*Expected:* the creation draws of all 4000 pieces are present in the ticks of the 20
detonations, which need not all fall in one tick (S-0203 claim 5 can make a projectile miss a
turn) (this tests the
draws, not how many pieces enter or stay in play, claim 1).
*Falsified by:* missing draws. A cap above 4000 would pass, so V9 backs no cap up to that count
only.

**V10. Resources.** *Setup:* a named test resource-flag falling type and meteor type ending on
clear cells, some germinable and some not (one holding an overlay, one sloped). *Record:* overlay
changes, stages and the draws. *Expected:* new resource only on the germinable cells, at stage 0,
each with one overlay draw R(0, 11) and then its growth-list offer, in step-direction order for the
meteor. No draw for the other cells; nothing at an end at deck height. *Falsified by:* resource on a
cell that is not germinable, a different stage, or a different draw order.

**V11. Attached system.** *Setup:* a fixed-path test type and a named test meteor type, each naming
a smoke system. *Record:* the system's position each tick and the tick its emission stops.
*Expected:* the system is made at the creation point and then sits at the falling piece's
position (zero displacement), its flight point (claim 11) after each flying turn, so from the
piece's first turn it is 10 leptons above the creation point plus the flight so far, truncated
toward zero in each axis. For the meteor piece it keeps the whole flight, creation point minus
start position, as its displacement. It is marked when the piece ends. The system takes its
position in its own turn (S-0616 claim 5), whose place against the piece's turn is open (claim 6),
so in each tick its position may be the one the piece had before or after that tick's piece turn,
and a tick in which the system misses its turn (S-0203 claim 10) leaves its position unchanged.
*Falsified by:* a position other than these.

**V12. Sounds.** *Setup:* a named test type with a start sound and a stop sound, and a long
duration, thrown so that the piece crosses the fixed view from one side to the other. *Record:* a
screen and audio capture: whether the start sound's stereo position follows the piece across the
view, and whether any further sound is heard when the piece ends. Where the measurement harness can
log each sound started, with its tick and position, that log is used instead.
*Expected:* the start sound at creation, moving with the piece; no stop sound. *Falsified by:* a
stop sound.

**V13. Zero span.** *Setup:* named test falling types with a horizontal limit of 0.25, of −0.25,
and of −0.5 (a span of −1, which does not stop), each thrown once. Also a falling type whose
lower vertical limit is 1 above its upper one (a vertical span of 0), and one whose spin limits
are a lower limit of 60 and an upper limit of 10 degrees per tick (a difference of about −0.87
radians, a spin span of 0), each with the other fields of the fixed-path test type. *Record:*
whether the program stops when the piece is created. *Expected:* a stop for 0.25 and −0.25 and
for the two further types, none for −0.5. *Falsified by:* a stop for −0.5, or none for any of the
others.

**V14. Off the map** (claims 20 and 21). *Setup:* a small test map whose legal cells are all at
height level 4, those outside the usable area included, with no low bridge, played in a freshly
started program (or one whose earlier games had no low bridge reaching off their map), so that
the original's stand-in cell stays blank (S-0009 claim 44). The ground there is 416 leptons
high, so a contact on a legal cell can be told from one off the legal map, at height 0.
- *Falling run:* a named test falling type with a horizontal limit of 15, an elasticity of 1, a
  duration of 3000 and an expiry and a bounce animation, thrown many times by a test warhead near
  one edge of the usable area. Some pieces head out across the legal cells outside the usable
  area and off the legal map.
- *Resource run:* the same falling type with the resource flag.
- *Meteor resource run:* a variant test map whose usable area reaches its outermost legal cells
  along one edge, clear, flat and germinable there, if the map's size fields allow one (if not,
  this run is not made and the meteor neighbours' test stays medium). A named test meteor type
  with the resource flag, no spawn type, a duration of 100, a horizontal limit of 15 and vertical
  limits of −10, thrown so that some pieces end just off the legal map beside that edge.
- *Meteor run:* a named test meteor type with a duration of 200, a horizontal limit of 15,
  vertical limits of −10, a spawn type and a spawn count of 2, thrown near the same edge, so
  that some start positions (F2) lie off the legal map.

*Record:* each piece's position and lifetime counter each tick (the read-only recorder, which
follows objects off the map too), each animation made with its position, overlay changes, and the
draws.
*Expected:*
- no piece leaves play for being off the map: each ends only as claim 6 says, in a turn whose
  counter is not positive, its lifetime run out or its counter set to 0 by a stop or a ground
  impact on water (claims 8 and 9),
- on legal cells outside the usable area, contacts at their ground height of 416 leptons,
- off the legal map, ordinary ground impacts at height 0, each with a bounce animation and no
  water case; a stopping turn there makes no bounce animation (claim 9), and a contact undone by
  the cliff test is left where claim 10 leaves it,
- a piece that comes back toward the map from off it meets the map's edge as claim 10's cliff
  test says, the legal cells being 4 levels above the off-map cell's level 0 (claim 18),
- at a falling piece's end off the legal map, the expiry animation there and, for the resource
  run, no new resource and no resource draws,
- in the meteor resource run, an end just off the legal map makes new resource, with its draws,
  on each of its neighbours that is a legal, germinable cell, in step-direction order, and on no
  other cell,
- in the meteor run, pieces placed at start positions off the legal map that fly in and spawn on
  the map as claim 12 says.

*Falsified by:* a piece removed in a turn whose counter, after that turn's reduction, is still
positive; a contact at height 0 on a legal cell outside the usable
area, which would show that such a cell is read as off the map; an ordinary ground impact off the
legal map at another height; a water
case off it; resource or resource draws at a falling piece's end off it; in the meteor resource
run, no resource on a germinable legal neighbour of an end off the map, or resource on a cell that
is not a neighbour; or a meteor piece with a start off the legal map that never appears or never
moves.

**V15. Drawing** (claims 22 to 28). Captured frames of a fixed view: every drawn frame, with its tick. *Setup:*
a named test model of two sections, each asymmetric under every rotation and distinctly coloured
(for example an L shape), and named test types that use it. *Record:* the frames, the creation
draws (random-state recorder) and the piece's position each tick (read-only recorder).
- *Part a (pose at creation, claim 22):* spin limits of 0.001 degrees per tick each, which give a
  spin rate too small to see (F5). *Expected:* the model is seen straight along its own *z* axis,
  its *x* axis to the right and its *y* axis up the screen, the same in every throw.
  *Falsified by:* the look of S-0105 claim 1's view, which a test vehicle with the same model as
  its body, facing east, shows beside it.
- *Part b (turn and light, claims 22 and 24):* spin limits of 10 degrees per tick each, an
  elasticity of 0.5, and a named test bounce animation, so that each contact that makes one is
  marked in the frames. *Expected:*
  - a turn about F5's axis, from the logged draws, of 10 degrees per flying turn, built up
    smoothly,
  - the sense reversed from each contact turn on, the contact turn itself included, for the
    contacts with its flat ground that make a bounce animation, which shows them (a stop or a
    ground impact on water makes none, and the piece flies no turn after it). The sense of one
    turn is read from the frames of its tick and of the tick before: the drawn model turns 10
    degrees on about the axis in the first sense and 10 degrees back in the reversed one, a
    difference of 20 degrees that the captured frames resolve,
  - the shades of F10's rotation applied to S-0306 F16.

  *Falsified by:* a turn in facing steps, a turn about another axis, a sense kept through a
  contact or in the contact turn itself, or shades of a light that turns with the model. A run
  with no contact that makes a bounce animation is inconclusive for the reversal.
- *Part c (palette and brightness, claim 25):* a map with a low ambient light (S-0104). Pieces
  come from the destruction of a test vehicle of a house with a distinctive colour (S-0312
  claim 4), and from a detonation, with and without the resource flag. *Expected:*
  - the house colour on the remapped indices of the destroyed vehicle's pieces, and untinted
    start-up mine-table colours on the detonation's,
  - the detonation's pieces without the resource flag brighter than the others, which take the
    cell's level.

  *Off-map run (claim 20's ground light):* where the view can show a position off the legal map,
  the destroyed vehicle's pieces thrown past the legal map's edge on the same map. *Expected:*
  those pieces at brightness 1000, brighter than the same house's pieces on a legal cell. The run
  is inconclusive if no view shows a position off the legal map.

  *Falsified by:* a house colour on a detonation's pieces, which would also show that they hold
  the thrower's house (claim 1), or a brightness that does not follow this rule.
- *Part d (section and translucency, claim 23):*
  - one test type with the section field 1 and the translucent flag,
  - one that leaves both keys absent,
  - one that shares the body part of a test vehicle whose motion file moves that section visibly
    from frame to frame.

  *Expected:* only section 1, at 50 %, for the first; section 0, opaque, for the second; and for
  the shared part, the section drawn without its motion file's transform and unchanged in shape
  from frame to frame apart from the spin. *Falsified by:* another section, another translucency,
  or a shared part that follows its motion file.
  Part d does not tell frame 0 of the motion file from no transform, since the file need not move
  the section in frame 0 (claim 23).
- *Part e (depth, claim 26):* pieces falling and bouncing at several heights over cells directly
  behind a test building, as seen on screen, so that the building lies in front of the ground
  point below each piece. The view stays fixed from the game's start, with no view jump, and the
  building's whole drawn image stays in view. Frames are judged only while the recorded position
  keeps the piece's ground point in those cells. The two departures of S-0102 F1 and F2 then have
  no effect on the cut: the original anchors the building's upright profile at its last row, as
  F2 does, and the expected cut is computed with the original's depth reference for that view,
  32768 plus the view's top world row at the game's start (S-0102 F1), not with F1's fixed one.
  *Expected:* in every judged frame, the pixels of the piece's image
  that fall within the building's drawn image are hidden exactly where S-0102 F2 hides a flat
  reader with the bias −2 − *L*(*z*) behind that building, computed from the recorded position
  and the building's depth profile (its companion depth image of S-0102 F5 included where it has
  one), and every other pixel is drawn. Over a building without a companion depth image this is
  S-0102's cut of a flat reader over an upright writer: the piece is cut along a row, and its
  rows nearest the building's base can stay drawn when its ground point lies just behind that
  base. A frame in which none of the image overlaps the building judges nothing. *Falsified by:*
  a hidden part that differs from F2's prediction by more than a row; in particular the one a
  bias of −2 alone would give, without the term that cancels the height lift, which hides the
  piece further toward the building's base, by about three quarters of its lift in rows.
  *Second run (the profile):* a named test type whose model is a tall column of voxels, its
  section field naming it, thrown so that it bounces just behind the building's edge. *Expected:*
  the hidden part of the column is the one the flat profile gives (S-0102 claim 12 and F2).
  *Falsified by:* the upright profile's hidden part where the two differ. A frame in which they
  hide the same pixels is inconclusive.
- *Part f (shadow, claim 27):* the frames of parts a and b over flat ground, and a throw over a
  high bridge's deck. *Expected:*
  - a shadow on the ground under the piece, which keeps its screen place as the piece rises and
    falls, at F11's footprint,
  - over the deck, at the deck height,
  - in part a, the outline of the section's bottom face; in part b, the turning outline that
    narrows as that face turns toward the vertical.

  *Falsified by:* a shadow that follows the piece's height, no shadow, or the footprint of S-0105
  claim 30's view.
- *Part g (shroud, claim 28):* pieces thrown from a revealed cell across the edge of the shroud.
  *Expected:* the piece stays drawn across the edge and darkens with the shroud's light-alpha
  values, to black under complete shroud. *Falsified by:* a piece that vanishes at the edge, or
  that is drawn undarkened under the shroud.

**V16. Spawn loops** (claim 29). *Setup:* as V4 on flat clear ground with no water, the recorder
logging the scenario random stream and, each tick, the number of pieces in play. A named test
rules layer gives a test warhead a debris list of one named test meteor type X, thrown once, one
piece per throw. Run each part on five seeds, once on the original and once on our engine.
- *Part a (one type, ends in later ticks):* X has a duration of 40, its spawn type X and a spawn
  count of 2. *Record:* for 300 ticks or until no piece of X is left, at each end of a piece of X
  its two spawn draws and the number of further pieces made. *Expected (original):* the layer
  loads with no refusal and X's spawn type stays X; every end not on water makes exactly the sum
  of its two draws, also at ends of further pieces, with no limit on the count in play.
  *Falsified by:* an end of a further piece that takes no spawn draws or makes fewer pieces than
  its draws give, or a load that refuses or changes the layer.
- *Part b (one type, ends in the first turn):* as part a with a duration of 1 and a spawn count
  of 4. *Record:* whether the game passes the tick of the first piece's end, and the tick
  counter. Stop the run from outside after 60 seconds if the game stops responding; do not wait
  for memory to run out. *Expected (original):* on a seed whose chain grows, the tick of the
  first end does not complete normally; on a seed where it dies out, the logged ends make the
  pieces their draws give, and since a piece that takes its turn in that tick ends in it and
  each end costs at most one other object its turn (S-0203 claim 10), at most as many pieces of
  X are in play after that tick as ends were logged in it. *Falsified by:* a seed that passes
  that tick with more pieces of X in play after it than ends logged in it, or an end of a
  further piece that takes no spawn draws.
- *Part c (two types):* X names a second test meteor type Y as its spawn type and Y names X, both
  with a duration of 40 and a spawn count of 2. *Expected (original):* as part a, with pieces of
  X and Y in turn.
- *Engine runs:* parts a to c. *Expected (our engine):* the importer warns once for each loop
  (once in parts a and b, once for the pair in part c); the thrown piece takes its two spawn
  draws and makes their sum, and none of those further pieces takes a spawn draw or makes a
  piece; every tick completes.

## Changes for other specs

- **S-0303 claim 22 and S-0312 claim 4 (F1):** the debris pieces they throw are objects of play,
  created with the draws of S-0317 claims 3 and 4; their behaviour after creation is S-0317's.
- **Glossary, *debris*:** add that S-0317 owns what a voxel-model piece of debris does after its
  creation.
- **S-0616 claim 1, the table row "A voxel-model animation with an attached system" and the
  sub-bullet *Animation with a voxel model*:** the sub-bullet's "at the animation's start position"
  should read "at the piece's creation point". The system is made at the piece's creation point,
  which is not its start position: for a meteor piece the start is moved back along its velocity
  (S-0317 claim 4), and for a falling piece it is 10 leptons higher (S-0317 claim 3); see S-0317
  claim 13.
- **S-0202 claim 10, animation creation draws, the bouncing-type row:** the row applies to the
  sprite animations of S-0109 (an animation type with the bouncer or the meteor flag). A
  voxel-model debris piece is not such an animation: its creation draws are S-0317 claims 3 and 4.
  This limits the row's scope only and leaves its content for sprite animations unchanged, which
  Q-1762 covers.

- **S-0302 claim 1, its table of sources:** add a row for a voxel-model debris piece's impact
  hits (S-0317 claim 7) and its expiry damage (S-0317 claim 11): no attacker and no attacking
  house.
- **S-0303 claims 19 and 23:** claim 23 could name a debris piece's expiry damage (S-0317 claim
  11) beside the disc laser as a producer that applies claims 1 to 20 and none of the
  projectile-only effects of claims 21 and 22. Unlike the disc laser, a piece's end also makes
  the warhead's light flash when the warhead has one (S-0317 claim 11). Claim 19's barrel debris could be named as pieces of
  S-0317, as claim 22's voxel-model debris is.
- **S-0009 claim 44:** its rule covers a read whose cell is not a legal cell. In the original some
  positions far off the legal map, and some neighbours of positions very far off it, compute a
  cell that is a legal cell elsewhere on the map, and the original reads that legal cell (S-0317
  claim 20; V4 part f for the reading in *y*). Claim 44 could add that such a read is not a read
  outside the map in the original, and that our engine reads the stand-in for it (S-0317 claim
  20's design rule).
- **S-0302 claim 8, *No-health targets*:** its list of sources that it cannot place under a
  filter names a debris piece's ground-impact hits with "S-0317 claim 7, which leaves this open
  for them". S-0317 claim 7 settles it: those hits skip no listed object for having no health
  left, so the object receives its hit and the no-health rows decide what the hit does, as for a
  bouncing animation's landing hits (S-0319 claim 7). Claim 8 can name them as a known source of
  such hits.
- **S-0312 claim 21, a building awaiting removal:** "it cannot be hit" holds for detonations
  (S-0303 claim 9). A debris piece's ground-impact hits (S-0317 claim 7) still reach it, since it
  stays listed in its cells; the hit is stopped with no side effect by S-0302 claim 8's
  no-health row for a building.
- **S-0007 claim 30:** its "with that part's motion file" concerns loading a shared part. A debris
  piece that draws a shared part does not apply the motion file when drawing it (S-0317 claim 23).

## Open questions
- How the original draws a piece whose three spin-axis draws are all 0 (F5), about one creation in
  2 × 10¹⁵. Our engine draws it without spin. This is drawing only and changes nothing in play.
- The hits of one ground impact (claim 7): whether an object that a hit removes from play stops the hits
  on the objects listed after it, and whether an object that enters the cell during the hits is
  hit. S-0319 claim 7 states for a sprite animation's hits, which go through the listing the same
  way, that a removed object ends them; S-0613 claim 13 states for its gas hits that a removed
  object does not stop the hits on the others. Which holds for a piece is not established, though
  the walk's likeness to S-0319 claim 7's favours that claim. The listing order of those
  hits is rated low and `unmeasured` for the same reason (claim 7).
- Whether a dying infantry unit, a crashing vehicle or aircraft (S-0302 claim 8) or a sinking
  vehicle (S-0203 claim 6(a)) stays listed in a cell's ground layer, where a ground impact's hits
  (claim 7) would reach it with no health left, is not known. A piece that bounces on a water cell
  (Edge cases) next to a sinking vehicle at 0 health would be a set-up that shows such a hit, if
  the vehicle stays listed there. That case would then belong with the notes on S-0302 claim 8
  and S-0312 claim 21 that Changes for other specs lists.
- The hits of claim 12's further pieces (medium, `unmeasured`): claim 1's reading, not yet shown
  for a further piece. A V4 run whose spawn type carries damage and whose further pieces end on
  V5 part f's layout, with the meteor thrown as in part g, would test the house.
  The attacker would stay `unmeasured`, as in claim 1.
- The original's behaviour in claim 12's fall-backs: when the overlay type at position 127 is not
  a resource overlay or does not exist, and the draws it takes when the chosen tiberium has never
  been given an image number. Our engine makes no new cell and takes no draws in both (design).
- Q-1702 question 4, a cap on the pieces in play: none is known (claim 1). V9 tests the creation draws of 4000
  pieces, not how many pieces enter or stay in play, nor that all the detonations fall in one
  tick; that no piece made is later removed for the number in play, and
  that no larger count meets a cap, are `unmeasured`.
- Whether every producer of pieces makes them in the order of its own list (S-0303 F6, S-0312 F1).
  This spec assumes the producer's order.
- The motion measure (F6) when the sum of the three squares reaches 2⁵³ or more: how that sum is
  rounded before the table root is taken.
- Whether a test map can hold a high bridge whose deck lies over a sloped cell (claim 17's deck
  sentence, V7 part e). If none can, that sentence stays `unmeasured`.
- The original's behaviour for a move of more than one cell at a cliff (claim 18) and for a section
  index outside the model (claim 23): both are undefined in the original, and our engine's rules
  are design.
- Where a piece's attached particle system (claim 13) stands in the update order, and so whether,
  under `quirks.update_order_removal_skip` of S-0203, a piece's end in a tick costs its system
  its turn in that tick. No record is known that shows it: the system is marked in the tick its
  piece ends (claim 13), and V11 records only its position and the tick its emission stops.
  Our engine places the system directly after its piece (design).
- Loops that pass through links of another family are not covered by claim 29's rule: for example
  a piece whose expiry, bounce or trailer animation leads into a loop of animations (S-0319), or
  whose expiry damage destroys an object or an exploding overlay that throws pieces of the
  piece's type again (S-0312 claim 4, S-0303 claim 19). Whether such a loop can close within one
  pass is not established. A piece's end makes no warhead debris (claim 11), so a warhead's
  debris list is reached from a piece only through such a destruction.
