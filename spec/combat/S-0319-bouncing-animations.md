---
id: S-0319
title: Bouncing animations (sprite debris and meteors): motion, landing and end
topic: combat
revision: 9
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-25
measurements: []
requests: [Q-1762]
supersedes: []
---

# S-0319: Bouncing animations (sprite debris and meteors): motion, landing and end

## Summary

S-0109 Terms define a *bouncing type*: an animation type with the bouncer flag or the meteor flag,
whose animation follows its own motion and is ended by its landing. This spec gives that motion
and that landing for sprite animations: the first velocity formed from the creation draws, the
flight under gravity, what counts as landing (ground, high-bridge deck, building cell, cliff), the
effects of the landing and of the end that follows at once, the ends without a landing, and the
draws all of these take. It also gives the trailer animation that any animation type can name,
since moving bouncing animations are its main users. It answers Q-1762 in part: the parts left
open are listed under Open questions.

Debris pieces with voxel models are not animations of this kind: S-0317 owns
them. The two share one flight and contact rule, which this spec states for sprite animations
and which S-0317 states for pieces, apart from the differences claims 6 and 7 name. The main difference: a sprite animation ends at its first
landing, while a piece rebounds and flies on.

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

2. **Creation of a falling animation.** The animation is placed at its creation point. Its flight
   position starts 10 leptons above the creation point, and its first velocity is given by F1 from
   three raw values on the scenario stream, taken in this order: *r*₁ (vertical speed), *r*₂
   (second horizontal axis), *r*₃ (first horizontal axis). Then three ranged draws R(−65535,
   65535) are taken; their values have no effect on the animation. The vertical speed lies from
   *Z*ₘᵢₙ up to *Z*ₘᵢₙ + |*W*_z| − 1 leptons per tick, where
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
   vertical speed is the type's lower vertical limit *Z*ₘᵢₙ. When the first horizontal speed is
   less than minus the second, both change sign, so the meteor's two horizontal speeds never sum
   to less than 0 (F2). The animation is placed at its start position, the
   creation point moved back by *T* times its velocity (F2), and its flight position starts
   there. Three ranged draws R(−65535, 65535) follow, as in claim 2. A meteor animation then flies
   in a straight line (claim 4), but the start position does not put the creation point on that
   line. Each turn's move takes the 1.4 of claim 4 off the vertical speed before the move and adds
   it back only after it, so every move is (*v*ₓ, *v*ᵧ, *Z*ₘᵢₙ − 1.4), up to the rounding of F3.
   After *T* turns the line is over the creation point horizontally and 1.4 *T* leptons (about
   71 to 98) below it. A descending meteor whose creation point is on the ground therefore meets
   the ground by its *T*-th turn, at the first turn whose move takes it below the ground height
   (claim 6), unless a stopping landing of claim 7 comes first. Its landing point lies short of the creation point along its line, by
   (*T* − *k*)(*v*ₓ, *v*ᵧ) horizontally for a landing in turn *k*.
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
     of its bridge overlay (S-0211 claim 9). The cells of a destroyed high bridge are no longer
     marked (S-0211 claim 15) and are tested in the same way.
   - **Ground.** Otherwise, when the new height is below the ground height, the move is a
     landing. The height is set to the ground height when it is less than 100 leptons below it,
     and is otherwise kept below the ground (Edge cases, `quirks.bouncing_anim_deep_landing`).
     "Otherwise" includes a high-bridge cell
     whose deck the move does not cross: there the ground test and the building test below are
     made as on any other cell.
   - **Building cell.** Otherwise, when the new height is at least the ground height and less
     than 150 leptons above it, and the cell of the new position holds a building, the move is a
     landing and the height is set to the ground height. Some buildings do not count, and some
     overlays count as a building does (Open questions).
   - **Cliff.** After any landing of the three kinds above, the move is undone when the cell of
     the new position is at least 2 height levels above the cell of the old position and the
     height *z*₁ after the landing's adjustment above is higher than the move could reach. With
     *u* the vertical speed at the start of the turn (before the 1.4 is taken off) and *z*₀ the
     height of the flight position before the move (not of the position in play, which is the
     flight position truncated toward zero, Terms, and which in a falling animation's first turn
     stands 10 leptons below it, at the creation point, claim 2), that is: for a
     descending animation (*u* < 0), when *z*₁ > *z*₀; for a rising
     one (*u* > 0), when *z*₁ > *z*₀ + *u* + 1. Our engine applies the two conditions as written,
     so an animation whose *u* is exactly 0 meets neither. Where the original's line between them
     lies, for an animation whose *u* is zero or within a small fraction of a lepton per tick of
     zero at the top of its arc, is `unmeasured` (below). This is S-0317 claim 10's cliff test
     for pieces, with one difference: for a sprite animation *z*₀ is the flight position's height,
     where S-0317 claim 10 takes the height of the piece's position before the move. The
     flight position and the velocity then become those before the move, and the velocity is
     turned off the cliff face. The position in play is then set from that restored flight position,
     truncated toward zero (Terms), so the landing point is the position in play before the move,
     except in a falling animation's first turn, where it stands 10 leptons above the creation
     point at which the animation stood before the move. So an animation that
     the landing would lift onto the higher cell by more than its own motion carried it lands at
     the foot of the cliff, while a rising animation that the landing lifts by little enough
     lands on the higher cell.
   These are the *contacts*. A move with no contact is still a landing when its motion measure is
   below 2.5 (claim 7). Every landing ends the animation in the same turn (claims 7 and 8): a
   bouncing animation lands once.
   - Backing: `measurable` (scenario BA2 for the ground, BA4 for high-bridge decks and the low
     bridge, BA5 part a for building
     cells and part b for the cliff test of a descending animation, BA2 part i for the 100-lepton limit,
     BA4's low run under the bridge for the ground test on a high-bridge cell whose deck the move
     does not cross). The cliff test's condition for a
     rising animation, and the line between the two conditions, are `unmeasured`: BA5 part b flies
     a meteor animation that meets the cliff cell while descending, and no scenario here builds one that
     meets it while rising or with a vertical speed within a small fraction of a lepton per tick
     of zero, which F1 and F3 make only by chance. Our engine's line at *u* = 0 is not claimed to
     match the original's within that margin. That *z*₀ is the flight position's height is
     `unmeasured`: the two heights differ by the 10-lepton lift in a falling animation's first
     turn, which no scenario here makes meet a cliff, and otherwise only by the fraction of a
     lepton that the truncation drops, which no scenario here can resolve. For the same reason
     the landing point of an undone first move of a falling animation, 10 leptons above the
     creation point, is `unmeasured`.
   - Confidence: high for ground landings and for ending at the first landing. Medium for the
     high-bridge rules, the low bridge, the 150-lepton building band and the descending cliff
     test, until BA4 and BA5 run, and for the 100-lepton limit until BA2 part i runs. Medium for
     the ground test under a high bridge until BA4 runs, for which height *z*₀ is and for the
     landing point of an undone first move (`unmeasured`). Low for
     the rising cliff condition, for which buildings and overlays count, and for the velocity
     turned off the cliff face (Open questions).

