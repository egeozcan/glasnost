---
id: S-0319
title: Bouncing animations (sprite debris and meteors): motion, landing and end
topic: combat
revision: 30
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-12
measurements: []
requests: [Q-1762, Q-1846, Q-1847, Q-2996]
supersedes: []
---

# S-0319: Bouncing animations (sprite debris and meteors): motion, landing and end

## Summary

S-0109 Terms define a *bouncing type*: an animation type with the bouncer flag or the meteor flag,
whose animation follows its own motion and is ended by its landing. This spec gives that motion
and that landing for sprite animations: the first velocity formed from the creation draws, the
flight under gravity, what counts as landing (ground, high-bridge deck, building cell, cliff), the
rebound that decides between an impact and a stopping landing, the effects of the landing and of
the end that follows at once, the new resource a landing makes, the ends without a landing, the
behaviour off the map, and the draws all of these take. It also gives the trailer animation that
any animation type can name, since moving bouncing animations are its main users, and what
animation types do whose trailer, spawned, bounce or expiry animations lead back to them. It
answers Q-1762, Q-1846, Q-1847 and Q-2996; the points left open are listed under Open questions.

Debris pieces with voxel models are not animations of this kind: S-0317 owns
them. The two share one flight and contact rule. This spec states it in full for sprite
animations; S-0317 states it for pieces, apart from the differences claims 6 and 7 name, and
its cliff test (S-0317 claim 10) agrees with claim 6 here. The main difference: a sprite
animation ends at its first landing, while a piece rebounds and flies on.

## Terms

Shared terms are in the glossary: *scenario random stream*, *raw value*, *ranged draw R(a, b)*
(S-0202 Terms), *tick*, *tick number*, *height level*, *land type*, *departure*, *importer
warning*, *random-state recorder*. RZ₂₄ is S-0205's rounding toward zero to single precision
(S-0205 Terms). S-0109's *stage*, *pass*, *remaining passes*, *follow-up type*, *loop delay*,
*start event*, *middle event*, *repeat factor* and *trigger-played animation* are used as there.
In addition:

- **Bouncing animation.** A sprite animation whose type is a bouncing type (S-0109 Terms) at its
  creation. A **meteor animation** is one whose type has the meteor flag at its creation; every
  other bouncing animation is a **falling animation**. A type with both flags makes a meteor
  animation (claim 1). The kind fixed at creation decides only the creation (claims 2 and 3).
  After a switch to a follow-up type, the per-turn meteor test of claim 4 and the water rows of
  claim 8 read the meteor flag of the type the animation has then (claim 11).
- **Creation point.** The point the creator names when it makes the animation (for example
  S-0312 F1's debris point).
- **Flight position.** A point in single-precision leptons that each bouncing animation holds
  from its creation, moved by its *velocity* (single-precision leptons per tick per axis) in each
  of its turns. Its *position in play* is the point at which the animation stands in the game; it
  is set to the flight position truncated toward zero in each axis at the end of each motion step
  (claim 4).
- **Motion step.** The part of a bouncing animation's turn that moves it (claim 4).
- **Landing.** A motion step whose move meets the ground, a high-bridge deck, a building's cell or a
  cliff face by the rules of claim 6 (a *contact*), or that leaves almost no motion (claim 7). A
  landing is an **impact landing** or a **stopping landing** (claim 7). The **landing point** is the position in play after the landing's motion step. After a move
  that claim 6's cliff test undoes, that is the flight position before the move, truncated: the
  position in play before the move, except in a falling animation's first turn, where it stands
  10 leptons above the creation point (claim 6).
- **Ground height.** The terrain surface height at a point (S-0106), in leptons; on a flat cell
  it is the cell's single height.
- **Deck height.** 416 leptons (4 height levels of 104) above the ground height at a point.
- **Motion measure.** The number *S* of S-0317 F6, formed from the velocity and from the height
  above the ground height after a move (claim 7).
- **Cell read for a position.** The cell that every reading of this spec takes for a position
  (*x*, *y*, *z*) in leptons. Its cell coordinates are (*c*ₓ, *c*ᵧ) = (trunc(*x*/256),
  trunc(*y*/256)), so a coordinate between −255 and 0 falls in cell 0. When that cell is a legal
  cell (glossary, S-0009 claim 1) the cell read is that cell; otherwise it is the **off-map
  cell** (claim 11). A position is **off the legal map** when its cell read is the off-map
  cell.
- **High cell.** In the cliff turn (claim 7, F7), a cell whose height level is at least the level
  of the cell of the old position plus 2.
- **Animation link.** A link from an animation type A to a type B, of one of five kinds: B is A's
  trailer animation (claim 12), A's bounce animation (claim 7), A's expiry animation (claim 8),
  A's spawn type when A's spawn count is above 0 (claim 9), or A's follow-up type (S-0109 claim
  15). The first four make an animation of B, and a follow-up link makes none. Links are taken
  from every type, whatever its flags, since a bouncing animation that switches to a follow-up
  type uses that type's fields (claim 11).
- **Link on a loop.** An animation link from A to B lies on a loop when A can be reached from B
  by animation links, B = A included.
- **Looped animation.** An animation made through a link on a loop. It stays looped for the rest
  of its life, through every follow-up switch. Whether an animation is looped is part of the
  saved state of a game.

## Behaviour

1. **Which animations move.** An animation moves as this spec says when its type has the bouncer
   flag or the meteor flag at the animation's creation. When the type has the meteor flag the
   animation is a meteor animation (claim 3), whether or not it also has the bouncer flag.
   Otherwise it is a falling animation (claim 2). An animation whose type has neither flag at
   creation never moves by this spec, even if it later switches to a follow-up type that has one
   (claim 11).
   - Backing: `measurable` (scenario BA6 part c for a type with both flags, BA8 part d for a type
     with neither flag whose follow-up type has the bouncer flag)
   - Confidence: high for a type with both flags. Medium for a type with neither flag at creation
     and a flagged follow-up type, until BA8 part d runs.

2. **Creation of a falling animation.** The animation is placed at its creation point, unless that
   point is exactly (0, 0, 0) (claim 11). Its flight position starts 10 leptons above the creation
   point, and its first velocity is given by F1 from three raw values on the scenario stream, taken
   in this order: *r*₁ (vertical speed), *r*₂ (second horizontal axis), *r*₃ (first horizontal
   axis). Then three ranged draws R(−65535, 65535) are taken; their values have no effect on the
   animation. The vertical speed lies from *Z*ₘᵢₙ up to *Z*ₘᵢₙ + |*W*_z| − 1 leptons per tick, where
   *W*_z is formed with a fixed upper vertical limit of 3.5 leptons per tick that no key changes
   (F1, `quirks.anim_fixed_upper_vertical_speed`). Each horizontal speed lies from −*M* to
   |*W*ₓᵧ| − 1 − *M*; for a negative span (claim 14) the remainder takes the span's size, so the
   same range holds with |*W*ₓᵧ|. These ranges hold for every raw value but −2147483648: while
   `quirks.random_magnitude_overflow` is on, that value keeps its sign (Formulas), its remainder
   is 0 or negative, and the speed on that axis then lies from −*M* − (|*W*ₓᵧ| − 1) up to −*M*
   (vertically from *Z*ₘᵢₙ − (|*W*_z| − 1) up to *Z*ₘᵢₙ). No speed limit, drag or spin applies.
   - Backing: `measurable` (scenario BA1, with the random-state recorder for the draws)
   - Confidence: high for the order of the draws, the formulas and the 10-lepton lift. Medium for
     the fixed upper vertical limit until BA1 part b runs. The raw value −2147483648 is
     `unmeasured`: a run meets it about once in 4 × 10⁹ draws. Medium for it, resting on S-0202
     claim 6.

3. **Creation of a meteor animation.** A meteor animation's creation takes three raw values: *q*₁
   and *q*₂ give its horizontal speeds and *q*₃ its travel time *T* of 51 to 70 turns (F2). Its
   vertical speed is the type's lower vertical limit *Z*ₘᵢₙ. When the first horizontal speed is less
   than minus the second, both change sign, so the meteor's two horizontal speeds never sum to less
   than 0 (F2). The animation is placed at its start position, the creation point moved back by *T*
   times its velocity (F2), unless that position is exactly (0, 0, 0) (claim 11), and its flight
   position starts there. Three ranged draws R(−65535, 65535) follow, as in claim 2. A meteor
   animation then flies in a straight line (claim 4), but the start position does not put the
   creation point on that line. Each turn's move takes the 1.4 of claim 4 off the vertical speed
   before the move and adds it back only after it, so every move is (*v*ₓ, *v*ᵧ, *Z*ₘᵢₙ − 1.4), up
   to the rounding of F3. After *T* turns the line is over the creation point horizontally and 1.4
   *T* leptons (about 71 to 98) below it. A descending meteor whose creation point is on the ground
   therefore meets the ground by its *T*-th turn, at the first turn whose move takes it below the
   ground height (claim 6), unless a stopping landing of claim 7 comes first. Its landing point lies
   short of the creation point along its line, by (*T* − *k*)(*v*ₓ, *v*ᵧ) horizontally for a landing
   in turn *k*.
   - Backing: `measurable` (scenario BA6)
   - Confidence: high for the draws, the sign change, the travel time range and the move of
     (*v*ₓ, *v*ᵧ, *Z*ₘᵢₙ − 1.4) in every turn, and for the start position of F2.

4. **Motion step.** In each of its turns, a bouncing animation first loses 1.4 leptons per tick
   from its vertical speed, then moves its flight position by its whole velocity (F3). The
   contact tests of claim 6 follow, and with them any rebound. A meteor animation then has the
   1.4 added back to its vertical speed, so its next move is again its vertical speed less 1.4:
   it flies in a straight line, sinking 1.4 leptons per turn faster than its vertical speed says,
   until it lands (claim 3). Its
   position in play is then set to the flight position truncated toward zero in each axis, in
   every motion step. There is no drag and no speed limit, and the game-wide gravity value of the
   rules does not change the 1.4. The motion step happens in every turn the animation receives,
   including its first update (S-0109 claim 5) and the updates of a loop delay (S-0109 claim 11),
   in which its stage does not advance. Until its first turn a falling animation stands at its
   creation point and a meteor animation at its start position.
   - Backing: `measurable` (scenario BA2 for the falling arc and its first-turn motion, BA2 part h
     for the independence from the game-wide gravity value, BA6 for the meteor's line, BA8 part e
     for the motion in the first update and during a loop delay)
   - Confidence: high for the gravity and the meteor's straight line. Medium for the independence
     from the game-wide value until BA2 part h runs, for the motion in the first update and during
     a loop delay until BA8 part e runs, and for the meteor's last-bit shortfall, the same as
     S-0317 claim 5's.

5. **The stage keeps playing while it flies.** Between creation and landing a bouncing
   animation's stage plays by S-0109 claims 5 to 16 like any other animation's, with S-0109's
   exceptions for bouncing types: while its type is a bouncing type, its advances produce no
   middle event and no damage over time.
   So a type that loops for ever (S-0109 claim 7) keeps looping in the air until it lands, and
   lands wherever its flight takes it. Its start sound, when still playing, follows its position
   in play; this is presentation only.
   - Backing: `measurable` (scenario BA2 part a, which records the image frames shown in flight).
     The start sound's position is `unmeasured`: the harness logs no sounds or where they play.
   - Confidence: high for the stage. Medium for the start sound's position (`unmeasured`).

6. **What counts as landing.** After the move of a motion step, with the old position before the
   move and the new one after it, the tests below are made in this order. The ground height and
   the deck height are taken at the new position, truncated toward zero in each axis.
   - **High-bridge deck.** When the cell of the new position or of the old position is a
     high-bridge cell (a cell that an intact high bridge marks, S-0211 claim 15): a move from at
     or above the deck height to below it puts the flight position's height at the deck height
     (landing on the deck); a move from below the deck height to at or above it puts the height
     at 20 leptons below the deck height (landing under the deck). Either is a landing. A low
     bridge has no deck: a low-bridge cell is not a high-bridge cell, so a move there is tested as
     on any other cell, by the ground test below at the cell's ground height, with the land type
     of its bridge overlay (S-0211 claim 9). The cells of a destroyed high bridge lose their mark
     (S-0211 claim 15) and are tested in the same way.
   - **Ground.** Otherwise, when the new height is below the ground height, the move is a
     landing. The height is set to the ground height when it is less than 100 leptons below it,
     and is otherwise kept below the ground (Edge cases, `quirks.bouncing_anim_deep_landing`).
     "Otherwise" includes a high-bridge cell
     whose deck the move does not cross: there the ground test and the building test below are
     made as on any other cell.
   - **Building cell.** Otherwise, when the new height is at least the ground height and less
     than 150 leptons above it, and the cell of the new position holds a building, the move is a
     landing and the height is set to the ground height. The cell holds a building for this test
     in either of two cases:
     - *A building is listed there.* The first building listed in the cell's ground layer
       (S-0209 claim 13) counts, unless it is an inactive laser-fence section (a laser-fence
       section that is not active, S-0211 Terms) or a vehicle-like building (S-0214 Terms). When
       the first building listed is one of those two kinds the cell does not count, whatever is
       listed after it and whatever overlay the cell holds.
     - *No building is listed there.* The cell counts when its overlay is one of the three wall
       overlays of S-0306's constants (S-0307 claim 11).

     Every other building counts: one in the construction state (glossary), whether in its
     build-up just after it is placed or being sold, one awaiting removal (S-0312 claim 21),
     one of any house or of no owner, a gate whether open or closed, and an active
     laser-fence section. No other overlay counts. Only the ground layer is examined, whatever
     the animation's height: an object on a high bridge's deck makes no building contact.
   - **Cliff.** After any landing of the three kinds above, the move is undone when the cell of
     the new position is at least 2 height levels above the cell of the old position and the
     height *z*₁ after the landing's adjustment above is higher than the move could reach. With
     *u*_z the vertical speed at the start of the turn (before the 1.4 is taken off) and *z*₀ the
     height of the flight position before the move (not of the position in play, which is the
     flight position truncated toward zero, Terms, and which in a falling animation's first turn
     stands 10 leptons below it, at the creation point, claim 2), that is: when *u*_z is below
     −0.0002 leptons per tick, when *z*₁ > *z*₀; for every other *u*_z, 0 included, when
     *z*₁ > *z*₀ + *u*_z + 1. The comparison with −0.0002 is made in double precision, with *u*_z
     (stored in single precision) widened exactly, and our engine draws the line at the same
     place. So an animation whose *u*_z is exactly 0 is undone when the landing lifts it more
     than 1 lepton above its height before the move. S-0317 claim 10 states the same test for
     pieces: the same line,
     the same condition for a vertical speed of 0 and the same *z*₀. After the first turn the two
     heights differ by less than 1 lepton, unless the height's conversion is out of the integer
     range (S-0205 claim 7). The flight position and the velocity then become those before the
     move, and the velocity is
     turned off the cliff face (claim 7, F7). The position in play is then set from that restored flight position,
     truncated toward zero (Terms), so the landing point is the position in play before the move,
     except in a falling animation's first turn, where it stands 10 leptons above the creation
     point at which the animation stood before the move. So an animation that
     the landing would lift onto the higher cell by more than its own motion carried it lands at
     the foot of the cliff, while a rising animation that the landing lifts by little enough
     lands on the higher cell.
   - **Cells read.** Every cell these tests take is the cell read for the position (Terms). Off
     the legal map that is the off-map cell of claim 11, S-0009 claim 44's stand-in, which our
     engine reads as a blank cell: flat, at height level 0 and ground height 0, of the clear land
     type, with no high-bridge mark, no building and no overlay. So there the
     ground test meets height 0 and no building contact is made. A deck is met off the legal map
     only through the cell of the old position, when a move leaves a high-bridge cell for the
     off-map cell, and is then tested as for any move from that cell. The cliff test compares height levels with level 0,
     so a move from off the legal map onto a legal cell 2 or more levels high is tested as at any
     other cliff.
   These are the *contacts*. A move with no contact is still a landing when its motion measure is
   below 2.5 (claim 7). Every landing ends the animation in the same turn (claims 7 and 8): a
   bouncing animation lands once.
   - Backing: `measurable` (scenario BA2 for the ground, BA4 for high-bridge decks and the low
     bridge, BA5 part a for building
     cells, part a′ for the kinds of building and the wall overlays, part b for the cliff test of a
     descending animation and part c for an animation whose *u*_z is 0, BA2 part i for the
     100-lepton limit, BA4's low run under the bridge for the ground test on a high-bridge cell
     whose deck the move does not cross, BA13 for the off-map cell). BA5 part c shows that an
     animation with *u*_z = 0 is undone. It cannot tell which of the two conditions undoes it: in
     that run both give the same result. Five items are `unmeasured`:
     - Which condition *u*_z = 0 takes, and the line at −0.0002. Only a vertical speed from −0.0002
       up to 0, or a lift of the landing by at most 1 lepton, would separate the two conditions.
       F1 and F3 make such cases only by chance, and no scenario here builds one.
     - The first-listed rule for buildings. No scenario here builds a cell that lists two
       buildings, which stock play rarely makes.
     - An active laser-fence section. No spec gives a route by which a scenario can make one
       (S-0211), so BA5 part a′ judges it only where a run happens to hold one.
     - A building awaiting removal. BA5 part a′ does not set one up: such a building stays listed
       in its cells only for the short time S-0312 claim 21 gives, and no scenario here times a
       landing into that window. S-0317 claim 19 leaves the same case `unmeasured`.
     - A building of no owner. S-0009 claim 27 drops a map object whose owner matches no house,
       so no map can place such a building, and no scenario here makes one in play. S-0317 claim
       19 leaves the same case `unmeasured`.

     The rising condition (*u*_z > 0) and the flight position's height as *z*₀ are `measurable`
     together (BA5 part e). Part e's first moves rise into a cell 2 levels higher, with *u*_z of
     197.5 or 198.5, and the two heights differ there by the 10-lepton lift of a falling
     animation's first turn. Its expected result, a move that is not undone, holds only when both
     rules hold: the descending condition would undo those moves from either height, and the
     rising condition measured from the position in play would undo them too. So part e
     falsifies the pair, and its failure would not say which of the two failed. It builds no
     rising move that the cliff test undoes, so it does not show the lift above *z*₀ + *u*_z + 1 at
     which such a move is undone. After the first turn the two heights differ by less than 1
     lepton unless the height's conversion is out of the integer range (S-0205 claim 7), and no
     scenario here resolves a lift that close to the line. The
     landing point of an undone first move of a falling animation, 10 leptons above the creation
     point, is `unmeasured`: in part e's band the move is not undone, and no scenario here builds
     an undone first move whose landing point is recorded.
   - Confidence: high for ground landings and for ending at the first landing. Medium for the
     high-bridge rules, the low bridge, the 150-lepton building band and the descending cliff
     test, until BA4 and BA5 run, and for the 100-lepton limit until BA2 part i runs. Medium for
     the ground test under a high bridge until BA4 runs, for which height *z*₀ is until BA5 part e
     runs, and for the landing point of an undone first move (`unmeasured`). Medium for the cliff test with
     *u*_z = 0 until BA5 part c runs, and for the rising condition with *u*_z > 0 until BA5 part e
     runs. Medium for the condition that *u*_z = 0 takes and for the line at −0.0002
     (`unmeasured`). Medium for the kinds of building and the wall overlays until BA5 part a′
     runs, for an active laser-fence section, a building awaiting removal and a building of no
     owner (`unmeasured`), and
     for the off-map cell's readings
     until BA13 runs. Low for the first-listed rule (`unmeasured`).

