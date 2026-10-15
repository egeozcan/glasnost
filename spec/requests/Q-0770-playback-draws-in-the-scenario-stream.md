---
id: Q-0770
title: Which animations take the playback draws of S-0109 claims 3 and 11 from the scenario stream, and in which order
status: open
related: [S-0109, S-0202, S-0203, S-0214]
answered-by: []
---

## Need
S-0109 claim 3 says a follow-up switch whose range has *d*<sub>1</sub>′ ≠ *d*<sub>2</sub>′ takes
one ranged draw from the scenario random stream, and claim 11 says the loop delay between passes
of a type whose random loop delay pair is not (0, 0) is a ranged draw from the scenario random
stream. Both happen during playback, long after creation. Our simulation takes only the creation
draws (S-0202 claim 10, `load::anim_created_at`) and keeps no playback clock, so it never takes
these later draws, and its stream drifts from the original's as soon as a played animation with
a random follow-up delay or a random loop delay reaches a pass end or a follow-up switch. To
decide what the simulation must model, we need to know which animations take these draws and
where they fall among the other draws of a tick. S-0214 claim 14 already says the move-target
marker takes no scenario draw; the question is about every other animation.

## Questions
1. Do the follow-up-delay draw (claim 3) and the between-pass loop-delay draw (claim 11) come
   from the scenario random stream for every animation in play, including one that no player can
   see: under the shroud, off the screen, whose type's detail field is above the player's detail
   level (claim 19), or whose type has no sprite?
2. Does an animation whose type has no sprite still play passes and switch to its follow-up type
   (and so take these draws), or does it end at once?
3. Within a tick, does each such draw fall at the animation's own turn in the update order
   (S-0203), so that it is ordered among the other objects' draws by that turn?
4. Is any other draw taken during playback (for example by a type change other than claim 15's
   follow-up switch)?

## Possible observation
A scenario with one trigger-played animation whose type has a descending random rate pair and a
follow-up type with another descending pair, and a second with a random loop delay pair and two
passes, recorded with the stream-state recorder of AN3 and AN9 each tick around the pass end and
the follow-up switch; repeat with the animation under the shroud, off the screen, and with a
detail field above the player's level.

## Current workaround
Interim rule, recorded in docs/development.md (section *Animation playback (M6b)*): the drawing
plays every animation in the app and takes these two draws from its own cosmetic stream
(`Rng::new(seed)`), never from the simulation's scenario stream; the simulation's stream is not
touched by playback. The creation draw the simulation takes is carried to the drawing in
`Effect::Anim::delay`, so the first stage delay matches the simulation's.