7. **Impact landing or stopping landing.** At a contact (claim 6) the velocity is first
   rebounded as S-0317 claim 10 gives it: on a flat cell the vertical speed is reversed and the
   whole velocity scaled by the type's elasticity *e*, on a sloped cell the component across the
   cell's surface is reversed instead (low confidence there, Open questions), and off a cliff face
   it turns as S-0317 leaves open. After every motion step, with or without a contact, the motion measure *S*
   is then taken from the velocity the step has left and the height above the ground height, as
   S-0317 F6 gives it, before a meteor's addition of 1.4. When *S* < 2.5 the step is a stopping
   landing, with or without a contact. Otherwise a step with a contact is an impact landing, and
   a step without one is no landing.
   - A **stopping landing** has no effect of its own; the end of claim 8 follows.
   - An **impact landing** first makes the type's bounce animation, when it names one, at the
     position in play before the move (the one the previous turn left, or in the first turn the
     creation point or the start position), with no loop delay, repeat factor 1 and no reverse
     request. Then each object listed in the ground layer of the one cell that holds the landing
     point whose horizontal distance |Δ*x*| + |Δ*y*| from the landing point is at most the type's
     damage radius receives a hit of the type's damage amount truncated toward zero, with its
     warhead, exactly as a piece's ground impact of S-0317 claim 7 (no attacker, no attacking
     house, defences not ignored, no no-escape marker, the lifted distance of S-0317 F4). The hits
     form a set, whose order matters only where one hit's result can affect another object of the
     cell before that object's hit, or where more than one hit takes draws from the scenario random
     stream (which draw each hit receives); there they come in the ground layer's listing order
     (S-0209 claim 13). Whether they skip a listed object that has no health left is open, as it is for a
     piece's impact (Open questions). The hits take the ground layer whatever the height of the
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
     radius limit (part j1), the defences and the no-escape marker, BA3 part b for the bounce animation and the hits on
     water, BA11 part a for the order of bounce animation, hits and end). No attacker is
     `unmeasured`: BA2 part f judges house credit, not an attacker object, and no run here judges
     retaliation, as for S-0317 claim 7. The
     listing order, where one hit's result can affect another object or more than one hit takes
     draws, is `unmeasured`: no scenario here gives an object hit at a landing a death that
     affects another object of the cell, or two hit objects that both take draws. The ground
     layer at a landing on a deck is `unmeasured` here: BA4 places no object on the deck or under
     it, and S-0317 V7 part b's run with objects tests the same rule for pieces. A stopping
     landing without a contact is `unmeasured`: BA6, BA2 part j and BA8 part a′ admit a stop in
     flight only as an allowed alternative, and whether and when it comes rests on S-0317 F6's
     form, itself `unmeasured` there, which no scenario here pins to a turn.
   - Confidence: low for a stopping landing without a contact, which rests on S-0317 F6's exact
     form; that a sprite animation then ends in the same turn, where a piece ends in the turn
     after, is medium. Medium for the bounce animation and the hits on water, until BA3 part b
     runs. Medium for the ground layer at a deck landing. High for the elasticity-0 case at the
     ground height, where the velocity and the height are both 0, so *S* = 0 under any form of
     S-0317 F6. Medium for the order of bounce animation, hits and end, as S-0317 claim 7 rates
     it, until BA11 part a runs. Low for the stop decision on a rebounded velocity that is not 0,
     which rests on S-0317 F6's exact form: that form is `unmeasured` there, and no scenario
     here pins such a stop to a turn (Open questions). Medium for the deck case with an
     elasticity of 0, until BA4 runs; low for the under-deck and deep cases, which rest on F6's
     form. Medium for the distance *D*, the radius limit, the defences and the no-escape marker
     until BA2 part j runs, and for no attacker. Medium for no attacking house until BA2 part f and its check of
     the S-0414 claim 21 reading run (Changes for other specs). Low for the listing order.

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
   such effects come only from the animations it makes, by S-0109. The area damage's own effects
   follow their rules: the warhead's particle system when it names one (S-0303 claim 20), owned by
   no house, and an exploding overlay in the centre cell (S-0303 claim 19). The projectile-only
   effects of S-0303 claims 21 and 22 are not made: no explosion animation from the warhead's
   animation list, no warhead debris and no nullify animation, as for S-0317 claim 11. The meteor
   test of the water rows uses the type the animation has at the landing (claim 11). A trigger-played animation (S-0109 Terms) deals the hits of
   claim 7 and the area damage of this claim like any other. With an empty splash list the water
   rows make no splash animation and the importer warns: a **departure** with the same result as
   S-0317 claim 11's, since the original's result is undefined.
   - Backing: `measurable` (scenario BA2 for the first row, BA3 for the water rows, BA4 for the
     deck test in cells that a high bridge marks, BA2 part d for the absence of damage without an expiry animation and for no
     scorch mark or crater of the landing's own, BA2 part g for the warhead's particle system and
     the missing projectile-only effects, BA2 part f for no source object and no house, BA2 part
     k for resources affected, BA9 for no trailer in the landing turn, BA11 for the order of the
     effects)
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
     until BA2 part f and its check of the S-0414 claim 21 reading run, for resources affected
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
   S-0202 claim 5, and the importer warns for it as that claim requires.
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
    - Backing: `measurable` (scenario BA10, part b for a meteor animation; BA11 part b′ for the
      placement's place among the draws)
    - Confidence: medium for the neighbour choice, the draws and the stage set after the
      placement. Low for how the new overlay enters the resource lists of S-0401 (Open
      questions).

11. **Ends without a landing, and follow-up types.** Apart from its owner's removal (S-0109
    claim 22), nothing ends a bouncing animation in flight over the map except its stage (no time
    limit, no distance, no screen edge). Whether leaving the map ends it is open (Open questions).
    When its last pass ends:
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
    when a move meets the ground, a deck or a building cell by claim 6 while it is over the map.
    What happens when its line leaves the map first is open (Open questions). With a lower limit
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