7. **Impact landing or stopping landing.** At a contact (claim 6) the velocity is first
   rebounded, and then the motion measure decides.
   - **Rebound.** At a contact that the cliff test does not undo, the velocity is reflected in a
     plane fixed by the slope shape (S-0106) of the cell of the new position, through that
     shape's original tilt transform (S-0106 table C4), and scaled by the type's elasticity *e*
     (F6).
     - On a flat cell, and on the saddle shapes, which C4 gives the identity, this is S-0317
       claim 10's flat rule: the vertical speed is reversed and the whole velocity scaled by *e*.
     - On the other shapes the plane's normal leans toward the downhill side, by C4's tilt (about
       29.31° for the edge and steep shapes, about 21.36° for the corner and valley shapes), not by
       the terrain as drawn. The component along that normal is reversed, so with a positive *e* an
       animation that comes straight down leaves with a horizontal speed toward the downhill side.
     - At a deck contact the rebound still takes the slope shape of the ground cell under the
       deck. Over sloped ground an animation therefore rebounds as off the slope, not off the flat
       deck: a bug of the original (Edge cases, `quirks.bouncing_anim_rebound_faults`).
   - **Cliff turn.** When the cliff test undoes the move, the velocity *u* of the start of the turn
     (before gravity) is turned off a face of the cliff and scaled by *e* instead (F7). Every face
     only exchanges or reverses the two horizontal components and keeps the vertical one, so
     *S* below is that of *e u* whichever face it is, and which face it is has no effect on a
     sprite animation. The one exception is the original's north-west turn of F7, in which the
     vertical speed takes the old second-axis speed: a bug of the original
     (`quirks.bouncing_anim_rebound_faults`). A move whose cell changes by more than one in either
     axis needs a horizontal speed of 256 leptons per tick or more, unless a coordinate's conversion
     is out of the integer range (S-0205 claim 7), which can change the cell by more than one at
     any speed. The original's face for such a move is undefined. This case is a **departure**:
     our engine takes the step direction from the signs of the two changes and turns the velocity
     as for that step (F7). The importer warns for a
     type when a bound *H* on its first horizontal speed's magnitude by claim 2's ranges, which F1
     and F2 share, is 256 or more: *H* = max(|*M*|, | |*W*ₓᵧ| − 1 − *M* |) and, while
     `quirks.random_magnitude_overflow` is on, also |*M* + |*W*ₓᵧ| − 1| for the raw value
     −2147483648. *H* may lie above every speed the type reaches, so the warning can come for a
     type that never makes such a move. No later turn changes the horizontal speed before the
     landing (F3; claim 6: it lands once).
   - **Measure.** After every motion step, with or without a contact, the motion measure *S*
     is then taken from the velocity the step has left (after the rebound or the cliff turn) and
     the height above the ground height, by S-0317 F6's form, before a meteor's addition of 1.4.
     The form is the same for a sprite animation and a piece. When *S* < 2.5 the step is a
     stopping landing, with or without a contact. Otherwise a step with a contact is an impact
     landing, and a step without one is no landing.
   - **After a cliff turn.** After an undone move the height is counted from the ground of the cell
     of the restored flight position, the lower cell. Between flat cells of level 0 or above that no
     high bridge marks, a descending animation whose move is undone stands more than 108 leptons
     above that ground. The higher cell is at least 208 leptons higher, and claim 6's snap reaches
     only less than 100 below it. The height also grows with the downward speed: the move is undone
     only when the position before it stood above the snap's reach by more than that speed, so a
     faster fall leaves a greater height above the lower ground. So with an elasticity *e* with
     0 ≤ *e* ≤ 1 and any turn but the north-west turn, *S* exceeds 150 however fast the fall, within
     the sums S-0317 F6 covers, and the landing is an impact landing. Between such cells, only a
     rising animation can be undone close to the lower ground, so in practice only for one can the
     turned velocity decide between the two kinds of landing.
   - A **stopping landing** has no effect of its own; the end of claim 8 follows.
   - An **impact landing** first makes the type's bounce animation, when it names one, at the
     position in play before the move (the one the previous turn left, or in the first turn the
     creation point or the start position), with no loop delay, repeat factor 1 and no reverse
     request. Then each object listed in the ground layer of the one cell that holds the landing
     point whose horizontal distance |Δ*x*| + |Δ*y*| from the landing point is at most the type's
     damage radius receives a hit of the type's damage amount truncated toward zero, with its
     warhead, with the same hit as a piece's ground impact of S-0317 claim 7 (no attacker, no
     attacking house, defences not ignored, no no-escape marker, the lifted distance of S-0317
     F4). The hits go through the cell's ground-layer listing in its order (S-0209 claim 13). The
     walk below is stated for sprite animations only: S-0317 leaves open whether a removed object
     stops a piece's hits (S-0317 Open questions). Its results:
     - a hit that takes its own object out of the cell's ground layer ends the hits, and no object
       listed after it is hit. A destroyed vehicle that does not crash or sink leaves play at the
       end of its death effects, at the moment its damage is applied (S-0312 claim 15, S-0203
       claim 6 (a)), and so within its hit. For example, a vehicle
       standing in a building's cell is listed before the building, and when its hit destroys it
       the building takes no hit. Our engine ends the hits in the same way after a killing hit on
       infantry removed at once or on a building that leaves at once (S-0203 claim 6 (b) and
       (c)), whether or not the original takes them out within the hit (Open questions); design
       rule, decided under the delegation, 2026-10-27,
     - an object that a hit takes out of the listing before it is reached is not hit,
     - an object that enters the cell's ground layer during the hits is hit only when it is
       listed after the object being hit, which by S-0209 claim 13 only a building can be.

     This walk is a bug of the original (Edge cases, `quirks.bouncing_anim_impact_hits_walk`).
     With that switch on or off, each object's radius test and hit distance are taken at its own
     hit, from its position then (for the switch off a design rule, decided at the delegated
     sign-off, 2026-10-26: it keeps one distance rule for both settings).
     No object is skipped for having no health left: such an object receives its hit, and S-0302
     claim 8's no-health rows decide what the hit does. S-0317 claim 7 states the same for a
     piece's hits. The order matters only where one hit's
     result can affect another object of the cell, or where more than one hit takes draws from
     the scenario random stream (which draw each hit receives). The hits take the ground layer whatever the height of the
     landing: a landing on a high-bridge deck hits the objects under the deck in that cell, never
     one standing on the deck, as a piece's impact does (S-0317 claim 7). This is a bug of the
     original (Edge cases, `quirks.bouncing_anim_deck_impact_layer`). Then the end of claim 8
     follows.
   With an elasticity of 0 the rebounded velocity is zero, so a contact that leaves the animation
   at the ground height has *S* = 0 and is a stopping landing: such a type makes no bounce
   animation and no impact hits there. The height that *S* counts is S-0317 F6's: in a high-bridge
   cell (claim 6) it is counted from the deck when it is at least 416 leptons above the ground
   height, in a low-bridge cell from the ground height, and after a deep landing below the ground it is negative. So with an elasticity of 0 a
   landing on a deck is a stopping landing, while a landing under a deck or 100 leptons or more
   below the ground leaves *S* at 2.5 or more and is an impact landing. Whether a step without a contact
   is a stopping landing depends on *S*, which S-0317 F6 forms from the velocity and the height
   together (Terms). Unlike a piece's ground impact, an impact landing has no water
   test: on a water cell it makes the bounce animation and the hits too, before claim 8's water
   row (Edge cases, `quirks.bouncing_anim_water_impact_hits`).
   - Backing: `measurable` (scenario BA2 parts b, c and f, BA2 part j for the distance *D*, the
     radius limit (part j1), the defences and the no-escape marker, BA2 part m for the stop
     decision on a rebounded velocity at the ground, BA2 part n for the hits' end after a
     destroyed vehicle, BA3 part b for the bounce animation and the hits on water, BA5 part b for
     the impact landing after an undone descending move, BA5 part d for a sloped cell's rebound
     against the flat rule, BA11 part a for the order of bounce animation, hits and end). Nine
     items are
     `unmeasured`:
     - No attacker. BA2 part f judges house credit, not an attacker object, and no run here
       judges retaliation, as for S-0317 claim 7.
     - The rest of the listing walk and the hit on an object with no health left. No scenario here
       removes a later object during the hits, makes an object enter during them, or holds a
       no-health object in the landing cell. The end of the hits after infantry or a building
       that leaves at once is a design rule.
     - The ground layer at a landing on a deck. BA4 places no object on the deck or under it,
       and S-0317 V7 part b's run with objects tests the same rule for pieces.
     - A stopping landing without a contact. BA6, BA2 part j and BA8 part a′ admit a stop in
       flight only as an allowed alternative, and no scenario here pins one to a turn.
     - The north-west turn and the measure after a rising animation's undone move. No scenario
       here builds a rising animation whose move the cliff test undoes: BA5 part e's rising first
       moves meet a cliff, but are not undone.
     - The rebound over a deck, which takes the ground cell's slope. A run would need a high
       bridge over sloped ground, and no spec shows that a test map can hold one.
     - The exact plane of a sloped rebound. BA5 part d tells the sloped rebound from the flat
       rule only by whether the bounce animation appears, which does not tell one tilted plane
       from another.
     - The cliff turn's scaling by *e* as imported, not narrowed first (F7). BA5's cliff runs use
       an elasticity of 0, which scales alike either way, and no scenario here turns an animation
       off a cliff with an elasticity that single precision does not hold.
     - F6's stores and its narrowing of *e*: each completed vector component kept in single
       precision, not in double precision, and *ε* taken as *e* narrowed once to single
       precision, not as *e* as imported. BA5 part d judges the landing kind, which the rebounded
       velocity's truncated components decide, with an elasticity of 0.5, which single precision
       holds exactly, and BA2 part m
       judges the stop at whole-number thresholds. The roundings differ only in the last places
       of a component, so no run here tells them apart. S-0317 V3 part e tests the same
       narrowing for a piece's flat rebound, not for a sprite animation's.
   - Confidence:
     - High for the elasticity-0 case at the ground height, where the velocity and the height are
       both 0, so *S* = 0 under any form of S-0317 F6.
     - Medium for the stop decision on a rebounded velocity that is not 0 until BA2 part m runs,
       and for a sloped cell's rebound against the flat rule until BA5 part d runs. Medium for the
       exact plane of a sloped rebound, for the rebound over a
       deck, for the cliff turn's unnarrowed *e*, and for F6's single-precision stores and its
       narrowed *e* (`unmeasured`).
     - Medium for the impact landing after an undone descending move until BA5 part b runs.
     - Medium for a stopping landing without a contact on a flat cell, low on a sloped cell. It
       rests on S-0317 F6's form, which is `unmeasured` there and which sprite animations share.
       S-0317 rates that form medium on a flat cell and low on a sloped cell, where its ground
       height is S-0106's surface height and no run checks the stop. BA2 part m checks that form
       at a contact only. Medium that a sprite animation then ends in the same turn, where a piece
       ends in the turn after.
     - Medium for the bounce animation and the hits on water until BA3 part b runs, and for the
       ground layer at a deck landing.
     - Medium for the order of bounce animation, hits and end, as S-0317 claim 7 rates it, until
       BA11 part a runs. Medium for the design rule that ends the hits after infantry or a
       building that leaves at once.
     - Medium for the deck case with an elasticity of 0 until BA4 runs, and for the under-deck
       and deep cases, which rest on S-0317 F6's form.
     - Medium for the distance *D*, the radius limit, the defences and the no-escape marker
       until BA2 part j runs, and for no attacker. Medium for no attacking house until BA2 part f
       and its check of S-0414 claim 21's crediting run.
     - Medium for the listing walk's end after a destroyed vehicle until BA2 part n runs, and for
       the rest of the walk, the hit on an object with no health left and the north-west turn
       (`unmeasured`). The multi-cell move's turn is a departure, a design rule, not an oracle
       value.

