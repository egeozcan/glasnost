---
id: S-0317
title: Debris pieces with voxel models (creation, flight, bounces, end of life, removal)
topic: combat
revision: 34
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-23
measurements: []
requests: [Q-1702]
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
building or a cliff face (claim 10). The exact rebound off slopes and cliff faces is only partly settled (Open
questions).

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
   is proposed to test. Whatever house its producer gives it (S-0312 claim 4 gives the pieces of a
   destroyed object that object's house; a detonation's pieces are taken to have none, which S-0303
   claim 22 does not state and V5 part f tests), every
   hit a piece deals (claims 7 and 11) is dealt by no attacker and on behalf of no house, its
   further pieces have no house (claim 12) and its attached system no owner house (claim 13). So
   the ownership of S-0312 claim 4 changes no outcome of a piece. A hit on behalf of a house would
   show: a destruction by
   a hit with an attacking house and no attacker is credited to that house (S-0302 claim 14), and
   so counts among that house's kills in the score-screen tally of S-0414 claim 21 (after the
   amendment that Changes for other specs lists). That tally is
   the sense of credit that V5 parts f and g observe: part f for the thrower's house, part g for
   the piece's own house. Both parts tell a hit with a house from one without only if that
   amendment holds in the original (V5 part f).

   | Part of the claim | Backing | Confidence |
   |---|---|---|
   | A piece is an object of play whose creation takes draws | `measurable` (V1) | high |
   | Every creation draw is taken for up to 4000 pieces | `measurable` (V9, read with the per-piece count that V1 establishes, claim 3) | medium until V1 and V9 run |
   | Every one of those pieces is made | `unmeasured`: V9's draws cannot tell it from a limit that takes the draws and makes no piece | medium |
   | No piece made is later removed for the number in play; no limit for larger counts | `unmeasured`: V9 does not record removal, and no finite scenario shows every count | medium |
   | The hits have no attacker | `unmeasured`: V5 part f judges house credit, not an attacker object, and judges no retaliation | medium |
   | The hits have no house, so neither the thrower's house nor the piece's own house is credited | `measurable` (V5 part f for the thrower's house, V5 part g for the piece's own house; V8's matching parts for the expiry damage, claim 11), once V5 part f's check of the S-0414 claim 21 amendment has run | medium until V5 parts f and g and that check run |
   | A detonation's pieces have no house | `measurable` together with the hits' lack of a house only (V5 part f: a credit to the thrower would show that the pieces hold the thrower's house and that their hits carry it). Once V5 part g shows that a piece's hits do not carry its own house, the house a detonation's pieces hold changes no outcome | medium until V5 part f runs |

2. **Lifetime at creation.** A piece's lifetime counter starts at its type's duration field. A
   meteor piece then loses *L* of it, 0 to 19 turns (claim 4, F2). For a meteor piece whose
   duration is −2147483648 + *j*, with 0 ≤ *j* ≤ 19, the loss can wrap the counter (S-0205): when
   *L* > *j* it becomes 2147483648 + *j* − *L*, a value from 2147483629 to 2147483647; when
   *L* = *j* it is exactly −2147483648, which claim 6 wraps at the piece's first turn; when
   *L* < *j* it stays negative and the piece ends in its first turn (claim 6). The duration
   −2147483648 itself (*j* = 0) therefore always wraps, at creation or at the first turn. Where F2
   then places a piece with a wrapped counter, and what follows, is open (Open questions).
   - Backing: `measurable` (scenario V2 for the duration field; V4 for a meteor piece's loss,
     through its start position, which F2 moves back by the shortened lifetime)
   - Confidence: high except as stated. The meteor pieces whose counter wraps are `unmeasured`
     and low: their placement is open.

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
   a piece's creation takes the four raw values above. Changes for other specs limits that row to sprite animations accordingly,
   and the reading stays medium until V1 runs (Questions for the project lead). The count rests on
   that amendment and on the lead's question. V1 decides it: four raw values before the first
   ranged draw confirm it, and three falsify it.
   - Backing: `measurable` (scenario V1 for the draw count and the order of raw and ranged draws, V3 for
     the velocity from the logged raw values)
   - Backing for the one raw value whose signed reading is −2147483648 (F1, under
     `quirks.random_magnitude_overflow`): `unmeasured`, as the setup cannot select the seed that
     makes that one value in 2³² occur (S-0202 claim 6).
   - Confidence: medium for the draw count until V1 runs, since it departs from S-0202 claim
     10's bouncing-type row as now written; it does not rise to high before V1 has run. Medium
     also for the order of the draws this claim names among themselves, the four raw values
     included, until V1 runs and while the lead's question on the scope of S-0202 claim 10 is open
     (Questions for the project lead). High
     that they are taken from the scenario random stream. Medium for the velocity's precision
     (F1). Medium for the case of that one raw value: it rests on F1's form, not on a run.

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
   - Backing: `unmeasured` for the fourth raw value's use as the spin rate and for which ranged
     draw gives which component of the spin axis: no scenario here records a piece's spin, so
     these are medium
   - Confidence: medium for the draw count until V1 runs, as in claim 3 (not high before V1 has
     run), and medium for the order of the draws this claim names among themselves until V1 runs
     and while the lead's question on the scope of S-0202 claim 10 is open, as in claim 3. High for the sign change and the start position
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
   object's next turn (a failed placement is open, Open questions); until its first turn a falling
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
   - whether these hits skip an object listed in the cell that has no health left, as S-0613
     claim 13's gas hits do, is open (Open questions). For the one such object known to stay
     listed in a cell, a building awaiting removal with 0 health (S-0312 claim 21), the answer
     changes nothing: a hit on it would be stopped with no side effect by S-0302 claim 8's
     no-health row for a building, so a skip and a hit give the same result.
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
    cell's centre height (medium: `unmeasured` on a sloped cell, since no run here throws pieces
    onto a slope; flat cells are covered by V3).
    - **High-bridge deck.** When the cell of the new position or of the old position is a
      high-bridge cell (a cell that an intact high bridge marks, S-0211 claim 15): a
      move from at or above the deck height to below it puts the height of the flight position at
      the deck height (on the deck); a move from below the deck height to at or above it puts it
      20 leptons below the deck height (under the deck). A low bridge has no deck: a low-bridge
      cell is not a high-bridge cell, so a move there is tested as on any other cell, by the
      ground test below at the cell's ground height, with the land type of its bridge overlay
      (S-0211 claim 9). The cells of a destroyed high bridge are no longer marked (S-0211 claim
      15) and are tested in the same way.
    - **Ground.** Otherwise (no deck is crossed, whether or not the cell is a
      high-bridge cell: Terms, *ground impact*), when the new height is below the ground height, the move reaches the
      ground. The height is set to the ground height when it is less than 100 leptons below it, and
      is otherwise left where it is, below the ground (Edge cases).
    - **Building cell.** Otherwise, when the new height is at least the ground height and less
      than 150 leptons above it, and the cell of the new position holds a building, the move is a
      contact and the height is set to the ground height. Some buildings do not count, and some
      overlays count as a building does (Open questions).
    - **No contact.** Otherwise the move has no contact: no ground impact and no rebound (the stop
      of claim 9 can still end the piece).

    At every contact that the cliff test below does not undo, the velocity rebounds off the surface of the cell of the new position and is
    scaled by the type's elasticity *e*. On a flat cell the vertical speed is reversed and the
    whole velocity is scaled by *e*: *v*′ = *e*(*v*ₓ, *v*ᵧ, −*v*_z). On a sloped cell the component
    across the cell's surface is reversed instead (Open questions). The **cliff** test is made
    for a contact whose new position is in a cell at least 2 height levels above the cell of the
    old position. Let *u* be the piece's vertical speed at the start of the turn, before gravity,
    *z*₀ the height of its position before the move (not of its flight position) and *z*₁ the height the contact has just set. The move is undone
    when:
    - the piece is descending (*u* < 0) and *z*₁ > *z*₀: the contact would put the piece higher
      than it was before the move, or
    - the piece is rising (*u* > 0) and *z*₁ > *z*₀ + *u* + 1: the contact would lift the piece
      more than 1 lepton above the height its own vertical speed, without gravity, would have
      carried it to.

    Our engine applies the two conditions as written, so a piece whose *u* is exactly 0 meets
    neither. Where the original's line between them lies, for a piece whose *u* is zero or within a
    small fraction of a lepton per tick of zero at the top of its arc, is `unmeasured` (below).

    When the move is undone, the flight position and the velocity become those at the start of
    the turn, the velocity is then turned off the cliff face and scaled by *e* (Open questions),
    and the ground impact's position (claim 7) is the position before the move. So a piece that
    the contact would lift onto a higher cell, by more than its own motion carried it, bounces off
    the cliff face instead, while a rising piece that the contact lifts by little enough climbs
    onto the higher cell.
    - Backing: `measurable` (scenario V3 for flat ground; V7 part a for the cliff test as far as
      its throws reach, the deep move and the undoing of a rising piece's contact; V7 parts b to d for the high-bridge deck and
      the low bridge, the building cell and the 100-lepton limit). The set-back of a descending
      piece less than 100 leptons below the higher ground, and a rising piece that climbs onto it,
      are `unmeasured`: V7 part a's throws stay far below a ground 2 height levels higher, and no
      run here throws a piece from higher up. The line between the cliff
      test's two conditions is `unmeasured`: it matters only for a piece whose vertical speed is
      within a small fraction of a lepton per tick of zero at the start of a turn, which F1 and F3
      make only by chance, and no scenario builds one. Our engine's line at *u* = 0 is not claimed to
      match the original's within that margin.
      Confidence: medium. The boundary of the ground test
      (a height exactly at the ground height is no contact outside a building cell) is
      `unmeasured`: F3's single-precision steps make an exact landing on the ground height rare,
      and no run here arranges one; medium.
    - Confidence: medium for the flat rebound, the high-bridge rules, the low bridge, the 100-lepton limit and the
      150-lepton building band, until V3 and V7 run. Low for the rebound on a sloped cell, the
      turn off a cliff face, and which buildings and overlays count. Medium for the cliff test's
      two conditions (until V7 part a runs, and `unmeasured` for the cases above that it does not
      reach).

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
    warns for it as that claim requires.

    A type with the resource flag, at an end that is not on water and not at deck height, adds new
    resource: a falling piece to its own cell, a meteor piece to each of the 8 neighbouring cells
    in step-direction order 0 to 7. Each such cell that is germinable (S-0401 claim 10) becomes a
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

15. **Spin.** The spin rate is formed by F5 and turns the drawn model each tick about the spin
    axis. It has no effect on play. Its raw value and the three axis draws are taken at every creation
    that completes (claims 3 and 4; claim 16 for a span of 0).
    - Backing: `measurable` (scenario V1 for the draws). The drawn rotation is `unmeasured`: it is
      presentation only, and no scenario here reads the model's orientation from captured frames
      (V3 part b and V12 capture the trailer's point and the start sound, not the spin).
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
the angle reached after many turns. No scenario here reads the drawn orientation (claim 15).
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
and the creation draws of each spawned piece in turn, then the resource draws.

## Edge cases and quirks

- **Expiry damage needs an expiry animation** (`quirks.voxel_debris_expiry_needs_animation`,
  default on). A piece's end deals area damage and makes the light flash only when its type names
  an expiry animation (also when the damage amount is 0, claim 11). With the switch off, the area
  damage and the flash happen at every end that is not on water, or on water at deck height,
  whenever the type names an expiry animation (as with the switch on, an amount of 0 included) or
  the damage amount is not 0.
- **The impact distance is lifted** (claim 7, F4). The distance passed to S-0302 is the lifted
  Manhattan sum, smaller than the lepton distance. This is the same rule as S-0613 F7 and has no
  switch of its own here (Questions for the project lead).
- **A deck impact hits the ground layer** (`quirks.voxel_debris_deck_impact_layer`, default on,
  claims 7 and 10). A piece whose ground impact puts it on a high-bridge deck hits the objects
  under the deck in that cell, not those on the deck. With the switch off, a ground impact that
  the deck test puts at the deck height hits the deck layer of the cell of the new position when
  that cell carries a high bridge, as a detonation on the deck does (S-0303 claim 4), and every
  other ground impact hits the ground layer. S-0607 has the same kind of fault for a dropped unit's
  landing, under a switch of its own: see Questions for the project lead.
- **A deck impact over water is a water impact** (`quirks.voxel_debris_deck_over_water`, default
  on, claims 8 and 11). A ground impact whose position before the move is in a high-bridge cell
  over water makes no bounce animation and no hit even when the deck test puts it on the deck, and
  the piece ends in its next turn on the deck with the first row of claim 11. With the switch off,
  a ground impact that the deck test puts at the deck height is treated as on land: claim 7 applies
  to it (its bounce animation, and its hits in the layer that
  `quirks.voxel_debris_deck_impact_layer` chooses), and the lifetime counter is not set to 0 by
  it. An impact under the deck over water stays a water impact either way.
- **An impact hits one cell only** (claim 7). Objects of neighbouring cells within the radius are
  not hit. No switch is proposed: the one-cell limit is the original's observable behaviour
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
| rules, same section, `Spawns`, `SpawnCount` | spawn type and count *c* (claim 12) | voxel-animation name; integer | none, 0 |
| rules, same section, `IsTiberium` | resource flag (claim 12) | boolean | no |
| rules, same section, `AttachedSystem` | attached particle system (claim 13) | particle-system name | none |
| rules, same section, `StartSound`, `StopSound` | start sound (claim 14); stop sound read, no effect | sound names | none |
| rules, the general section, `Wake` | wake animation (claim 11) | as S-0312's Import mapping | as there |
| rules, the combat-damage section, `SplashList` | splash list (claim 11) | as S-0303's Import mapping | as there |

The type's drawing keys belong to S-0007 and do not change play.

Backing of the absent-key defaults: `measurable` for the duration (V2 run with the key absent), the
speed limits and the elasticity (V3 run with those keys absent: F1's ranges and the rebound factor,
and for the upper vertical limit only as far as F1's vertical span shows it, since any upper limit
from 4.5 up to but not including 5.5 gives the same span with the lower limit 3.5),
and the three animations (V3 run with those keys absent: no bounce, expiry or trailer animation
appears). The damage, radius and warhead defaults are `unmeasured`: with them a ground impact deals no
hit with a visible result, so no run tells them from other values that also give none. The spin
defaults are `unmeasured`: they change only the drawn rotation, which no run here records.
Confidence: high for the `measurable` defaults, except the exact upper vertical limit: medium, as
V3 shows only the span. Medium for the damage, radius, warhead and spin
defaults, and for the spin defaults' conversion to radians. The defaults of the meteor flag, the
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
differs is inconclusive. A run whose placement fails at that start is reported so (Open
questions).
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
the thrower's, but only if the amendment of S-0414 claim 21 that Changes for other specs proposes
holds in the original. Under claim 21 as it stands, a destruction without an attacking object
gives no house a kill, and the thrower's count would not change whether or not the hit carries a
house. So a separate run first checks the amendment, on the same map with the same two houses:
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
part tells the two apart only once part f's check of the S-0414 claim 21 amendment has shown a
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

**V7. Cliff, bridge and building** (claim 10). Named falling test types with chosen horizontal
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
a rising piece that climbs onto it, are recorded if they occur but are not expected to arise.
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
- **S-0414 claim 21:** "A destruction without an attacking object gives no house a kill" is
  broader than S-0302 claim 14 allows: that claim credits a destruction by a hit with an attacking
  house and no attacking object to that house. Proposed wording: "A destruction with neither an attacking
  object nor an attacking house gives no house a kill (S-0302 claim 14), but still gives the
  victim's house a loss when it qualifies." S-0317 V5 parts f and g read the tally in this sense.

## Open questions
- How the original draws a piece whose three spin-axis draws are all 0 (F5), about one creation in
  2 × 10¹⁵. Our engine draws it without spin. This is drawing only and changes nothing in play.
- The hits of one ground impact (claim 7): whether an object that a hit removes from play stops the hits
  on the objects listed after it, and whether an object that enters the cell during the hits is
  hit. S-0613 claim 13 states for its gas hits that a removed object does not stop the hits on the
  others; the same reading is likely here but is not established. The listing order of those
  hits is rated low and `unmeasured` for the same reason (claim 7).
- Whether a ground impact's hits (claim 7) skip an object with no health left, as S-0613 claim
  13's gas hits do. The original is believed to make no such test of its own (low), so that S-0302
  claim 8's rows for a target with no health would decide each such hit; no run can show it yet.
  For the only case known to arise, a building awaiting removal (S-0312 claim 21), a skip and a
  hit give the same result (claim 7), so either conforms there. Whether a dying infantry unit, a
  crashing vehicle or aircraft (S-0302 claim 8) or a sinking vehicle (S-0203 claim 6(a)) stays
  listed in a cell's ground layer is not known; a piece that bounces on a water cell (Edge cases)
  next to a sinking vehicle at 0 health would be a set-up that tells the two apart, if the
  vehicle stays listed there. If the absence is confirmed, S-0302 claim 8's sentence that no
  known source delivers a hit to a target with no health, and S-0312 claim 21's "cannot be hit",
  would need a note naming this source.
- The rebound on a sloped cell and the turn off a cliff face (claim 10): the velocity after the
  rebound, for each slope type of the cell, and after the turn, for each direction a cliff face
  can face, as a function of the velocity before it. This spec does not state it. The outcome to
  measure is the per-tick velocity that V7's recorder logs for pieces that rebound on each slope
  type and off cliff faces of each direction. Which buildings do not count as a building contact, and which
  overlays count as one, are not established. The ground height that claim 10's contact test and
  F6 read on a sloped cell is settled as S-0106's surface height at the piece's truncated position
  (claim 10), but no run checks it there; a slope run that logs each piece's position would. Confidence
  low for the rebound until scenario V7 and a slope run are made.
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
- Q-1702 question 4, leaving the map in flight: whether a piece that flies off the map is removed
  for that. It is believed not to be (low confidence, `unmeasured` until a piece can be followed
  off the map). What its ground impacts and its end read there as its
  cell (land type, ground height), and so when it stops or ends, is not established.
- What happens to a piece whose placement at creation fails, for example a meteor piece whose start
  position, moved back by up to its lifetime times its speed, lies off the map, as it may for a
  meteor piece whose counter wraps (claim 2). It may never take a turn and so never end.
- Whether every producer of pieces makes them in the order of its own list (S-0303 F6, S-0312 F1).
  This spec assumes the producer's order.
- The motion measure (F6) when the sum of the three squares reaches 2⁵³ or more: how that sum is
  rounded before the table root is taken.
- Where a piece's attached particle system (claim 13) stands in the update order, and so whether,
  under `quirks.update_order_removal_skip` of S-0203, a piece's end in a tick costs its system
  its turn in that tick. No record is known that shows it: the system is marked in the tick its
  piece ends (claim 13), and V11 records only its position and the tick its emission stops.
  Our engine places the system directly after its piece (design).

## Questions for the project lead

- The scope of S-0202 claim 10's bouncing-type row (claim 3; Changes for other specs): this spec
  reads it as covering sprite animations only, so that a piece takes four raw values. The reading
  needs the lead's confirmation, and V1 settles the count.
- Whether the lifted impact distance (claim 7, F4; Edge cases) gets a quirk switch. It is the
  same rule as S-0613 F7, so one switch would cover both specs.
- The proposed `quirks.voxel_debris_deck_impact_layer` (claim 7, Edge cases): whether the
  fault gets a switch, and whether it is one switch for pieces only or one shared with other
  landings that read the ground layer on a deck, such as the dropped unit of S-0607
  (`quirks.lift_deck_impact_reads_ground`).
- The proposed `quirks.voxel_debris_deck_over_water` (claim 8, Edge cases): whether a deck
  impact over water treated as a water impact gets a switch of its own or shares one with
  `quirks.voxel_debris_deck_impact_layer`, since both concern a ground impact on a high-bridge
  deck.