12. **Trailer animation.** Every animation whose type names a trailer animation, bouncing or not,
    makes one in each of its turns that a landing does not end, while it is in play, after its
    motion step and before its stage update. An attached animation whose owner has been removed
    makes none at the update that deletes it (S-0109 claim 22). It does this on ticks
    whose tick number is divisible by the type's trailer separation *s*, or on every tick when
    *s* = 1. The trailer is made at the animation's position in play after the motion step, with
    an initial loop delay of 1 update, repeat factor 1 and no reverse request, the settings
    S-0109 claim 29 gives a projectile's trailer. A negative *s* acts as its magnitude. The
    first update and the updates of a loop delay are such turns too: in them the trailer is made
    on the same tick-number cadence as in any other turn. A separation of 0, the
    absent-key default, makes the original stop with a program error at the first such turn
    (S-0205 claim 3). This is a **departure**: our engine treats a separation of 0 as 1, so a
    trailer is made in every such turn, and the importer warns for a type that names a trailer and
    no separation or a separation of 0.
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
    draws and then each spawned animation's creation draws, then for each cell that receives
    resource, cells in F5's order, the overlay draw R(0, 3), the draws of the overlay's placement
    (S-0202 claim 10: none for the overlay itself; when the overlay type has a cell animation, that
    animation's creation draws; then any tile-animation draws of the cell's recalculation) and
    the stage draw R(0, 2); on water,
    the wake's and then the splash's creation draws. All are on the scenario stream. The light
    flash takes none.
    - Backing: `measurable` (scenario BA11 with the random-state recorder: part a for a dry end
      with an impact landing and spawns, part b for the resource draws, part b′ for the placement draws among them, part c for the
      water end)
    - Confidence: high for the creation draws and the spawn draws. Medium for the full order at
      an end, which follows from claims 7 to 10, until BA11 runs. Low for whether a new resource
      cell also takes a growth-list draw of S-0401 claim 11 (Open questions).

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
    such a type. S-0317 claim 16 gives pieces a different result for the same case (Changes for
    other specs).
    - Backing: `measurable` (scenario BA12 for the original's stop). The departure's result is a
      design rule, not an oracle value.
    - Confidence: high for the horizontal span. Medium for the vertical span until BA12 part b
      runs.

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
offers the neighbour in step direction 0. A cell offered again after it has received resource is
no longer germinable and takes no draws. Under `quirks.anim_resource_spread_neighbours_only` off,
the pair offers the cell (*X* + *i*, *Y* + *j*) instead, in the same order.

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
| water splash lift | 3 | leptons | claim 8 |
| resource overlay choice | R(0, 3) | overlay-list places | claim 10 |
| resource stage | R(0, 2) | stages | claim 10 |

## State and ordering

| State | Event | Result |
|---|---|---|
| Created | creation | placed (falling: creation point, meteor: start position); creation draws (claim 13) |
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
- **Zero spans** (claim 14) and **a trailer separation of 0** (claim 12) are departures.
- **An empty splash list** is a departure (claim 8).

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
absent: F1's ranges, BA1 part c), and for the trailer
separation's fault (BA12 part c). The elasticity default is `unmeasured`: a landing's kind under claim 7 does not fix its exact
value. The damage, radius and warhead defaults are `unmeasured`: with any of them absent a
landing deals no hit that a run can see, so no run tells the stated default from another value
that also deals nothing. Confidence: high for the read and unread keys. Medium for the
horizontal and lower vertical defaults until BA1 part c runs, and for the `unmeasured`
defaults.

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
claim 14), but only if the reading of S-0414 claim 21 that Changes for other specs gives holds
in the original: under claim 21 as it stands, a destruction without an attacking object gives no
house a kill, and C's count would not change whether or not the hit carries a house. So a
separate run first checks that reading, on the same map with the same two houses: an infantry
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
fence section, none of the kinds the Open question on which buildings count concerns) with a footprint of at least 3 × 3 cells on flat ground, and a waypoint in the centre cell of its
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
building's roof. Repeats with other kinds of building, or with an overlay in place of the
building, bear on the Open question of which count and judge nothing here.
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
*u* = −1 and *z*₁ > *z*₀, so it is undone. Most seeds with *v*ₓ from 5 to 7 qualify, and some with
*v*ₓ = 4: about 45 % of seeds. *Expected:* for each qualifying seed, the expiry animation appears
at the flight position before that move, truncated, in the air over the low cell. The
qualifying bounds allow about 111 to 207 leptons above the low ground (more than 1 and less than
96.6 leptons below the high ground, which lies 208 above it); over all raw values and travel
times the qualifying seeds lie from about 151 to 206 above it. Whether the bounce animation also appears there (an impact
landing) or not (a stopping landing) is recorded for the Open question on the velocity turned
off the cliff face and judges nothing. *Falsifier:* for a qualifying seed, an expiry animation in
the high cell at its ground height, or anywhere but that position. Seeds that do not qualify are
not judged.

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
animation's creation draws. *(c)* BA3 part b's motion at a waypoint in open water, elasticity 0 and no bounce
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
animation's and it is inconclusive. *(2) Order at the landing.* The first
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
the cell animation's six draws come between each cell's overlay draw and its stage draw. In part c
the wake's six draws, then the splash's. *Falsifier:* in a judged flying tick, any advance; in
the landing tick, a made animation whose velocity matches no draw position of the tick, made
animations whose positions come in another order, a spawn count, overlay type or stage that does
not match the draws at the positions claim 13's order gives it, a draw between two resource
cells, or in part b′ a cell animation's draws outside its cell's overlay and stage draws.

