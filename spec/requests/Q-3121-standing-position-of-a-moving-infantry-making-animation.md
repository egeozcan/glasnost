---
id: Q-3121
title: The standing position and creation point of a moving infantry-making animation
status: answered
related: [S-0611, S-0109, S-0319]
answered-by: [S-0611]
---

## Need
S-0611 claim 9 says the rules of claims 9 to 11 apply to every infantry-making animation, and
S-0109 claims 16 and 22 repeat it for every creator, attached animations included. Claim 8 says
that while it plays an infantry-making animation "holds its own standing position: in each of its
updates its position is marked taken", and that "at its end the position is shown free again".
Claim 10 places the new unit at "the animation's world point".

For an animation that stays where it was created this is clear. Two kinds of animation move while
they play:
- an animation attached to an owner (S-0109 claim 22), whose position is the owner's plus its
  displacement in each update, for example a paratrooper's parachute (S-0522 claim 13) or a drain
  animation (S-0608);
- an animation of a bouncing type (S-0319), which moves by its flight in every turn and ends at a
  landing (S-0319 claims 7 and 8) rather than at the end of its last pass.

The specs do not say, for such an animation, which standing position it marks in each update (the
one at its current point, or the one at its creation point), what becomes of the mark it left in an
earlier cell when its point has moved to another cell, and where the creation of claim 10 is tried.
For a bouncing animation they also do not say whether a landing counts as the end of claim 9 (a
creation attempt, with claim 11's retry, which a landing that ends the animation in the same turn
leaves no room for) or as a removal with no infantry. For an attached animation whose owner is
removed (S-0109 claim 22: "no infantry" in that update), they do not say whether the standing
position it marked is shown free. (Issue #291.)

## Questions
1. For an infantry-making animation attached to an owner, which standing position is marked in each
   update: the one its current point is in, or another one? When its point moves into another
   standing position or cell, is the earlier mark freed, kept until the end, or left as it is?
2. Where is the new unit of claim 10 placed for such an animation: at its point at the end?
3. For an infantry-making animation of a bouncing type, the same two questions. Does its landing
   (S-0319 claims 7 and 8) make a creation attempt of claim 9, and if it does, what happens when the
   placement is refused (claim 11's retry needs the animation to stay)?
4. When an attached infantry-making animation is deleted because its owner left the game (S-0109
   claim 22), or any infantry-making animation is removed by another rule before its last pass ends
   (a drain link's end, a new teleport source pick, the end of loading for a first-pass tile
   animation), is its standing position shown free as at its end (claim 8), and is that free
   "whatever else has marked it" (`quirks.infantry_animation_occupancy`)?

## Possible observation
An art section that gives a parachute or a bouncing debris type a make-infantry index (S-0611 F4),
with a one-entry animation-to-infantry list. Record each tick the occupancy marks of the cells the
animation passes and of its creation cell, and whether and where an infantry unit appears.

## Current workaround
An infantry-making animation that every other creator makes (a projectile or debris trailer, a tile
or overlay animation, a teleport source marker, a reverse request) now marks its standing position,
tries its creation at its end and retries as claims 8 to 11 say, when it is neither attached to an
owner nor of a bouncing type at its creation. The standing position is the one of its creation point.
An attached or bouncing one is still played as an ordinary animation with none of claims 8 to 11,
as before issue #291. A removal by another rule frees the position as its end does and makes no
infantry. See `crate::anims` and `crate::mutation` in `crates/deterministic/sim/src/`.