8. **The end at a landing.** In the turn of its landing, after claim 7, the animation ends. Two
   tests are made at its landing point: *on water* (its cell has the water land type) and *at deck
   height* (its height is at least the deck height; this test is made at any cell, whether or not
   a high bridge marks it). Then, in this order:

   | Case | Effects |
   |---|---|
   | Not on water, or at deck height | When the type names an expiry animation: that animation at the landing point (no loop delay, repeat factor 1, a drawing-only depth adjustment), then a detonation's area damage there (S-0303 claims 1 to 20) with the type's damage amount truncated toward zero and its warhead, no source object and no house, resources affected, and the warhead's drawing-only light flash at the same point. With no expiry animation, none of these (`quirks.bouncing_anim_expiry_needs_animation`). |
   | On water and not at deck height, falling animation | The game-wide wake animation at the landing point, then the first entry of the game-wide splash list 3 leptons higher |
   | On water and not at deck height, meteor animation | The last entry of the splash list 3 leptons higher |
   | Not on water, or at deck height | After the first row: the type's spawned animations (claim 9) |
   | Resource flag, not on water and not at deck height | After the rows above: new resource (claim 10) |

   Then the animation leaves play. It does not advance its stage, make a trailer or switch to its
   follow-up type in that turn. The landing makes no sound, scorch mark or crater of its own:
   such effects come only from the animations it makes, by S-0109. Bounce, expiry and spawned
   animations of a type that leads back to the landing type are claim 15's case. The area
   damage's own effects follow their rules: the warhead's particle system when it names one
   (S-0303 claim 20), owned by no house, and an exploding overlay in the centre cell (S-0303
   claim 19). The projectile-only effects of S-0303 claims 21 and 22 are not made: no explosion
   animation from the warhead's animation list, no warhead debris and no nullify animation, as
   for S-0317 claim 11. The meteor test of the water rows uses the type the animation has at the
   landing (claim 11). A trigger-played animation (S-0109 Terms) deals the hits of claim 7 and
   the area damage of this claim like any other. With an empty splash list the water rows make no
   splash animation and the importer warns: a **departure** with the same result as S-0317 claim
   11's, since the original's result is undefined.
   - Backing: `measurable` (scenario BA2 for the first row, BA3 for the water rows, BA4 for the
     deck test in cells that a high bridge marks, BA2 part d for the absence of damage without an
     expiry animation and for no scorch mark or crater of the landing's own, BA2 part g for the
     warhead's particle system and the missing projectile-only effects, BA2 part f for no source
     object and no house, BA2 part k for resources affected, BA9 for no trailer in the landing
     turn, BA11 for the order of the effects)
   - Backing for the trigger-played case: `measurable` (BA2 part e)
   - `unmeasured`: that the deck-height test needs no high-bridge mark. An end at deck height in
     a cell that no high bridge marks needs a landing point at least 416 leptons above that
     cell's ground, such as a deck landing whose old cell is marked and whose new cell is not, and
     no scenario here makes one.
   - `unmeasured`: the warhead's light flash, with an expiry animation and without one, since it
     is drawing-only and the harness records no lighting.
   - `unmeasured`: that the landing makes no sound of its own, since the harness logs no sounds
     and a sound heard in a run cannot be tied to the landing turn among the dying vehicle's own.
     Also `unmeasured`: that the animation does not advance its stage or switch to its follow-up
     type in that turn, since it leaves play in that turn, so neither has a visible result, and no scenario
     here can make its last pass end in its landing turn.
   - Confidence: high for which row an end takes (land, water or deck height) in cells that a
     high bridge marks or that are not at deck height, and for the expiry-animation condition.
     Medium for the deck-height test in a cell that no high bridge marks (`unmeasured`). Medium for the order of the effects at an end, as claim 13 rates
     it, until BA11 runs. Medium for the splash lift of 3, for which splash entry each kind makes
     and for the meteor's splash row until BA3 runs. Medium for no source object and no house
     until BA2 part f and its check of S-0414 claim 21's crediting run, for resources affected
     until BA2 part k runs, for no scorch mark or crater until BA2 part d runs, and for the light
     flash, no sound
     and the stage and follow-up type in the landing turn (`unmeasured`).

9. **Spawned animations.** A type that names a spawn type and a spawn count *c* > 0 makes, at an
   end of claim 8's fourth row, *n* = R(0, *c*) + R(0, *c*) animations of the spawn type at the
   landing point (F4), one after another, each with no loop delay, repeat factor 1 and no reverse
   request, each taking its own creation draws (S-0202 claim 10) in turn. Both ranged draws come
   before the first creation. This holds for falling and meteor animations alike. A spawned type
   that is itself a bouncing type moves and lands by this spec. The sum *n* is a signed 32-bit
   integer that wraps, as for a meteor piece (S-0317 claim 12): when *c* > 1073741823 the two
   draws can add up to more than 2147483647, the wrapped sum is then negative, and no animation is
   made (the two draws are still taken). A spawn count of 2147483647 gives each draw the span of
   S-0202 claim 5, and the importer warns for it as that claim requires. A spawn type that leads
   back to the landing type is claim 15's case.
   - Backing: `measurable` (scenario BA7, part b for a meteor animation). The wrapped sum is `unmeasured`: it needs a spawn
     count above 1073741823 and a pair of draws whose sum passes 2147483647, which a run meets
     only by chance.
   - Confidence: high for the count and the order. Medium for the wrapped sum.

10. **New resource.** A type with the resource flag that ends not on water and not at deck height
    offers cells near its landing point to new resource, using its resource spread radius *R*
    (cells) and its resource spawn overlay type (F5). Each offered cell that is germinable
    (S-0401 claim 10), when the type names a spawn overlay type, receives an overlay of the type
    that stands one of 0 to 3 places after the spawn overlay type in the overlay list, R(0, 3),
    at the stage R(0, 2). The overlay draw comes first; the overlay is then placed, with the
    effects and draws of an overlay placement (S-0202 claim 10); the stage draw comes last, and
    the stage it gives is the cell's stage whatever the placement set. The cells offered are the landing cell's eight
    neighbours only, chosen by F5 (with radius 1, those in step directions 7, 0 and 1), not the
    cells of the radius
    (`quirks.anim_resource_spread_neighbours_only`). A type with the resource flag and no spawn
    overlay type makes no resource and takes no draws for it.
    - **The resource lists.** A cell that receives resource this way is offered to neither of
      S-0401's lists. It joins no growth list and no spread list, and takes no growth-list draw. A
      meteor piece's new resource cell (S-0317 claim 12) and the new resource cell of S-0401 claim
      11 are both offered to the growth list. This is a bug of the original (Edge cases,
      `quirks.anim_resource_no_growth_entry`).
    - **A cell with a terrain object.** This offer places no overlay in a cell that holds a
      terrain object, although S-0401 claim 10 counts such a cell (a tree's, for example) as
      germinable. Such a cell receives no overlay from the offer. Its overlay draw and its stage
      draw are still taken, and the stage has no visible effect. It stays germinable, so each
      further pair of F5 that offers it in the same landing takes both draws again. This rule is
      for a bouncing animation's resource offer only. It differs from S-0401's own new resource,
      which S-0401 claim 10 and its scenario E8 let a tree's cell receive; this spec does not
      cover that placement (Changes for other specs, S-0401 claim 10).
    - **The placement's own conditions.** Of the offered cell, the offer itself tests only that it
      is germinable. The placement it then makes is an overlay placement of S-0202 claim 10, with
      that claim's conditions as well as its effects. Of the types the offer can place, the vein
      overlay is placed only when S-0202 claim 10's vein placement test passes, and a
      bridge-building end marker has that claim's end-row results and never stays on the cell
      itself. A placement that such a condition refuses changes nothing and takes no placement draw.
      The offer's overlay draw and stage draw are still taken, and the cell stays germinable, as a
      cell with a terrain object does.
    - **Places of 255 or more.** In play a cell's overlay is identified by its place in the
      overlay list, however large. A place of 255 or more within the list is an ordinary overlay:
      the value 255 means no overlay only in the map file's overlay array (S-0012 claim 7).
    - **Past the end of the list.** When the drawn place lies past the end of the overlay list,
      which a spawn overlay type among the list's last three entries allows, the original's
      result is undefined: the cell may take an overlay that is no entry of the list, or the
      program may stop with an error. This case
      is a **departure**: our engine places no overlay and takes the overlay draw and the stage
      draw as usual, and the cell stays germinable, as a cell with a terrain object does. The
      importer warns for a type whose spawn overlay type has fewer than three entries after it in
      the overlay list.
    - Backing: `measurable` (scenario BA10, part b for a meteor animation, part c for the list's
      last entries and places of 255 or more, part d for a cell with a terrain object; BA11 part
      b′ for the placement's place among the draws, for the absent growth-list draw and for no
      growth-list entry). `unmeasured` for the absence of a spread-list entry: part b′ shows that
      the new cells never grow, which a spread-list entry would not change, and no scenario here
      empties a neighbour of a new cell or rebuilds the lists, which would show it. The past-list
      departure is a design rule, not an oracle value. `unmeasured` for the original's result
      past the end of the list: it is undefined, and BA10 part c only records what one run
      gives there. `unmeasured` for the placement's own conditions (a vein overlay or a
      bridge-building end marker as the placed type): no scenario here names such a spawn overlay
      type, and S-0202 claim 10 states those conditions.
    - Confidence: medium for the neighbour choice, the draws and the stage set after the
      placement. Medium for the growth list until BA11 part b′ runs, and for the spread list
      (`unmeasured`). Medium for the cell with a
      terrain object until BA10 part d runs, and for places of 255 or more until BA10 part c
      runs. Medium for the original's result past the end of the list (`unmeasured`, as above)
      and for the departure there, a design rule. Medium for the placement's own conditions
      (`unmeasured`).

11. **Ends without a landing, and follow-up types.** Apart from its owner's removal (S-0109
    claim 22), nothing ends a bouncing animation in flight except its stage (no time limit, no
    distance, no screen edge, no leaving the map: below). When its last pass ends:
    - with no follow-up type it ends and leaves play by S-0109 claim 16, including its exceptions, with none of claim 8's
      effects.
    - with a follow-up type it switches to it by S-0109 claim 15 and keeps its flight position,
      its velocity and its motion. It goes on moving and landing even if the follow-up type has
      neither flag. From then on the meteor test of claim 4 (adding the 1.4 back) and all of
      claims 7 to 10 use the follow-up type's fields, its meteor flag included. The kind fixed at
      creation decides its creation only (Terms): a falling animation whose follow-up type has
      the meteor flag flies on in a straight line from its velocity at the switch.
    So a type that loops for ever ends only by landing. The sentences below hold while the
    animation's type keeps the meteor flag it had at creation. A falling animation always comes
    down under gravity, so zero speeds do not keep it up. A meteor animation moves vertically by its lower vertical limit less 1.4 in
    every turn (claim 3). With a lower limit of 1.4 it keeps its height, so high above the ground
    it never lands and ends only with its passes. With a lower limit below 1.4 it sinks, and lands
    when a move meets the ground, a deck or a building cell by claim 6, on the map or off it
    (below). With a lower limit
    above 1.4 it rises in every turn: it lands when a move still ends below the ground height
    (for example a first turn whose start position F2 puts below the ground there) or crosses a
    high-bridge deck from below (claim 6).
    - Backing: `measurable` (scenario BA8; part c for a follow-up type with the meteor flag, part
      a′ for the sinking meteor). The rising meteor is `unmeasured`: whether its start position
      lies below the ground depends on the ground height where F2 puts it, and no scenario here
      fixes that.
    - Confidence: high for the pass end without landing effects. Medium for the absence of other
      ends over the map, for a follow-up type's continued motion and for its use of the
      follow-up's fields, until BA8 runs. Medium for the rising meteor.

    **Off the legal map.** A bouncing animation is never removed for leaving the map, at once or
    later: it takes its turns wherever it is. Every reading of claims 6 to 10 that takes the cell
    of a position takes the cell read for it (Terms). For a position off the legal map that is the
    **off-map cell**, S-0009 claim 44's stand-in. Our engine reads it as a blank cell at every
    position and at every time: flat (slope shape 0), at height level 0 and ground height 0, of the
    clear land type, with no overlay, no object listed in any layer and no high bridge. This is
    S-0009 claim 44's departure, with no switch. In the original the stand-in is one shared cell
    whose state can change: a low bridge whose end row or pieces reach outside the map gives it
    that overlay at map load, and every later read of any off-map cell, in the same game or in a
    later game of the same program run, sees that state (S-0009 claim 44). On a map with no such
    low bridge, in a fresh program run or one with no earlier game whose low bridge reached off its
    map, the original's stand-in reads as the same blank cell, and the rest of this claim describes
    that blank cell. So off the legal map an animation:
    - lands on the ground at height 0 and rebounds there as on a flat cell (claims 6 and 7), and
      makes no building contact there. The off-map cell has no high-bridge mark, so claim 6's
      deck test applies there only through the cell of the old position, on a move from a
      high-bridge cell into the off-map cell,
    - is never on water, so an impact landing makes its bounce animation and its hits, which find
      no object (claim 7),
    - at an end at a landing makes the first and fourth rows of claim 8 at its landing point: the
      expiry animation, an area damage in which cells off the map take part in nothing (S-0303),
      the light flash and the spawned animations (claim 9). An end when its passes are over has
      none of claim 8's effects, off the map as on it (above),
    - at an end at a landing, offers new resource (claim 10) to the eight neighbours of the
      landing point's cell, found as below. A neighbour that is a legal, germinable cell becomes
      a resource cell as on the map; any other takes no draws. Whether a neighbour of a position
      off the legal map can be germinable depends on whether the map's usable area reaches its
      outermost legal cells.

    A position outside the usable area but on a legal cell is on the map. Its cell is a real cell
    with its own height, land type, overlay and contents, and every claim applies to it unchanged.
    - **Neighbours.** A neighbouring cell (the steps of claim 7's cliff turn, F7, and claim 10's
      resource cells) is found by adding the step to the cell coordinates of the old position (for
      the cliff turn) or of the landing point (for the resource cells). When the sum names a legal
      cell, that cell is the neighbour. Otherwise the neighbour is the off-map cell.
    - **Far off the map.** Our engine reads the off-map cell for every position off the legal map,
      however far away, and for every neighbour that names no legal cell (design rule, decided at
      the delegated sign-off, 2026-10-25). The original departs from it for some positions far off
      the legal map and some neighbours of them, and, on a map whose legal cells reach the
      highest columns, for some positions just beyond the lowest or highest legal column: these
      read a legal cell elsewhere on the map. These are the cell readings S-0317 claim 20 states
      for pieces, since an animation's cell is read in the same way. Reading the off-map cell
      there keeps an animation that has flown far away free of terrain it is nowhere near.
    - **Placement.** An animation is placed at its creation point (a falling animation) or its start
      position (a meteor animation, F2), except a point of exactly (0, 0, 0) (below), with no test
      against the map's bounds, so a meteor animation whose start position lies off the legal map is
      placed there and flies in. All the creation draws (claims 2, 3 and 13) come at creation.

    No switch for the placement anywhere: our engine places an animation as the original does,
    at every placement point this claim establishes other than (0, 0, 0).
    - **The point (0, 0, 0).** Our engine does not place an animation whose placement point is
      exactly (0, 0, 0), as S-0317 claim 21 states for a debris piece. Such an animation has taken
      all its creation draws (claims 2, 3 and 13), but it never takes a turn, so it never moves,
      lands or ends by itself and is never drawn. This is a design rule (decided at the delegated
      sign-off, 2026-10-27), with no switch: pieces and animations then share one placement rule,
      and no producer in stock play is known to name that point. The original's behaviour is in
      Open questions.
    - Backing: `measurable` (scenario BA13, on a map with no low bridge, in a program run with no
      earlier such map: no removal, the landing at height 0, the end's effects off the map, a
      start position off the map, the ordinary behaviour of legal cells outside the usable
      area). Four items are `unmeasured`:
      - The original's off-map cell being the same blank cell at every position and every time on
        such a map in such a program run: BA13 samples only some, and S-0009 claim 44 leaves
        changes in play open. Our engine's blank cell is S-0009 claim 44's departure, not a
        measured fact.
      - The light flash of an end off the map: it is drawing-only and the harness records no
        lighting, as for the flash on the map (claim 8).
      - The readings far from the map. Our engine's reading of the off-map cell there is a design
        rule. For the original's far readings that it replaces: for a position about 2³¹ leptons
        off the map in *y*, S-0317 claim 20 and its V4 part f test the same cell reading with a
        piece, and no scenario here flies a sprite animation that far; the column readings and
        the far neighbours are reached by no scenario here or in S-0317.
      - The point (0, 0, 0): our engine's rule there is a design rule, and no producer here can be
        made to name exactly that point.
    - Confidence: medium until BA13 runs. Medium for the light flash off the map and for the
      design rules far from the map and at the point (0, 0, 0): choices, which no measurement of
      the original confirms.
      Medium for the original's far reading in *y*, until S-0317 V4 part f runs, since it rests
      on that run and on the shared cell reading; low for the original's column readings and far
      neighbours (`unmeasured`, as above).