**BA12. Zero spans (claim 14, claim 12).** *Setup:* test types with *(a)* *M* = 0.3, *(b)*
*Z*ₘᵢₙ = 4.0 and the bouncer flag, *(c)* a trailer and no separation key. *Expected:* the original
stops with a program error at creation in parts a and b, and at the first animation turn in part c.
Our engine's run (departure) shows the departure's results: in parts a and b every animation of
the type has the same first speed on the zero-span axis (claim 14), and in part c a trailer is
made in every turn, as with separation 1; the importer warns for each type.

## Changes for other specs

- **S-0109 Terms, *bouncing type*:** "the specs of projectiles and debris own both" should name
  S-0319, and add that a bouncing animation that switches to a follow-up type keeps moving even
  when the follow-up type has neither flag (S-0319 claim 11).
- **S-0109:** an animation type's own trailer animation (S-0319 claim 12) applies to every
  animation; S-0109's list of what it does not cover can point there.
- **S-0312 claim 4:** sprite debris animations move and end by S-0319.
- **S-0317 claim 16:** its departure for a zero span (the remainder equals the dividend) gives a
  piece a speed of up to the size of a raw value. S-0319 claim 14 takes the remainder as 0 instead,
  a fixed speed. S-0317 can adopt the same result at its next revision.