12. **Trailer animation.** Every animation whose type names a trailer animation, bouncing or not,
    makes one in each of its turns that a landing does not end, while it is in play, after its
    motion step and before its stage update. An attached animation whose owner has been removed
    makes none at the update that deletes it (S-0109 claim 22). It does this on ticks whose tick
    number is divisible by the type's trailer separation *s*, or on every tick when *s* = 1. The
    trailer is made at the animation's position in play after the motion step, with an initial
    loop delay of 1 update, repeat factor 1 and no reverse request, the settings S-0109 claim 29
    gives a projectile's trailer. A negative *s* acts as its magnitude. The first update and the
    updates of a loop delay are such turns too: in them the trailer is made on the same
    tick-number cadence as in any other turn. A separation of 0, the absent-key default, makes
    the original stop with a program error at the first such turn (S-0205 claim 3). This is a
    **departure**: our engine treats a separation of 0 as 1, so a trailer is made in every such
    turn, and the importer warns for a type that names a trailer and no separation or a
    separation of 0. A trailer that leads back to the animation's type is claim 15's case.
    - Backing: `measurable` (scenario BA9, its third run for a negative separation; BA8 part e for
      the first update and a loop delay)
      The owner-removal case is `unmeasured`, for S-0109 claim 22's reason: that last update cannot
      be timed from outside.
    - Confidence: high for the cadence and the position. Medium for the creation settings and for
      the place before the stage update and for a negative separation, until BA9 runs, for the
      first update and loop delays until BA8 part e runs, and for the owner-removal case.

13. **Random draws.** A bouncing animation's flight takes no draws of its own. Its creation takes
    three raw values and then three ranged draws (claims 2 and 3), as S-0202 claim 10's
    bouncing-type row says, after its stage-delay draws (S-0202, S-0109 claim 3). In a turn, the
    draws come in this order: any trailer's creation draws (claim 12), then, in a turn with no
    landing, the stage update's own draws (S-0109); at an impact landing, the
    bounce animation's creation draws, then the draws of the hits (S-0302); at the end, the
    expiry animation's creation draws, the area damage's draws (S-0303), the two spawn-count
    draws and then each spawned animation's creation draws, then for each germinable cell that is
    offered resource, offers in F5's order, the overlay draw R(0, 3), the draws of the overlay's
    placement and the stage draw R(0, 2), with no growth-list draw (claim 10); on water,
    the wake's and then the splash's creation draws. All are on the scenario stream. The light
    flash takes none.

    The draws of the overlay's placement are all those that S-0202 claim 10's overlay placement
    gives for the placed type, in its order, and none when no overlay is placed (claim 10). The
    placed type can be any overlay type up to 3 places after the spawn overlay type, not only a
    resource overlay, so they can include, for example, a cell animation's creation draws, the
    tile-animation draws of a recalculated cell, and, for a low-bridge end marker that writes its
    end row and finds a partner, the pieces' raw values with the recalculation draws between
    them. The overlay itself takes none.
    - Backing: `measurable` (scenario BA11 with the random-state recorder: part a for a dry end
      with an impact landing and spawns, part b for the resource draws, part b′ for the placement
      draws among them, part c for the water end). `unmeasured` for the placement draws of a
      placed type that is no resource overlay, such as a low-bridge end marker: no scenario here
      names a spawn overlay type that reaches one, and those draws are S-0202 claim 10's.
    - Confidence: high for the creation draws and the spawn draws. Medium for the full order at
      an end, which follows from claims 7 to 10, until BA11 runs, for the absence of a
      growth-list draw until BA11 part b′ runs, and for the placement draws of a type that is no
      resource overlay (`unmeasured`, as above).