- **S-0302 claim 1, its table of sources:** add a bouncing animation's landing hits and expiry
  damage (S-0319 claims 7 and 8): no attacker and no attacking house.
- **S-0303 claim 23:** besides the disc laser and a debris piece's end (S-0317 claim 11), the end
  of a bouncing animation (S-0319 claim 8) is a third such producer: its expiry damage applies
  the area damage of claims 1 to 20 with resources affected and none of the projectile-only
  effects of claims 21 and 22, and, like a piece's end, makes the warhead's light flash. As for a
  piece, it happens only when the type names an expiry animation and the end is not on water or
  is at deck height (`quirks.bouncing_anim_expiry_needs_animation`).
- **S-0414 claim 21:** the reading that S-0317's Changes for other specs proposes, "A
  destruction with neither an attacking object nor an attacking house gives no house a kill
  (S-0302 claim 14), but still gives the victim's house a loss when it qualifies", is the one
  S-0319 BA2 part f reads the tally in, and that part's check tests it.

## Open questions

Q-1762 is answered in part. Question 5 is answered in full. Question 1 is answered apart from
the elasticity, damage, radius and warhead defaults, which are stated but `unmeasured` (Import
mapping). Question 6 is answered apart from whether a new resource cell takes a growth-list draw
(below). Questions 2, 3 and 4 leave the items below open: leaving the map (questions 2 and 4), and
which buildings and overlays count, the turn off a cliff face, the rebound on a sloped cell, the
stop decision on S-0317 F6's form, the rising cliff condition and the height that the cliff test
compares (question 3).