14. **Zero spans.** F1 and F2 divide by spans made from the type's fields. A span of 0 makes the
    original stop with a program error (S-0205 claim 3). For field values whose span sums
    (2*M* and *Z*ₘₐₓ − *Z*ₘᵢₙ + 1) lie within the signed 32-bit range the spans are 0 exactly when:
    - −0.5 < *M* < 0.5, for *W*ₓᵧ (both kinds),
    - 3.5 < *Z*ₘᵢₙ < 5.5, for *W*_z of a falling animation, with the fixed upper limit of
      claim 2 (under `quirks.anim_fixed_upper_vertical_speed` off, −2 < *Z*ₘₐₓ − *Z*ₘᵢₙ < 0).
    These intervals are exact in exact arithmetic. The spans are formed in floating point (F1 and F2,
    under S-0205), so a difference so close to an end of its interval that the floating-point sum rounds
    onto that end gives that end's span: the intervals hold up to that rounding.
    A negative span is not 0 and does not stop the original: the remainder then takes its size
    (F1). The one exception is a span of −1 with a magnitude value of −2147483648 (under
    `quirks.random_magnitude_overflow`), which faults as S-0205 claim 3 says; our engine gives it
    S-0205 claim 3's result. A span of 0 is a **departure**: our engine takes that remainder as 0,
    so every animation of the type gets the same speed on that axis, and the importer warns for
    such a type. S-0317 claim 16 gives pieces a different result for the same case.
    - Backing: `measurable` (scenario BA12 for the original's stop). The departure's result is a
      design rule, not an oracle value.
    - Confidence: high for the horizontal span. Medium for the vertical span until BA12 part b
      runs.

15. **Types that lead back to themselves.** An animation type can lead back to itself through
    animation links (Terms): it can be its own trailer, two types can name each other, or a
    bouncing type's spawn, bounce or expiry animation can be a type that leads back to it,
    directly or through follow-up types.
    - **At load.** The original neither refuses nor changes such data: the types load and play as
      claims 1 to 14 and S-0109 say. S-0004 claim 15's long-name case, in which a name reference
      creates types at load, is S-0004's and is not this claim's subject.
    - **Number of animations.** No limit of the original's own on the number of animations in
      play is known: it makes each new one for as long as the program can get room for it, and
      what happens when it cannot is open (Open questions). An animation made during the turn
      pass is added after every object already in the update order and takes its first turn later
      in the same pass (S-0203 claim 4), unless a removal costs it that turn (S-0203 claim 10).
    - **Trailer loops within one tick.** An animation's first turn reaches claim 12's trailer
      test, and whether a trailer is due depends only on the tick number and the type's
      separation, so an animation made on a tick on which its type is due makes its own trailer
      in its first turn, in that same pass. When a type's trailer is the type itself, or when
      every type of a loop of trailer links is due on a tick, each new animation of the loop
      makes the next in the same pass, the pass never ends and the game stops responding. On a
      tick on which some type of the loop is not due, the chain stops at an animation of that
      type. A bouncing animation of such a loop breaks it when its first turn is a landing, which
      makes no trailer (claim 8). A follow-up switch never comes in an animation's first turn
      (S-0109 claim 5), so a follow-up link does not close a loop within one tick.
    - **Landing loops within one tick.** A bouncing animation made by a landing, or as the
      trailer of a bouncing animation, lands in its own first turn when its first move makes a
      contact (claim 6) or is a stopping step (claim 7: a motion measure *S* below 2.5, formed by
      S-0317 F6 from the velocity and the height that the step leaves), with or without a
      contact. Two sufficient cases of a contact, each for a surface that is the same everywhere
      within reach of the first move, are these. For a falling animation made at a landing point
      on level ground with no other ground height, water or deck within reach of its first move,
      or on a deck away from its edges, a first vertical speed below −8.6 leptons per tick, since
      its flight starts 10 leptons above the point (claim 2) and its first move is that speed
      less 1.4 (claim 4); claim 2's draws give such a speed only with a lower vertical limit
      below −8.6, or with claim 2's raw value −2147483648 while
      `quirks.random_magnitude_overflow` is on. For one made at a landing point on a building
      cell of a building on flat ground, whose height claim 6 puts at the ground height, a first
      move that stays in a building cell with a first vertical speed below 141.4. Elsewhere claim
      6's heights at the new position decide whether the first move makes a contact. A stopping
      step without a contact can also end the first turn, at other speeds. A landing that makes
      an animation of a type that leads back to the landing type can therefore make the next
      landing in the same pass. Each landing ends its animation, a counting removal of S-0203
      claim 10, so while `quirks.update_order_removal_skip` is on it costs the object directly
      after it its turn in that tick, and when the landing animation stands last that object is
      the first animation it has just made. A loop that makes one animation per landing (for
      example an expiry animation of the landing's own type, landing on a building cell)
      therefore does not keep a pass going: from its second landing in a tick on, each new
      animation follows its maker directly and misses its turn, and the loop goes on at about one
      landing per tick for as long as its landings come. A loop that makes more than two
      animations per landing on average, each landing in its first turn, can keep a pass going
      without end, depending on the spawn-count draws (claim 9). With the switch off, any loop
      whose animations each land in their first turn and make at least one animation of the loop
      keeps the pass going.
    - **Across ticks.** A loop whose animations each make more than one animation of the loop
      over their lives on average, for example a spawn loop with a spawn count of 2 or more, or a
      trailer loop that a follow-up switch closes, can grow the number of animations in play
      without bound, tick after tick.
    - **Our engine: a departure, with no switch.** A looped animation (Terms) makes nothing
      through a link on a loop: no trailer, bounce animation or expiry animation through such a
      link, and no spawned animations through a spawn link on a loop. It takes none of the draws
      of a creation it does not make: neither the animation's creation draws (S-0202 claim 10)
      nor, for a spawn link, the two spawn-count draws of claim 9. The rest of its turn is
      unchanged: a landing whose expiry animation is not made still makes claim 8's area damage
      and light flash, which depend on the type naming an expiry animation, and an impact landing
      whose bounce animation is not made still makes claim 7's hits. Every other creation is
      unchanged, a looped animation's creations through links on no loop included, and the
      animations those make are not looped. A follow-up switch is unchanged, and the animation
      stays looped through it. The importer warns once for each loop: each largest group of types
      that all lead to one another through animation links (a group that no further type can
      join), a type with a link to itself and no other such type counting as a group. So a type
      on a loop, made by anything other than an animation link (a warhead, a debris list, a
      trigger), makes one generation through each link on the loop and no more: a type that is
      its own trailer makes one trailer per due tick, and those trailers make none. This departs
      from the original also for loops that the original can run, such as an expiry animation of
      the landing's own type, which the original keeps making for as long as its landings come.
    - Backing: `measurable` (scenario BA14 for trailer loops, BA15 for landing loops and the
      per-tick counts, both for the load). `unmeasured` for the absence of a limit on the number
      of animations, as S-0317 claim 1 rates the same for pieces: no finite scenario shows every
      count, BA14 and BA15 included. The departure is a design rule (decided at the
      delegated sign-off, 2026-11-12), not an oracle value. `unmeasured`: whether a never-ending
      pass would end once the program can get no room for a further animation, since the original
      then makes none, and what the game would do then. A run is stopped from outside long
      before.
    - Confidence: at load, medium for the absence of any check until BA14 and BA15 load such
      data. Number of animations: medium for the absence of a limit, and low for what happens
      when no room is left (both `unmeasured`). Trailer loops within one tick: medium that a new
      animation's first turn reaches the trailer test in its creation tick, as claim 12 rates
      that test until BA8 part e runs; high for the due test by tick number; medium for the
      never-ending pass until BA14 runs. Landing loops within one tick: medium for the first-turn
      landing conditions and the removal-skip consequence until BA15 runs. Across ticks: medium
      for the growth. Our engine: high for the departure, a design rule.

## Formulas

All raw values are signed 32-bit numbers. |*r*| is S-0202's "magnitude first" value (claim 6):
the absolute value, except that −2147483648 keeps its sign while
`quirks.random_magnitude_overflow` is on (the default). "rem" is the integer remainder truncated
toward zero, with the sign of the dividend; so a negative divisor acts by its magnitude. trunc is
truncation toward zero (S-0205 claim 7). Floating-point expressions are formed in double
precision (S-0205 claim 4) from the double-precision type fields, and each stored speed or position is narrowed to single precision, RZ₂₄ under S-0205's
`quirks.chop_rounding`.

**F1. Falling animation, first velocity.** With the type's horizontal speed limit *M*, its lower
vertical limit *Z*ₘᵢₙ, the upper vertical limit *Z*ₘₐₓ = 3.5 (fixed, claim 2) and the raw values
*r*₁, *r*₂, *r*₃ of claim 2:

> *W*ₓᵧ = trunc(2*M*),  *W*_z = trunc(*Z*ₘₐₓ − *Z*ₘᵢₙ + 1)
>
> *v*ₓ = RZ₂₄((|*r*₃| rem *W*ₓᵧ) − *M*),  *v*ᵧ = RZ₂₄((|*r*₂| rem *W*ₓᵧ) − *M*)
>
> *v*_z = RZ₂₄((|*r*₁| rem *W*_z) + *Z*ₘᵢₙ)

Each remainder is an exact integer; *M* and *Z*ₘᵢₙ enter in double precision, not narrowed first.
Units: leptons per tick. The flight position starts at the creation point (*c*ₓ, *c*ᵧ, *c*_z + 10),
each axis an integer converted to single precision, exactly for every point on a map (each
coordinate is below 2²⁴ in size).

**F2. Meteor animation.** With the raw values *q*₁, *q*₂, *q*₃ of claim 3 in the order taken:

> *v*ᵧ = RZ₂₄((|*q*₁| rem *W*ₓᵧ) − *M*),  *v*ₓ = RZ₂₄((|*q*₂| rem *W*ₓᵧ) − *M*),  *v*_z = *Z*ₘᵢₙ in single precision
>
> when *v*ₓ < −*v*ᵧ: *v*ₓ ← −*v*ₓ and *v*ᵧ ← −*v*ᵧ
>
> *T* = 70 − |*q*₃ rem 20|  (an integer from 51 to 70; the remainder is taken before the absolute value)
>
> start = (trunc(*c*ₓ − *T* *v*ₓ), trunc(*c*ᵧ − *T* *v*ᵧ), trunc(*c*_z − *T* *v*_z))

Each product and difference of the start is formed with S-0205 claim 4's 53-bit significand, from
the integer creation point, the integer *T* and the single-precision speed, and truncated once.
The product *T v* is exact (an integer below 2⁷ times a 24-bit significand), and a difference
rounded toward zero (S-0205 claim 5) truncates to the same integer as the exact difference, so
each coordinate of the start is the exact value trunc(*c* − *T v*), the same result as S-0317 F2
gives a meteor piece. Units: leptons, turns.

**F3. Motion step.** With *v* and *p* the velocity and flight position at the start of the step,
in single precision, and *g* = 1.4 (the single-precision value, widened):

> *v*′ = (*v*ₓ, *v*ᵧ, RZ₂₄(*v*_z − *g*)),  *p*′ = RZ₂₄(*p* + *v*′) per axis

then the contact tests of claim 6 on *p*′ (with any rebound), then, for a meteor animation,
*v*_z ← RZ₂₄(*w*_z + *g*), where *w*_z is the vertical speed the step has left. The position in
play is trunc(*p*′) per axis after the tests. For a meteor animation the restored *v*_z enters the
next step's *v*′ again less *g*, so each move is (*v*ₓ, *v*ᵧ, RZ₂₄(*v*_z − *g*)). The subtraction
and the addition are each rounded to single precision, so they need not cancel: under S-0205's
`quirks.chop_rounding` both round toward zero, as S-0317 claim 5 and F3 give it for meteor pieces.
After *k* turns the flight position is the start position plus about *k* (*v*ₓ, *v*ᵧ, *Z*ₘᵢₙ − *g*)
(claim 3). With a lower vertical limit of 1.4 (stored as the same
single-precision value as *g*), *v*_z − *g* is exactly 0, and adding *g* back gives 1.4 again
exactly, so *v*_z does not shrink.

**F4. Spawn count.** *n* = R(0, *c*) + R(0, *c*), with *c* the type's spawn count, the sum
formed in signed 32-bit arithmetic that wraps; no animation when *n* ≤ 0 (the two draws are
still taken). No draw and no animation when *c* ≤ 0 or no spawn type is named. Units: animations.

**F5. Resource cells.** With the landing cell (*X*, *Y*) and the radius *R* (cells), the pairs
(*i*, *j*) with −*R* ≤ *i* ≤ *R* and −*R* ≤ *j* ≤ *R* are taken with *i* in the outer order and *j*
in the inner, each from lowest to highest. A pair is used when trunc(√(*i*² + *j*²)) ≤ *R*. The cell
it offers is the neighbour of (*X*, *Y*) in step direction *i* mod 8 (least non-negative residue,
step directions 0 to 7 of S-0204). *j* does not change the cell. With *R* = 0 the one pair (0, 0)
offers the neighbour in step direction 0. Each offer tests whether the cell is germinable at that
offer, and a cell that is not takes no offer draws. A cell that received no overlay from this
offer (claim 10: a cell with a terrain object, a placement that one of S-0202 claim 10's
conditions refuses, or a place past the overlay list) stays germinable and takes both draws at
each offer. Under `quirks.anim_resource_spread_neighbours_only` off,
the pair offers the cell (*X* + *i*, *Y* + *j*) instead, in the same order. Neighbours are found
as claim 11 says.

**F6. Rebound** (claim 7). Let *v* = (*v*ₓ, *v*ᵧ, *v*_z) be the velocity that the turn's move
leaves (*v*′ of F3), and *M* the matrix of S-0106 table C4 for the slope shape of the cell of the
new position (at a deck contact, of the ground cell under the deck). *M*'s rows are as C4 lists
them; shapes 0 and 17 to 20 give the identity. Write *D* = diag(1, −1, 1), *Z* = diag(1, 1, −1)
and *ε* for *e* narrowed once to single precision. Then

> *w* = *M*ᵀ · *D* · *v*,  *w*′ = *Z* · (*ε* *w*),  *v*″ = *D* · *M* · *w*′

and *v*″ is the velocity after the rebound. Read the formula from right to left.
- **Stores.** Each completed vector component is stored in single precision: the three
  components of *M*ᵀ · *D* · *v*, each product *ε* *w*ᵢ, and the three components of
  *D* · *M* · *w*′. A change of sign is exact.
- **Sums.** A matrix-vector product groups its row sums as S-0613 F3 does: (*a*ᵧ + *a*_z) + *a*ₓ
  for *x*, and (*a*ᵧ + *a*ₓ) + *a*_z for *y* and *z*, with no fused operations.
- **Flat cell.** With *M* the identity, *v*″ = (*ε* *v*ₓ, *ε* *v*ᵧ, −*ε* *v*_z), each product
  stored.
- **Exact arithmetic.** *v*″ = *ε*(*v* − 2(*v* · *n*)*n*) with *n* = (*m*₁₃, −*m*₂₃, *m*₃₃),
  where *m*ᵢⱼ is *M*'s entry in row *i* and column *j*, to C4's precision (its rows are unit
  vectors only to single precision). The tilt of *n* from the vertical is arccos *m*₃₃. The stored
  result can differ from this exact form in its last place: the stores and sums above decide the
  result, and this form only describes it.

For the default elasticity *ε* is the single-precision 0.8; for an elasticity of 0.5 it is 0.5
exactly. With `quirks.bouncing_anim_rebound_faults` off, a deck contact takes the identity.
Units: leptons per tick.

**F7. Cliff turn** (claim 7). Let *u* = (*u*ₓ, *u*ᵧ, *u*_z) be the velocity at the start of the
turn, before gravity, (*x*, *y*) the cell of the old position and *d* the step direction (glossary)
of the cell of the new position from it. The turned velocity, before scaling, is one of
(±*u*ₓ, ±*u*ᵧ, *u*_z) or (±*u*ᵧ, ±*u*ₓ, *u*_z), the face deciding which, except in one case:

> **North-west turn.** When *d* = 7 and, of the cells (*x* − 1, *y*) and (*x*, *y* − 1), only
> (*x*, *y* − 1) is a high cell (Terms): (*c*, *u*ₓ, *u*ᵧ), with *c* = 2.3771235942840576.

Each component of the result is then multiplied by *e* as imported, in double precision and not
narrowed first (unlike F6), and stored in single precision. A reversal or an exchange leaves the
pair of horizontal sizes and the vertical speed unchanged, so for every face claim 7's measure *S*
is that of (*e* *u*ₓ, *e* *u*ᵧ, *e* *u*_z). After the north-west turn it is that of
(*c* *e*, *e* *u*ₓ, *e* *u*ᵧ), whose vertical speed is the scaled old second-axis speed. *c* is
a single-precision number a little below 2.38. With `quirks.bouncing_anim_rebound_faults` off, the
north-west case turns as a face across the second axis: (*e* *u*ₓ, −*e* *u*ᵧ, *e* *u*_z). For a
move whose cell changes by more than one in either axis, *d* is the step direction whose two
components have the signs of the two changes (claim 7's departure). Units: leptons per tick.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| gravity *g* | 1.4 (the single-precision value, widened) | leptons per tick per tick | every bouncing type, claim 4 |
| start lift of a falling animation | 10 | leptons | claim 2 |
| fixed upper vertical limit | 3.5 | leptons per tick | claim 2, F1 |
| meteor travel time | 70 − (0 to 19) | turns | F2 |
| creation draw bounds | −65535 to 65535 | — | three ranged draws whose values have no effect, claims 2 and 3 |
| stop threshold | 2.5 | — (S-0317 F6's plain number) | claim 7 |
| deck height | 416 | leptons above the ground height | 4 height levels of 104; the deck of a high bridge (claim 6) |
| under-deck landing | 20 | leptons below the deck height | claim 6 |
| ground snap limit | 100 | leptons below the ground height | claim 6 |
| building band | 150 | leptons above the ground height | claim 6 |
| cliff | 2 | height levels | claim 6 |
| cliff speed line | −0.0002 (double precision) | leptons per tick | claim 6: below it the descending condition |
| north-west turn constant *c* | 2.3771235942840576 (single precision) | leptons per tick, before scaling by *e* | F7 |
| multi-cell move warning | 256 | leptons per tick | claim 7: importer warning when the bound *H* on the first horizontal speed is at or above it |
| water splash lift | 3 | leptons | claim 8 |
| resource overlay choice | R(0, 3) | overlay-list places | claim 10 |
| resource stage | R(0, 2) | stages | claim 10 |

## State and ordering

| State | Event | Result |
|---|---|---|
| Created | creation | placed (falling: creation point, meteor: start position), except at (0, 0, 0) (claim 11); creation draws (claim 13); looped when made through a link on a loop (claim 15) |
| Flying | turn with no landing | motion step; trailer when due (claim 12); stage update by S-0109 |
| Flying | turn with a stopping landing | motion step; end (claim 8); leaves play |
| Flying | turn with an impact landing | motion step; bounce animation and hits (claim 7); end (claim 8); leaves play |
| Flying | last pass ends, no follow-up type | leaves play, no landing effects (claim 11) |
| Flying | last pass ends, follow-up type | switches type and keeps flying (claim 11) |

A bouncing animation takes its turns in the update order of S-0203 (claims 2 and 3). Its removal
at a landing is a removal for S-0203 claim 10 (`quirks.update_order_removal_skip`). Within a
turn: motion step, then either the landing's effects and end, or the trailer and then the stage
update.

## Edge cases and quirks

- **The upper vertical limit is fixed** (`quirks.anim_fixed_upper_vertical_speed`, default on,
  claim 2, F1). A sprite type's upper vertical speed key is not read: F1 always uses 3.5. With a
  lower limit of 5.5 or more the span is negative (between 3.5 and 5.5, both excluded, it is 0:
  claim 14) and the vertical speed lies from *Z*ₘᵢₙ up to
  *Z*ₘᵢₙ + |trunc(4.5 − *Z*ₘᵢₙ)| − 1, so it grows with the lower limit. With the switch off, F1
  uses the type's upper vertical speed field (Import mapping) for *Z*ₘₐₓ.
- **Expiry damage needs an expiry animation** (`quirks.bouncing_anim_expiry_needs_animation`,
  default on; claim 8). With the switch off, the area damage and the light flash of claim 8's
  first row are made at every such end whenever the type names an expiry animation or its damage
  amount is not 0. Same rule as S-0317's `quirks.voxel_debris_expiry_needs_animation`, kept as a
  switch of its own so that each spec's switch covers only its own objects.
- **A deck landing hits the ground layer** (`quirks.bouncing_anim_deck_impact_layer`, default
  on; claim 7). An impact landing on a high-bridge deck hits the objects under the deck in that
  cell, not those on the deck. With the switch off, an impact landing that claim 6 puts at the
  deck height hits the deck layer of the cell of the landing point when that cell carries a high
  bridge, as a detonation on the deck does (S-0303 claim 4), and every other impact landing hits
  the ground layer. Same rule as S-0317's `quirks.voxel_debris_deck_impact_layer`, kept as a
  switch of its own in the same family.
- **Resource goes to neighbours only** (`quirks.anim_resource_spread_neighbours_only`, default
  on; claim 10, F5). The radius selects pairs but the cell depends only on the first number of the pair, so
  at most the eight neighbours receive resource and the landing cell never does. Fixed behaviour
  in F5.
- **Deep landings stay below the ground** (`quirks.bouncing_anim_deep_landing`, default on,
  claims 6 and 7). A move that ends 100 leptons or more below the ground height is not put back
  on the ground, so its landing point, and the expiry animation, area damage and spawns made
  there, are below the ground, and claim 7's measure counts a negative height there. With the
  switch off, every landing on the ground (not on a deck or a building cell) is put at the ground
  height, however far below it the move ends, and claim 7 then counts a height of 0. S-0317 gives
  its pieces' deep move no switch; this switch covers sprite animations only, so S-0317's rule
  is unchanged.
- **No bounce.** A bouncing animation never rebounds into a second flight (claim 6); the type's
  elasticity matters only to the stop test of claim 7. No switch: this is the original's design.
- **Hits on water** (`quirks.bouncing_anim_water_impact_hits`, default on; claim 7). An impact
  landing on water makes its bounce animation and hits before the splash, where a piece's ground
  impact on water makes neither (S-0317 claim 8). With the switch off, an impact landing whose
  landing point is on water and not at deck height (claim 8's water test) makes no bounce
  animation and no hits, and the end of claim 8's water rows follows. No switch of S-0317
  covers this case, so this one is new.
- **Rebound faults** (`quirks.bouncing_anim_rebound_faults`, default on; claim 7, F6, F7). One
  switch covers two faults of the original's rebound, both of which change only claim 7's choice
  between an impact and a stopping landing:
  - a deck contact rebounds off the plane of the ground cell's slope shape under the deck,
  - the north-west cliff turn gives (*c* *e*, *e* *u*ₓ, *e* *u*ᵧ), which matches no face.

  With the switch off, a deck contact rebounds by the flat rule (F6 with the identity), and the
  north-west case turns across the second axis (F7). The switch covers sprite animations only.
- **Resource joins no list** (`quirks.anim_resource_no_growth_entry`, default on; claims 10 and
  13). A new resource cell of claim 10 is offered to neither of S-0401's lists. With the switch
  off, each such cell is offered to its tiberium's growth list as S-0401 claim 11 offers a new
  resource cell, and the offer takes S-0401 claim 7's raw draw when it adds an entry. That draw
  comes after the overlay's placement and its draws and before the stage draw R(0, 2). A cell
  that receives no overlay (claim 10) is not offered, and neither is a cell whose overlay is no
  resource overlay of any tiberium: it has no tiberium whose list could take it, and it takes no
  draw for the offer (design rule, decided at the delegated sign-off, 2026-10-25). This switch
  is separate from `quirks.anim_resource_spread_neighbours_only`, which chooses the cells.
- **Impact hits follow the listing** (`quirks.bouncing_anim_impact_hits_walk`, default
  on, claim 7). The hits go through the cell's listing as it changes, so a hit that takes its object
  out of the cell ends the hits, an object taken out before it is reached is not hit, and a
  building that enters within the radius is hit. With the switch off, the objects listed in the cell's ground layer
  when the hits begin each receive their hit in that order, unless an earlier hit has taken them
  out of play, and an object that enters during the hits receives none. The radius test and the
  hit's distance are taken for each object at its own hit, from its position then, as with the
  switch on (design rule, decided at the delegated sign-off, 2026-10-26: it keeps one distance
  rule for both settings). Either way the hits go
  through the layer that claim 7 hits: the ground layer, or, with
  `quirks.bouncing_anim_deck_impact_layer` off, the deck layer for an impact landing at the deck
  height of a high-bridge cell. So with both switches off, such a landing hits the objects
  listed in the deck layer when the hits begin.
- **No switch for the cliff line at −0.0002** (claim 6): our engine draws it where the original
  does.
- **No switch for the original's choice of face for a diagonal cliff step** (F7): it has no effect
  on a sprite animation.
- **The off-map cell is always blank** in our engine (claim 11): S-0009 claim 44's departure,
  with no switch. The original's stand-in can take a low bridge's overlay at map load, and every
  later off-map read sees it, in later games of the same program run too.
- **No switch for the placement anywhere** (claim 11): our engine places an animation as the
  original does, at every placement point claim 11 establishes. At the point (0, 0, 0) our engine
  does not place it: its creation draws are taken and it never takes a turn, as for a debris
  piece (S-0317 claim 21; design rule, decided at the delegated sign-off, 2026-10-27).
- **A far cell read as a legal cell** is a departure (claim 11). In the original some positions
  far off the legal map, and some neighbours of them, read a legal cell elsewhere on the map. Our
  engine reads the off-map cell for them (decided at the delegated sign-off, 2026-10-25). No
  switch: such positions are reached only far from play or beyond the edge of a map of the
  largest sizes, and a far cell's terrain there is a fault with no use in play.
- **Zero spans** (claim 14) and **a trailer separation of 0** (claim 12) are departures.
- **An empty splash list** is a departure (claim 8).
- **A move of more than one cell at a cliff** (claim 7, F7) and **a place past the overlay list**
  (claim 10) are departures.
- **Types that lead back to themselves** (claim 15) are a departure, with no switch: a looped
  animation makes nothing through a link on a loop and takes no draws for it. Where the original
  never ends the pass, its result has no use in play. The departure also bounds loops that the
  original can run.

## Import mapping

| Source | Concept | Parse form | Default when absent |
|---|---|---|---|
| the animation type's own art section (S-0109's Import mapping), `Bouncer` | falling animation (claims 1, 2) | boolean | no |
| same section, `IsMeteor` | meteor animation (claims 1, 3); also tested at each motion step and at a water end (claims 4, 8, 11) | boolean | no |
| same section, `MaxXYVel` | horizontal speed limit *M* (F1, F2) | floating point, leptons per tick | 15.0 |
| same section, `MinZVel` | lower vertical limit *Z*ₘᵢₙ (F1), a meteor's vertical speed (F2) | floating point, leptons per tick | 3.5 |
| same section, `MaxZVel` | not read for animation types: *Z*ₘₐₓ = 3.5 (claim 2). Read only under `quirks.anim_fixed_upper_vertical_speed` off | floating point, leptons per tick | 3.5 |
| same section, `Elasticity` | elasticity *e* (claim 7) | floating point | 0.8 (single precision, widened) |
| same section, `Damage`, `DamageRadius`, `Warhead` | landing hits and expiry damage (claims 7, 8); the damage amount is also S-0109's damage value | floating point (truncated toward zero when used); integer leptons; warhead name | 0, 0, none |
| same section, `BounceAnim`, `ExpireAnim` | bounce and expiry animations (claims 7, 8) | animation names | none |
| same section, `TrailerAnim`, `TrailerSeperation` | trailer animation and separation *s* (claim 12) | animation name; integer, ticks | none; 0 (acts as 1, claim 12's departure) |
| same section, `Spawns`, `SpawnCount` | spawn type and count *c* (claim 9) | animation name; integer | none, 0 |
| same section, `IsTiberium`, `TiberiumSpawnType`, `TiberiumSpreadRadius` | resource flag, spawn overlay type, radius *R* (claim 10) | boolean; overlay name; integer, cells | no, none, 0 |
| rules, the general section, `Wake` | wake animation (claim 8) | as S-0312's Import mapping | as there |
| rules, the combat-damage section, `SplashList` | splash list (claim 8) | as S-0303's Import mapping | as there |

The loop count, follow-up type and the other playback keys are S-0109's. Backing of the defaults:
`measurable` for the horizontal limit and the lower vertical limit (BA1 run with those keys
absent: F1's ranges, BA1 part c), and for the trailer separation's fault (BA12 part c). The
elasticity default is `unmeasured`: a landing's kind under claim 7 does not fix its exact value.
The damage, radius and warhead defaults are `unmeasured`: with any of them absent a landing deals
no hit that a run can see, so no run tells the stated default from another value that also deals
nothing. Confidence: high for the read and unread keys. Medium for the horizontal and lower
vertical defaults until BA1 part c runs, and for the `unmeasured` defaults.

Three further importer warnings come with departures. A type whose first horizontal speed can
reach a magnitude of 256 or more by claim 7's bound *H*, from its horizontal speed limit, gets
one (claim 7's move of more than one cell). So does a type whose resource spawn overlay type has
fewer than three entries after it in the overlay list (claim 10's place past the list). Each loop
of animation links gets one (claim 15).

## Measurement scenarios

All runs use stock data with test types added by a temporary test art file, no gameplay
changes and the measurement harness at 800 × 600. Each run takes the seed the game chose
(S-0202), recorded with the run; "20 seeds" means 20 runs, each with its own recorded seed, and the
random-state recorder supplies the draws that the expectations use. The recorder reads the
scenario stream's state between ticks only (S-0202's measurement scenarios), and S-0202's
recurrence gives from it the raw value at every later draw position. A creation's *logged raw
values* are the values at its draw positions in the tick that makes it: the tick's first
positions where no other draw comes before it in that tick (as in a trigger-played run on a map
where the ticks before show no advance), and otherwise the positions found from its observed
first velocity as BA11 finds them. "Dying test vehicle" means a
vehicle type whose debris list (S-0312 F1) names one test type, killed by a scripted hit, so that
one debris animation is made at a known point. "Played at a waypoint" means made by a trigger
action of the test map that plays the test animation at a waypoint marker (a trigger-played
animation, S-0109 Terms); its creation point is the waypoint's position on the ground there.

**BA1. First velocity (claims 2, 14; F1).** *Setup:* a named test bouncing type with the bouncer
flag, a fixed *M* and *Z*ₘᵢₙ, elasticity 0, looping for ever, on flat open ground. *Procedure:* 20
seeds; the random-state recorder gives the creation's logged raw values (above); the harness
logs the position in play every tick. *Observable:* the first two turns'
displacements. *Expected:* displacement of turn 1 = (*v*ₓ, *v*ᵧ, *v*_z − 1.4) from F1 with the
logged raw values, from the creation point lifted by 10. *Falsifier:* any turn-1 displacement off
by a whole lepton, or a different raw value feeding an axis. *Part b:* *Z*ₘᵢₙ = 20 with the upper
vertical key set to 100: vertical speeds 20 to 34 only (fixed upper limit); falsified by any
vertical speed of 35 or more. *Part c:* the horizontal and lower vertical speed keys absent (the
elasticity key stays set to 0). *Expected:* horizontal speeds from −15 to 14
by F1 with *M* = 15 from the logged raw values, and a vertical speed of 3.5 in every seed (*W*_z
= 1). *Falsifier:* a horizontal speed outside that range or off F1, or any other vertical speed.

**BA2. Ground landing (claims 4 to 8).** *Setup:* as BA1, with an expiry animation, a damage
amount, a warhead and radius, and test infantry spread around the expected landing cells.
*Procedure:* 20 seeds; log per tick the position in play, the image frame, the animations made
and the health of each infantry. *Observable:* the landing tick, the landing point, which
animations appear where, the damage. *Expected:* *(a)* the stage loops in flight; the animation
leaves play in its landing turn, with no second flight. *(b)* With elasticity 0: no bounce
animation, no hit on infantry of the landing cell apart from the area damage. *(c)* With
elasticity 1.0 and a bounce animation named: the bounce animation at the previous position in
play, a hit on each infantry of the landing cell within the radius before the area damage, then
the expiry animation and the area damage at the landing point. *(d)* No expiry animation named,
elasticity 0 and no bounce animation: no area damage, and no scorch mark or crater
in the landing cell that was not there in the tick before the landing (with nothing else made at
the landing, any such new mark would be the landing's own; one left by the dying vehicle's
explosion is already there). *(e)* The same type played by a trigger action: the same damage.
*(f) No house.* The dying test vehicle belongs to a computer-controlled house C with no base,
given no orders, and the infantry of the landing cell to a house E hostile to C, one of them
with health below the damage amount. Run 1 uses elasticity 0 (only the expiry area damage hits),
and run 2 elasticity 1.0 with a bounce animation (the landing hits come first). *Record:* each
house's count of units destroyed on the score screen (the kill tally of S-0414 claim 21), read
in the tick before the infantry's death and in the tick after it. Short game is off, so E and C
stay undefeated, and every object of C stands beyond the sight and weapon range of E's surviving
infantry. A hit with C as its attacking house and no attacker would raise C's count (S-0302
claim 14), as S-0414 claim 21 credits such a hit to its attacking house. A separate run first
checks that crediting, on the same map with the same two houses: an infantry
object of E is destroyed by a hit with C as attacking house and no attacker, for example the gas
particle damage of a particle system owned by C (S-0302 claim 1, S-0613 claim 13). *Expected of
the check:* C's count rises by one. If it does not, part f cannot tell a hit with a house from
one without, and its house result is inconclusive. A source object (an attacking object) would
raise its own house's count, so run 1 also judges no source object, read together with the check
as for the house (as S-0317 reads its pieces' expiry damage).
*Expected:* in both runs the infantry dies, no house's count of units destroyed changes
across its death, and E counts one loss; no object gains experience.
*(g) Warhead effects.* As run 1 of part f, once with the expiry animation and once without, the
warhead naming a particle system, an animation list of one test type other
than the expiry animation and a positive maximum debris count. *Expected:* with the expiry
animation, the particle system at the landing point, and no animation of the warhead's list and
no warhead debris; without it, no particle system.
*(h) Game-wide gravity.* 20 runs of part a's type with the rules' game-wide gravity value
changed (a temporary test rules file), each logging its creation draws. *Expected:* in every run
the displacement of each turn is the one that F1 and F3 give from that run's logged raw values
with *g* = 1.4, the same rule part a checks.
*Falsifier:* a second flight, damage without an expiry animation, or a bounce animation with
elasticity 0; in part d, a scorch mark or crater in the landing cell; in part h, any displacement that differs from F1 and F3 with *g* = 1.4; in part g, no particle system with the expiry animation, one without it, an
animation of the warhead's list, or warhead debris; in part f, once its check has shown a rise, a rise in C's or any other house's count of units destroyed at the
infantry's death, or experience for any object.
*(i) Deep landings (claim 6, `quirks.bouncing_anim_deep_landing`).* A falling test type with a
horizontal speed limit of 0.5, a lower vertical limit of −150 (so *W*_z = 154 and the first
vertical speed lies from −150 to 3 by F1) and an expiry animation named, played at a waypoint on
flat open ground (so the creation point is on the ground; part i observes only where the expiry
animation stands, not damage), 20 seeds. The first turn's move ends at 10 + *v*_z − 1.4 leptons
from the ground height, so it ends 100 leptons or more below it for a first vertical speed of −109 or less, about a
quarter of the seeds.
*Record:* the logged raw values, the height of the first move's end that F1 and F3 give, and the
height of the expiry animation. *Expected:* where that end is below the ground height and less
than 100 leptons below it (a first vertical speed from −108 to −9), the expiry animation stands
at the ground height. Where it is 100 or more below, the expiry animation stands at that end's
height, below the ground. Seeds whose first move ends at or above the ground height (a first
vertical speed of −8 or more) make no landing in the first turn and are not judged. *Falsifier:* an expiry animation put on the ground after an end 100 or more below it, or
one left below the ground after an end below the ground and less than 100 below.
*(j) The hit (claim 7).* The landing hits alone: elasticity 1.0, no expiry animation (so no area
damage, claim 8), a horizontal speed limit of 0.5 and the lower vertical limit key absent, so that
the animation lands within a few leptons of its creation point. It is played by a trigger action
at a waypoint near the centre of a flat open cell (a trigger-played animation, S-0109 Terms),
which deals the landing hits like any other (claim 8, part e), so no dying object stands in the
landing cell. The harness records the landing point and each hit
object's position. *(j1) Distance:* a test warhead whose spread makes S-0302's falloff (S-0302
F3) depend on the distance, and one infantry object per run at a recorded displacement from the
landing point, at several displacements with sums up to the damage radius, and in further runs
at displacements within the landing cell whose sum |Δ*x*| + |Δ*y*| exceeds the damage radius by
1 to 20 leptons (the radius chosen small enough that such places lie in the cell, and the
warhead's spread wide enough that S-0302's falloff would still give a loss there). *Expected:*
each health loss within the radius is S-0302's for the distance *D* of S-0317 F4; an object
whose sum exceeds the radius loses no health. *Falsifier:* a loss that matches the unlifted sum
or the straight-line distance instead, where these give a different loss; any loss for an object
whose sum exceeds the radius. *(j2)
Defences:* a test warhead whose versus value against the hit object's armour is 50 %.
*Expected:* each loss is S-0302's defended amount for that versus value at the object's distance
*D* (S-0302 claims 5 and 6). *Falsifier:* a loss of the full damage amount, which a hit that
ignores defences would give. *(j3) No-escape marker:* a test rules file that sets the game-wide
crew escape chance to always succeed, and a crewed test vehicle with no passenger capacity and
health below the damage amount in the landing cell within the radius, with room around it for
the crew. *Expected:* the vehicle is destroyed and leaves a crew (S-0312 claim 13). *Falsifier:*
no crew, which the no-escape marker would give. Part j's arc, which BA3 part b shares, is the same
in every run, and S-0317 F6 gives it a measure of at least 3 in every flying turn, so no stopping
landing in flight is expected. A stop in flight (claim 7) would count against F6's form, not
against the hits, and the hits would then go unjudged.
*(k) Resources affected (claim 8).* Part j's motion with elasticity 0 and an expiry animation
named whose type has no resource chain reaction (so its start event, S-0109, leaves the
resource alone), landing in a cell that holds a test resource overlay type with the chain-reaction flag (as
S-0303 scenario A10 variant d builds it), the warhead having the resource flag. *Record:* the
cell's resource units before and after the end. *Expected:* the cell loses the resource units
S-0303 claim 17 gives for the area damage there. *Falsifier:* no loss, which a producer that
does not allow resource reduction would give. Parts j and k, and BA3 part b, use trigger-played
animations, so they are read only once part e has shown that such an animation deals the same
damage; if it does not, they are inconclusive.
*(m) Stop decision at a contact (claim 7).* Part j's motion and setup with a bounce animation and
an expiry animation named, a damage amount of 0, and two runs: elasticity 0.4 and elasticity 0.5.
In every run the animation lands by a ground contact in its seventh turn, whose move is
(−0.5, −0.5, about −6.3), with the height set to the ground height. *Expected:* with elasticity
0.4 the rebounded velocity is about (−0.2, −0.2, 2.52), so *S* = trunc(*ρ*(0 + 0 + 2²)) = 2: a
stopping landing, the expiry animation and no bounce animation. With elasticity 0.5 it is about
(−0.25, −0.25, 3.15), so *S* = trunc(*ρ*(3²)) = 3: an impact landing, with the bounce animation.
In the sixth turn S-0317 F6 gives a height term of trunc(1.4 × 1 − 4.9) = −3 and *S* = 3, so
neither run stops in flight. *Falsifier:* a bounce animation at elasticity 0.4, none at 0.5, or a
landing in another turn.
*(n) Hits after a destroyed vehicle (claim 7).* Part j's setup played at a waypoint in a building
cell that a test vehicle can stand in, such as a repair depot's pad, with the vehicle standing
there within the damage radius of the landing point. The test vehicle's type throws no debris,
has no death weapon, is not exploding and names no destruction animation, so that its death
effects do not touch the building (S-0312 claims 4 to 9), and part j's type has no expiry
animation, so that its end deals no area damage (claim 8). The building is listed after the vehicle
(S-0209 claim 13). Because the building makes the first move a building contact (claim 6), the
type has elasticity 1.5: the first turn's rebounded velocity, about (−0.75, −0.75, −3.15), then
gives *S* = 3 and an impact landing. The warhead has a versus value of 100 % against both the
vehicle's and the building's armour. The type's damage amount and the warhead's spread are
chosen so that the amount after S-0302's falloff for the vehicle's distance *D* (S-0317 F4),
floored at 0 (S-0302 claim 6), exceeds the vehicle's health. The building lies within the radius
too, at a distance *D* at which the same falloff still leaves a loss of at least 1, so a hit on it
would show. The harness records the landing point, the vehicle's position and health, and the
building's health. *Expected:* the vehicle is destroyed, and the building's health does not
change. *Falsifier:* the vehicle is destroyed and the building loses health, which a hit on every
object listed at the start would give (`quirks.bouncing_anim_impact_hits_walk` off). The run is
void if the vehicle survives its hit (then claim 7 lets the hits go on to the building), and
inconclusive if the vehicle crashes or sinks instead of leaving play at once (S-0312 claim 15).

**BA3. Water (claims 7, 8).** *Setup:* the BA2 type thrown over a water cell (a dying test vehicle
on a bridge end or a shore). A temporary test rules file sets the splash list to three distinct
test animation types, first, middle and last. *Expected:* *(a)* wake at the landing point and the
first splash type 3 leptons higher, no expiry animation, no area damage, no spawns. *(b)* With
elasticity 1.0, a bounce animation named, a horizontal speed limit of 0.5 and the lower vertical
limit key absent (as BA2 part j), played at a waypoint near the centre of a water cell that holds
a test naval vessel within the damage radius of the landing point (recorded). On water the end
deals no area damage (claim 8), so any loss of the vessel's health comes from the landing hits.
*Expected:* the bounce animation, then a loss of the vessel's health by the landing hit, then the
wake and the first splash type. *(c)* A meteor test type with a horizontal speed limit of 0.5 and
a negative lower vertical limit, played at a waypoint in the middle of open water at least two
cells from any shore, so that its short path lands on water: only the last splash
type, 3 leptons higher, and no wake. *Falsifier:* expiry effects on water, a splash height other
than landing height + 3, the middle or last type from a falling animation, or the first or middle
type from a meteor; in part b, no bounce animation or no loss of the vessel's health
(`quirks.bouncing_anim_water_impact_hits` off would give neither).

**BA4. Bridge decks (claims 6, 7, 8).** *Setup:* a dying test vehicle on a bridge over water, and one
under a bridge over land. For the rising run the test type has a lower vertical limit of 40 and a
horizontal speed limit of 0.5, so that it rises well past the deck height (claim 2's range for
a negative span) while it stays in the bridge's cell. *Expected:* an animation falling onto the
deck lands at deck height and makes its expiry animation and area damage there, even over water.
An animation rising through the deck from below lands 20 leptons under it. *Low run under the
bridge:* the BA2 type (whose rise stays far below the deck height) thrown under the bridge over
land: it lands at the cell's ground height and makes the first row's effects there, as on any
cell. With elasticity 0 and a
bounce animation named, a landing on the
deck makes no bounce animation (a stopping landing, claim 7), while a landing under the deck makes
it. *Low bridge:* a dying test vehicle beside a low bridge over water, its debris thrown over
the low-bridge cell. The test low bridge's overlay types have the own-land-type flag and the
land type *road*, so that S-0211 claim 9 gives its cells that land type. *Expected:* no landing
at deck height there; the animation lands at the cell's ground height, and the end follows the
bridge overlay's land type, which is not water, so the first row's effects appear and no wake or
splash. *Falsifier:* an animation passing through
the deck, or a bounce animation at an elasticity-0 deck landing; in the low run under the
bridge, a landing anywhere but the ground height; for the low bridge, a landing
at deck height or a wake or splash on the low-bridge cell.

**BA5. Building cells and cliffs (claim 6).** *(a) Building cell.* *Setup:* an ordinary test building type (no wall or
fence section, and not of claim 6's two excluded kinds) with a footprint of at least 3 × 3 cells on flat ground, and a waypoint in the centre cell of its
footprint. A meteor test type with a horizontal speed limit of 0.5, a lower vertical limit of
−8.6, elasticity 0, a damage amount of 0, an expiry animation named and looping for ever is
played at that waypoint, 20 seeds. By F2 both horizontal speeds are then 0.5 in every seed (each
remainder is 0, and the sign change applies), the start position lies within 35 leptons of the
waypoint horizontally and *h*₀ = trunc(8.6 *T*) leptons (438 to 602) above the ground, and each
move is (0.5, 0.5, about −10) (claim 3), so every move ends inside the footprint, first at 150
leptons or more above the ground. *Record:* the logged raw values (for *T*), the per-tick position
in play, and the tick and point at which the expiry animation appears. *Expected:* from *T*, F2
and F3 give each end's height; the landing comes in the first turn *k*ᵦ whose move ends less than
150 leptons above the ground (from 140 to 150 above it), and the expiry animation appears at
that turn's position in play with its height set to the ground height. Without the building
contact the animation would fly on down to the ground and land about 14 or 15 turns later,
within a turn of its first end below the ground height (a stop just above the ground, claim 7,
is possible there); that ground-landing turn and point are computed for each seed and compared.
Since the ends step by about 10 leptons and *h*₀ differs from seed to seed, the landing turns of
the 20 seeds bound the band's top to within a few leptons of 150. A seed with an end within a
hundredth of a lepton of 150 above the ground is not judged. *Falsifier:* a landing in another
turn than *k*ᵦ, in particular at the ground-landing prediction (which a move through the building
cell with no contact would give) or in a turn whose move ends 150 or more above the ground; or an
expiry animation at any height but the ground height, such as the height of the move's end or the
building's roof.
*(a′) Kinds of building and overlay.* Part a's meteor type and procedure, repeated with the
waypoint at the centre of a cell holding, in turn:
- each of the three wall overlays, with no building,
- a gate, closed and open,
- an active laser-fence section and an inactive one,
- a vehicle-like building (S-0214 Terms), a test type if no stock type is one,
- where the run can time it, a building still under construction.

The meteor drifts less than 128 leptons from the waypoint, so a one-cell building or overlay
suffices. *Expected:* a building contact as in part a for the wall overlays, the gate in both
states, the active section and the building under construction; for
the inactive section and the vehicle-like building, no building contact, and the landing at the
ground-landing prediction of part a. *Falsifier:* the other result for any of them. No spec gives
a route to an active laser-fence section, so that cell is set up only where a run happens to
hold one; without it, the active section stays `unmeasured` (claim 6).
*(b) Cliff.* *Setup:* a long straight cliff whose cells step directly from flat low ground to flat
high ground 2 height levels higher, with no slope cell between, running along the second axis,
the low ground lying toward lower first-axis values and stretching at least three cells from
the cliff, and the cliff and both grounds reaching at least three cells either side of the
waypoint along it. A meteor test type with a horizontal speed limit of 8, a lower vertical limit of −1,
elasticity 0, a bounce animation and an expiry animation named, looping for ever, is played at a
waypoint at the centre of a high cell next to the cliff edge, 20 seeds. Each move is then
(*v*ₓ, *v*ᵧ, −2.4) (claim 3), and the start position (F2) lies *T* leptons above the high ground.
*Record:* the logged raw values, the per-tick position in play, and where the expiry animation
appears. *Qualifying seeds:* those whose speed *v*ₓ along the first axis is positive and whose start
position lies over the low ground, and for which F2 and F3 put the flight position, before the
move that enters the high cell, below the high ground by more than 1 and less than 96.6 leptons.
That move is then a ground landing less than 100 leptons below the high ground, set to it, with
*u*_z = −1 and *z*₁ > *z*₀, so it is undone. Most seeds with *v*ₓ from 5 to 8 qualify, and some with
*v*ₓ = 4: about 45 % of seeds. *Expected:* for each qualifying seed, the expiry animation appears
at the flight position before that move, truncated, in the air over the low cell. The
qualifying bounds allow about 111 to 207 leptons above the low ground (more than 1 and less than
96.6 leptons below the high ground, which lies 208 above the low ground); over all raw values
and travel times the qualifying seeds lie from about 151 to 206 above it. The bounce animation
appears at the same position, since the landing is an impact landing: the height above the low
ground gives *S* above 150 whatever the turned velocity (claim 7). *Falsifier:* for a qualifying
seed, an expiry animation in the high cell at its ground height, or anywhere but that position, or
no bounce animation there. Seeds that do not qualify are not judged.
*(c) Vertical speed 0.* As part b, but with a lower vertical limit of 0. Each move is then
(*v*ₓ, *v*ᵧ, −1.4), *u*_z is exactly 0 in every turn (F3), and the start position lies at the
waypoint's height, the high ground's (F2). *Qualifying seeds:* those whose *v*ₓ is positive and
whose start position lies over the low ground at least one move short of the cliff edge, so that
the move entering the high cell is not the first. About 60 % of seeds qualify, most of those
with *v*ₓ from 3 to 8 and some with *v*ₓ = 2. That move ends 1.4(*k* + 1) leptons below the high
ground after *k* ≥ 1 earlier moves, and is a ground landing set to the high ground. Its lift
above the height before the move is 1.4*k*, more than 1, so the move is undone. *Expected:* for
each qualifying seed, the expiry animation and the bounce animation (an impact landing, as in
part b) appear at the flight position before that move, truncated, in the air over the low cell,
1.4*k* leptons below the high ground. *Falsifier:* for a qualifying seed, the expiry animation in
the high cell at its ground height with no bounce animation, which a test that undoes no move
with *u*_z = 0 would give (a stopping landing there, with elasticity 0). Both of claim 6's
conditions undo these moves, so this part does not tell which of them *u*_z = 0 takes.
*(d) Sloped cells.* A meteor test type with a horizontal speed limit of 0.5, a lower vertical
limit of −4.9, elasticity 0.5, a bounce animation and an expiry animation named, a damage amount
of 0, looping for ever, played at a waypoint at the centre of a cell of each slope shape 1 to 8
and of a flat cell, 20 seeds each. By F2 both horizontal speeds are 0.5, and each move is
(0.5, 0.5, about −6.3): the meteor drifts less than 40 leptons and stays in its cell. *Record:*
the logged raw values, the per-tick position in play, and whether the bounce animation appears.
*Qualifying seeds:* those for which F2, F3, S-0106's surface height at each truncated point and
S-0317 F6 give no stop in flight. Their landing is then a ground contact with the move (0.5, 0.5,
about −6.3), set to the surface. About a third of the seeds qualify. In the others a turn whose
height above the surface truncates to 3 to 6 leptons stops in flight, as F6 predicts. *Expected:*
for each qualifying seed, on every sloped shape F6 gives a rebounded velocity whose truncated
components give *S* = 2: a stopping landing, with the expiry animation and no bounce animation. On
the flat cell *S* = trunc(*ρ*(3²)) = 3: an impact landing,
with the bounce animation. *Falsifier:* a bounce animation on a sloped cell, which the flat rule
would give, or none on the flat cell. The part tells the sloped rebound from the flat rule; it
does not tell one tilted plane from another (claim 7).
*(e) Height before the move.* Part b's cliff. A falling test type with a horizontal speed limit
of 255, a lower vertical limit of 197.5, elasticity 0, an expiry animation and a bounce animation
named, is played at a waypoint at the centre of a low cell next to the cliff edge, over a sweep
of seeds with the random-state recorder. By F1 each vertical speed is a whole number plus 0.5,
from 197.5 to 389.5, and each horizontal speed is a whole number from −255 to 254.
*Qualifying seeds:* those whose logged raw values give, by F1, *v*ₓ above 128 and |*v*ᵧ| below
128, so that the first move ends in the high cell next to the waypoint's, and a vertical speed of
197.5 or 198.5. About 1 creation in 800 qualifies, and the sweep runs until at least 5 do. The
first move then ends 0.9 or 1.9 leptons below the high ground, 208 leptons above the low ground,
a ground landing set to the high ground. Measured from the flight position before the move,
10 leptons above the low ground, that lifts the animation by less than *u*_z + 1, so the move is
not undone. Measured from the position in play, at the low ground, it would lift it by more, and
the move would be undone. *Record:* the logged raw values, the per-tick position in play, and
where the expiry animation and any bounce animation appear. *Expected:* for each qualifying seed,
a stopping landing on the high cell: the expiry animation at the high ground's height in the
first turn, and no bounce animation. *Falsified by:* for a qualifying seed, the expiry animation
over the low cell, about 10 leptons above the low ground, which a *z*₀ taken from the position in
play would give. Seeds that do not qualify are not judged.

**BA6. Meteor (claims 1, 3, 4).** *Setup:* a named test meteor type with a horizontal speed
limit *M* of 8 and a lower vertical limit *Z*ₘᵢₙ of −8, played at a waypoint on flat open ground,
elasticity 0. *Procedure:* 20 seeds,
log the raw values, the start position and the per-tick position. *Expected:* *(a)* start =
F2 from the logged raw values; a straight path whose move in every turn is
(*v*ₓ, *v*ᵧ, *Z*ₘᵢₙ − 1.4) to within a lepton; landing in the first turn *k* whose move takes it below
the ground height, by turn *T* and short of the creation point by (*T* − *k*)(*v*ₓ, *v*ᵧ)
horizontally; *(b)* the two horizontal speeds never sum to less than 0; *(c)* a type with both
flags moves as a meteor. (On flat ground a meteor's landing move ends less than 100 leptons
below the ground height, so deep landings are tested with a falling type, BA2 part i.) *Falsifier:* a curved path,
a vertical move of *Z*ₘᵢₙ per turn, or a landing after turn *T*. A stopping landing in flight
before the expected landing (claim 7) is an allowed alternative, judged by claim 7 from the
logged velocity and height, not a falsifier; with *M* = 8 it needs both horizontal speeds to be
small, so few seeds can meet it.

**BA7. Spawns (claim 9).** *Setup:* *(a)* the BA2 type with a spawn type (a second test bouncing
type) and a spawn count, falling animation; *(b)* the BA6 meteor type with the same spawn type and
count, played at a waypoint on flat open ground. 20 seeds each. *Expected:* in both parts,
R(0, *c*) + R(0, *c*) spawned animations at the landing point, matching the recorded draws, each
flying by F1. *Falsifier:* no spawns from either kind, or a count not matching the draws.

**BA8. Ends without a landing (claim 11).** *Setup:* *(a)* a meteor test type with a horizontal
speed limit of 0.5 (so both horizontal speeds are 0.5 after the sign change) and a lower vertical
limit of 1.4, made by a dying test vehicle on a high-bridge deck over water well inside the map,
so that its creation point lies at least the deck height above the water (S-0312 claim 4) and
its start position, below the deck, is far above the water, loop count 3,
and *(a′)* the same with a lower vertical limit of 0 and looping for ever, so that it lands
before its passes end; *(b)* a falling test type with loop count 1, a short stage and a follow-up
type with neither flag; *(c)* the same with a follow-up type that has the meteor flag; *(d)* a
test type with neither flag, loop count 1 and a follow-up type with the bouncer flag, made at a
waypoint. *Expected:* *(a)* it keeps the height of its
start position (about 71 to 98 leptons below the creation height), drifts half a lepton per turn
on each horizontal axis, and leaves play at the end of its third pass with no expiry animation or
damage; *(a′)* it sinks 1.4 leptons in every turn and lands: by S-0317 F6's form with a stopping landing
in flight a few leptons above the deck (claim 7), while a contact landing on the deck is an
allowed alternative, judged by claim 7, not a falsifier; *(b)* it switches type in flight,
keeps falling and lands with the follow-up type's expiry animation; *(c)* after the switch its
vertical speed stops changing, and each move is its velocity at the switch less 1.4 vertically. In
*(d)* it never moves, before or after the switch. *Falsifier:* landing effects at the pass end in
*(a)*, a change of height in *(a)*, no descent in *(a′)*, the follow-up stopping in mid-air in
*(b)*, a vertical speed that keeps falling by 1.4 per turn after the switch in *(c)*, or any move
in *(d)*.
*(e) First update and loop delay.* Part a's type with loop count 2, a random loop delay pair of
(10, 10) (S-0109 claim 11), so that a loop delay of 10 updates follows its first pass, and a
trailer test animation with separation 1. *Record:* the position in play and the trailers made in
every tick from creation. *Expected:* the flight position moves half a lepton per horizontal
axis in every turn, the first update and every update of the loop delay included, so the position
in play advances one lepton on each horizontal axis every second turn without a pause; a trailer
is made in each of those turns. *Falsifier:* a position in play that stays fixed for three or more
turns in a row within the first update and the loop delay, or a turn of either without a
trailer.

**BA9. Trailer (claim 12).** *Setup:* the BA2 type with a trailer test animation and separation 2,
a second run with separation 1 and a third with separation −2; a fourth run with a test animation that has neither flag, a
trailer and separation 2, played at a waypoint, looping for ever. *Expected:* trailers on even
tick numbers (every tick with 1, even tick numbers with −2, as with 2) at the position in play after each move; none in the landing
turn; the trailer's first image frame timing as for a projectile's trailer. In the fourth run,
trailers on even tick numbers only, every one at the waypoint's position, for as long as the
animation plays. *Falsifier:* trailers on odd ticks with separation 2 or −2, at the position before
the move, or in the landing turn; in the fourth run, no trailer, a trailer on an odd tick number, or a missing trailer on
an even one.

**BA10. Resource (claim 10).** *Setup:* *(a)* a falling test type with the resource flag, a spawn
overlay type and radius 1, landing on open germinable ground; *(b)* the BA6 meteor type with the
same resource fields, played at a waypoint with open germinable ground for at least three cells
around it. *Expected:* in both parts, resource on exactly the three
neighbours in step directions 7, 0 and 1 (S-0204) of the landing cell, received in that order,
with the recorded R(0, 3) and R(0, 2) per cell and no draws for the six further pairs that offer
them again. *Falsifier:* resource on the landing cell, on any of the other five neighbours or on
a cell two away, or another order of the draws.
*(c) End of the overlay list.* Part b with a temporary test rules file that orders the overlay
list so that the spawn overlay type is *(c1)* its last entry and *(c2)* its second-last entry, the
entries after it being test resource overlay types; and *(c3)* a list lengthened with test
resource overlay types to more than 258 entries, with the spawn overlay type at place 254, so
that the four places it can give are 254 to 257. 20 seeds each, with the random-state recorder.
*Record:* each offered cell's overlay type and stage, the draws of the landing tick, and whether
the game goes on. *Expected:* in c1 and c2, each cell whose overlay draw names a place within the
list receives the type at that place, at the drawn stage. A draw that names a place past the
list is recorded with its result and judges nothing, since the original's result there is
undefined and may be a stop with a program error; cells after it in that landing are not judged.
In c3 every offered cell receives the type at place 254 plus its draw, at the drawn stage,
places 255 to 257 included, each an ordinary overlay of its type. *Falsifier:* within the list, a
cell with another type, no overlay or another stage; in c3, no overlay or another type for a
place of 255 or more.
*(d) Cell with a terrain object.* Part b with a tree standing on the neighbour in step direction 0,
a cell that S-0401 claim 10 counts as germinable. As in BA11 part b, the spawn overlay type and the
overlay types after it have no cell animation, and the cells around hold no tile animation, so that
the placements take no draws (S-0202 claim 10). The part tests this offer's placement only (claim
10), not S-0401's own new resource. With radius 1 each of the three neighbours is offered by three
pairs (F5). *Record:* as part c. *Expected:* no overlay from the offer under the tree, and two
draws, an overlay draw and a stage draw, at each of the three offers of that cell, so ten draws in
all for the three neighbours, all of them the offer's overlay and stage draws, in F5's order: two
for direction 7, six for direction 0, two for direction 1. *Falsifier:* resource under the tree; two
draws in all for that cell, which a cell that stops being germinable after its first offer would
give; or no draws for it.

**BA11. Draw order (claim 13).** *Setup:* every test animation is played at a waypoint by a
trigger action (so no dying object's draws mix in), on a map with no object other than those
named. *(a)* The BA2 part c type (elasticity 1.0, hits) with BA2 part j's motion (a horizontal
speed limit of 0.5, the lower vertical limit key absent), so that it lands within a few leptons
of the waypoint, near the centre of a flat open cell that holds test infantry within the damage
radius; it names a bounce animation, an expiry animation and the BA7 spawn count. *(b)* The same
motion with the BA10 part a resource fields and the same spawn type and count, on open
germinable ground; the spawn overlay type and the overlay types after it have no cell animation,
and the cells around hold no tile animation, so that their placements take no draws (S-0202
claim 10). *(b′)* As part b, but the spawn overlay type and the three overlay types after it
each name a cell animation, a test bouncing type as below, so that each placement takes that
animation's creation draws. Part b′ runs with settings under which S-0401's growth update runs,
and with one ordinary resource cell of the same tiberium, in its growth list, a few cells from the
landing. The harness logs the stage of the new cells and of that cell for long enough for the
ordinary cell to grow several stages, and short enough that S-0401's F7 rebuilds no list in that
time. *(c)* BA3 part b's motion at a waypoint in open water, elasticity 0 and no bounce
animation named, so that the landing is a stopping landing (claim 7) with no bounce animation and
no hits, with a temporary test rules file that names one test type as the game-wide wake
animation and another as the only splash entry. Every animation made at a landing (bounce,
expiry, spawn, wake and splash types) is a distinct test bouncing type with a horizontal speed
limit of 100 and a lower vertical limit of 20, so that its first velocity (F1) takes one of
about 200 × 200 × 15 values, and with no expiry animation, spawns or resource of its own. Every
test type, flying or made, has no random rate pair, so that its creation takes no stage-delay
draw (S-0109 claim 3) and takes only the six draws of claim 2. Every flying test type has no
random loop delay pair, no follow-up type and no trailer, so that its stage update takes no draws
in flight (S-0109 claims 11 and 15). 20 seeds per part.
*Procedure:* the random-state recorder reads the scenario stream's state between every two ticks
(S-0202's measurement scenarios), from before the trigger fires until after the landing; the
harness logs every animation's position in play per tick and, in part b, each new resource
cell's overlay type and stage. From a recorded state S-0202's recurrence gives the raw value at
every later draw position, and the next recorded state gives how many draws the tick between
took (its position modulo 250, resolved by matching the state).
*Expected:* *(1) No draws in flight.* In each tick in which the test animation flies, makes no
trailer and does not land, the stream does not advance. This part is judged only where the ticks
before the trigger fires show no advance either; otherwise other objects' draws hide the
animation's and it is inconclusive. Part b′ is not judged by this check, since its growth
updates (S-0401 claim 7) take draws in ticks in which the animation flies. *(2) Order at the
landing.* The first
velocity of each animation made in the landing tick, read from its first two turns'
displacements as BA1 reads them, matches F1 for the three raw values at one draw position of
that tick; a match by chance at another position of the tick is less likely than one in a
thousand.
These positions follow claim 13's order. In part a: the bounce animation's three raw values
and three ranged draws first; then the hits' draws (S-0302), if any; then the expiry animation's
six; then the area damage's draws (S-0303), if any; then two draws that give, by F4 and S-0202's
ranged draw, the number of spawned animations seen; then the spawned animations' six each, one
after another with no draw between them. In part b, after the last spawned animation's draws,
the cells in F5's order, each with a draw that gives by R(0, 3) the overlay type seen in that cell
and the next draw giving by R(0, 2) its stage, with no other draw between two cells; in part b′
the cell animation's six draws come between each cell's overlay draw and its stage draw, and no
further draw comes between them (no growth-list draw, claim 10). *(3) Resource lists, part b′.*
In the logged time the new cells' stages never change, while the ordinary cell grows. In part c
the wake's six draws, then the splash's. *Falsifier:* in a judged flying tick, any advance; in
the landing tick, a made animation whose velocity matches no draw position of the tick, made
animations whose positions come in another order, a spawn count, overlay type or stage that does
not match the draws at the positions claim 13's order gives it, a draw between two resource
cells, or in part b′ a cell animation's draws outside its cell's overlay and stage draws, a
further draw between them (which an offer to the growth list would give,
`quirks.anim_resource_no_growth_entry` off), or a new cell whose stage rises in the logged time.

**BA12. Zero spans (claim 14, claim 12).** *Setup:* test types with *(a)* *M* = 0.3, *(b)*
*Z*ₘᵢₙ = 4.0 and the bouncer flag, *(c)* a trailer and no separation key. *Expected:* the original
stops with a program error at creation in parts a and b, and at the first animation turn in part c.
Our engine's run (departure) shows the departure's results: in parts a and b every animation of
the type has the same first speed on the zero-span axis (claim 14), and in part c a trailer is
made in every turn, as with separation 1; the importer warns for each type.

**BA13. Off the map (claims 6, 11).** *Setup:* a small test map whose legal cells are all at
height level 4, those outside the usable area included, with no low bridge, played in a freshly
started program (or one whose earlier games had no low bridge reaching off their map), so that
the original's stand-in cell stays blank (S-0009 claim 44). The legal cells' ground is 416
leptons high, so a landing on a legal cell can be told from one off the legal map, at height 0. Each run is
played at a waypoint near one edge of the usable area, close enough to the legal map's edge that
some flights leave it.
- *Falling run:* a named test falling type with a horizontal speed limit of 20, a lower vertical
  limit of 40 (so a first vertical speed of 40 to 74, claim 2), elasticity 1, a bounce animation,
  an expiry animation and a spawn type with a spawn count of 1 named, looping for ever, played at
  such a waypoint, 40 seeds. A flight lasts about 60 to 110 turns and can carry it up to about
  eight cells.
- *Resource run:* the same type with the resource flag, a spawn overlay type and radius 1.
- *Meteor run:* a named test meteor type with a horizontal speed limit of 40 and a lower vertical
  limit of −10, otherwise as the falling run, played at such a waypoint, 40 seeds, so that many
  start positions (F2), up to about eleven cells from the waypoint, lie off the legal map.

*Record:* each animation's position each tick with the read-only recorder, which follows objects
off the map too; each animation made, with its position; overlay changes; and the draws, with
the random-state recorder.
*Expected:*
- no animation leaves play for being off the map. Each ends only at its landing, its path from
  the logged raw values by F1 to F3 throughout, off the map included,
- on legal cells outside the usable area, landings at their ground height of 416 leptons, and
  there the end that any other legal cell would give,
- off the legal map, ordinary ground landings at height 0, each an impact landing with its
  bounce animation, then the expiry animation and any spawned animations at the landing point,
  and never a wake or splash. A move undone by the cliff test (claim 6) lands at the position in
  play before the move, at whatever height that is; a stopping landing (claim 7) makes no bounce
  animation; and a deep landing, 100 leptons or more below height 0, stays at the height its move reached
  (claim 6),
- in the resource run, an end off the legal map makes resource, with its draws, only on
  neighbours that are legal, germinable cells (claim 11), and so none when the usable area does
  not reach the legal map's outermost cells,
- in the meteor run, animations placed at start positions off the legal map that move from
  their first turn, fly in by F3 and land as claim 6 says, any landing on entering the map
  tested as at a cliff from level 0, and every seed's six creation draws in the creation tick.

*Falsifier:* an animation removed while flying, a landing at height 0 on a legal cell outside the
usable area (which would show such a cell read as off the map), an ordinary ground landing off
the legal map at another height than 0, a water end off it, resource or resource draws on a cell that is not a legal
germinable neighbour, or a meteor animation with a start off the legal map that never appears,
never moves or takes fewer creation draws.

**BA14. Trailer loops (claim 15).** *Setup:* a named test art layer with test animation types
that have neither flag, each with a few image frames, a loop count that keeps it in play for at
least 10 ticks (parts a and b) and no follow-up type unless stated, each played at a waypoint of
a small test map by a trigger at a known tick. *(a)* Type X names X as its trailer, separation 1.
*(b)* X names Y and Y names X as trailers, separations 2 and 3. *(c)* X names Y as its trailer,
separation 10, and Y names no trailer and has X as its follow-up type. X keeps an animation in
play for at least 70 ticks, whether played or reached by a follow-up switch, with a loop count
below 256 as S-0109 claim 15 needs, and Y switches to X from 1 to 8 ticks after its creation as a
trailer, never in the tick that makes it. A pre-check confirms both lengths: X played alone, and
the run of part c with Y given no follow-up type, recording the ticks from each Y's creation to
its end. X is played on a tick whose number leaves 1 when divided by 10. *Procedure:* one run per
part, in a fresh session each. Record per tick the number of animations of the test types in play
with the read-only recorder, for 60 ticks from the trigger's tick or until the tick count stops
advancing, and whether loading reaches play. Stop a run from outside when its tick count has not
advanced for 60 seconds. Do not wait for resource exhaustion. *Expected:* every part loads and
reaches play. In part a the tick in which X is played does not complete normally. In part b the
ticks before the first tick number divisible by 6 on which an animation of the loop is in play
complete, with counts growing, and that tick does not complete normally. In part c every tick
completes, and the 60 ticks hold six ticks whose number is divisible by 10. In the window no
animation of the loop ends: X outlives the window, and a Y does not end but switches to X before
the next such tick. On each such tick every X in play makes one Y (claim 12), and a Y made on one
such tick is an X by the next, so the k-th such tick doubles the number of X. The count of X and
Y in play is 1 until the first such tick, 2ᵏ from the k-th to the next, and 64 at the window's
end. *Falsifier:* a load refusal or a changed type, a completed tick in part a, a stop in part b
at another tick, or in part c a stop or a count of X and Y in play other than 2ᵏ after the k-th
tick divisible by 10, k from 1 to 6, or other than 1 before the first. Our engine: in part a X
makes one trailer per tick and the trailers make none; in part b and part c the loop's second
generation makes nothing through the loop's links, so in part c the count is k + 1 after the k-th
tick divisible by 10; every tick completes, and the importer warns once per part.

**BA15. Landing loops (claim 15).** *Setup:* a named test art layer with a falling test type X
(bouncer flag, elasticity 0, a horizontal speed limit of 1, looping for ever). *(a)* X names X as
its spawn type with a spawn count of 2 and a lower vertical limit of 20, played at a waypoint on
flat open ground, 10 seeds. *(b)* X names X as its expiry animation, played at a waypoint on a
cell of an ordinary test building, with no lower vertical limit key (the default of the Import
mapping). *(c)* As part a with a spawn count of 3, on that building cell, 10 seeds. *Procedure:*
as BA14, 60 ticks per run. Record per tick the number of animations of X in play and each landing
with its tick, and the draws with the random-state recorder. *Expected:* every part loads and
reaches play. In part a no spawned X lands in its creation tick, each landing makes the number of
X that its two logged spawn-count draws give, and the count goes on from landing to landing,
growing in the seeds whose draws do not end it. In part b about one landing per tick, each in the
turn after its maker's, for the whole run, and every tick completes. In part c landings in the
creation tick of the animations that land, each making the number of X that its logged
spawn-count draws give. Whether a tick of part c fails to complete is recorded per seed as an
observation, set beside that seed's logged draws; it depends on them (claim 15), so it has no
falsifier. *Falsifier:* a load refusal or a changed type, a spawned X landing in its creation
tick in part a, a stop in part b, or in part c no landing in a creation tick in any of the 10
seeds, or a landing that makes another number of X than its logged draws give. Our engine: the X
played at the waypoint makes its spawned or expiry animations, and those make none and take no
spawn-count draws, so every run ends with no X in play after the second generation lands.

## Changes for other specs

- **S-0109 Terms, *bouncing type*:** "the specs of projectiles and debris own both" should name
  S-0319, and add that a bouncing animation that switches to a follow-up type keeps moving even
  when the follow-up type has neither flag (S-0319 claim 11).
- **S-0109:** an animation type's own trailer animation (S-0319 claim 12) applies to every
  animation; S-0109's list of what it does not cover can point there.
- **S-0312 claim 4:** sprite debris animations move and end by S-0319.
- **S-0302 claim 1, its table of sources:** add a bouncing animation's landing hits and expiry
  damage (S-0319 claims 7 and 8): no attacker and no attacking house.
- **S-0303 claim 23:** besides the disc laser and a debris piece's end (S-0317 claim 11), the end
  of a bouncing animation (S-0319 claim 8) is a third such producer: its expiry damage applies
  the area damage of claims 1 to 20 with resources affected and none of the projectile-only
  effects of claims 21 and 22, and, like a piece's end, makes the warhead's light flash. As for a
  piece, it happens only when the type names an expiry animation and the end is not on water or
  is at deck height (`quirks.bouncing_anim_expiry_needs_animation`).
- **S-0302 claim 8, *No-health targets*:** a bouncing animation's landing hits (S-0319 claim 7)
  are a source with no filter: they hit a listed object that has no health left, and the
  no-health rows decide what the hit does. So claim 8 should name them as a known source of such
  hits.
- **S-0312 claim 21, a building awaiting removal:** "it cannot be hit" holds for detonations
  (S-0303 claim 9). A bouncing animation's landing hits (S-0319 claim 7) still reach it, since
  it stays listed in its cells.
- **S-0401 claim 10:** a germinable cell that holds a terrain object (a tree's, for example)
  receives no overlay from a bouncing animation's resource offer (S-0319 claim 10): the offer
  takes its draws and leaves the cell germinable. S-0401 claim 10 can note this for S-0319's
  offers, beside its own rule that trees do not prevent a cell from being germinable. This spec
  does not cover S-0401's own new resource cells (its claims 9 and 11, scenario E8).
- **S-0109 claim 15:** an animation stays looped through a follow-up switch, by S-0319 claim 15,
  and S-0109 claim 16 can point to S-0319 claim 15 for follow-up types that lead back to a type
  whose animations make the first.

## Open questions

Q-1762, Q-1846, Q-1847 and Q-2996 are answered. Q-1762's question 1 leaves the elasticity,
damage, radius and warhead defaults stated but `unmeasured` (Import mapping). Several answers
rest on `unmeasured` claims, each rated where it is stated: the condition that a vertical speed
of 0 takes and the line at −0.0002 (claim 6), the landing point of an undone first move (claim 6)
and the rest of the hits' listing walk (claim 7). Four points are left open:

- Which killing hits take their object out of the cell's ground layer within the hit, and so end
  the remaining impact hits of claim 7. A destroyed vehicle that neither crashes nor sinks does,
  since it leaves play at the moment its damage is applied (S-0312 claim 15, S-0203 claim 6 (a)),
  and a building awaiting removal does not (S-0312 claim 21). Infantry removed at once and
  buildings that leave at once leave play the same way (S-0203 claim 6 (b) and (c)); this spec
  does not establish whether every such leaving unlinks the object before its hit is over. Our
  engine ends the hits after them too (claim 7's design rule). Medium for the vehicle; BA2 part n.
- Whether the original places an animation whose placement point (claim 11) is exactly the point
  (0, 0, 0), off the legal map. It is believed not to, so that such an animation would take its
  creation draws and never take a turn. Low and `unmeasured`: no producer here can be made to
  name exactly that point, and none in stock play is known to. S-0317 claim 21 states that a
  debris piece whose placement point is exactly (0, 0, 0) is not placed; this entry concerns
  sprite animations, which S-0317 does not cover. Our engine's rule there is claim 11's design
  rule, so the entry does not affect an implementation.
- Loops that pass through links of another family (for example a landing whose area damage
  destroys an object whose debris list makes the landing type, or a debris piece whose end shows
  an animation of a type that makes that piece) are not covered by claim 15's rule. Whether they
  can close within one pass is not established.
- Whether a never-ending pass of claim 15 would end once the program can get no room for a
  further animation, and what the game would then do. Low and `unmeasured` (claim 15). Our
  engine's departure keeps every pass of claim 15's loops finite, so the entry does not affect an
  implementation.