- Which buildings do not count for the building band of claim 6, and which overlays count as a
  building does. Low confidence; BA5.
- The velocity turned off the cliff face at a cliff landing (claim 6). Only the landing point
  matters for a sprite animation's position, but claim 7's stop decision uses the velocity the
  step has left. Low; BA5.
- The rebound at a contact on a sloped cell (claim 7): the component across the cell's surface
  is reversed, by S-0317 claim 10's rule. Low; no scenario here lands on a sloped cell, and it
  decides between a stopping and an impact landing there.
- The stop decision where the rebounded velocity is not 0, and a stop without a contact (claim
  7): both rest on S-0317 F6's exact form, which is `unmeasured` there. Low.
- The cliff test of a rising animation (claim 6): whether the condition *z*₁ > *z*₀ + *u* + 1
  holds as stated, and where the line between the two conditions lies near *u* = 0. Low,
  `unmeasured`.
- The height *z*₀ of claim 6's cliff test. This spec takes the flight position's height before
  the move, where S-0317 claim 10 takes the height of a piece's position before the move. For a
  sprite animation the two differ by the 10-lepton lift in a falling animation's first turn, and
  otherwise only by the fraction of a lepton that the truncation drops; no scenario here
  separates them, and the landing point of an undone first move (claim 6) rests on the same
  reading. Medium, `unmeasured`.
- Leaving the map in flight: whether an animation that flies off the map is removed for that, and
  what land type and ground height its landing reads there. A meteor's start position (F2) can
  lie off the map; whether its placement there succeeds is not established.
- How a resource overlay placed by claim 10 enters the resource lists of S-0401 (growth and
  spread), and so whether each new resource cell also takes the growth-list raw draw of S-0401
  claim 11, and where among that cell's draws it would fall (claim 13).
- The hits of an impact landing in a cell whose listing they change (S-0317's open question,
  the same here).
- Whether the hits of an impact landing (claim 7) skip a listed object with no health left, as
  S-0613 claim 13's gas hits do. The landing hits are a piece's ground-impact hits (S-0317 claim
  7), where the same question is open. For a building awaiting removal with 0 health (S-0312
  claim 21) skipping it and hitting it give the same result.
