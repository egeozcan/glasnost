---
id: Q-1762
title: Bouncing (sprite debris) animations: motion, landing and end
status: answered
related: [S-0109, S-0312, S-0303, S-0202, Q-1702]
answered-by: [S-0319]
---

## Need

S-0109 Terms define a *bouncing type* (bouncer or meteor flag): "Such an animation follows its
own motion, and its landing ends it (the specs of projectiles and debris own both)." No approved
spec gives that motion or landing. S-0312 claim 4 and F1 say only how many debris pieces a death
throws and at what height. S-0202 claim 10 gives the creation draws of a bouncing type, but not
what the drawn values do. Q-1702 asks the same things for voxel debris pieces only.

In our engine a bouncing animation takes its creation draws (`glasnost_sim::load`) and then
plays its stages in place with no motion and no landing. That is the seam "bouncing motion and
landing" in docs/development.md, `glasnost_sim::anims`. A looping bouncing type never ends,
so a destroyed vehicle's sprite debris (S-0312 claim 4: its type's debris animations or the
game-wide metallic debris list, `glasnost_sim::combat::death::debris`) keeps playing for ever
at its death point. This most likely explains the lead's play note "destroyed units leave some
spinning particles".

## Questions

1. **Start of motion.** How do a bouncing animation's creation draws turn into its first speed
   and direction (horizontal and vertical)? Which type keys or game-wide keys scale them, and
   what is each key's default when absent?
2. **Flight.** How does it move each tick: gravity, any drag, and does its stage keep playing
   while it flies? Does it stop at cliffs, buildings or bridge decks, and can it leave the map?
3. **Landing and bounces.** What counts as landing (ground, water, bridge deck)? Does it bounce,
   how many times, and how does each bounce change its speed? What happens at the final landing:
   does the animation end at once, switch to its follow-up type, make another animation, a
   sound, a scorch mark or damage? What does landing on water do?
4. **End without landing.** If a bouncing type loops for ever and never lands (for example it
   is created below the surface, or its motion is zero), does anything else end it, such as a
   time limit, its passes running out, or leaving the screen or map?
5. **Meteor flag.** Does a type with the meteor flag but not the bouncer flag move and land the
   same way? If not, what differs?
6. **Random stream.** Do the motion or the landing take further draws from the scenario stream
   after creation? If so, how many and in what order (S-0202)?

## Possible observation

Destroy a test vehicle whose debris list names one looping bouncing sprite type, on flat ground,
next to water, next to a cliff and under a bridge. Record each piece's path, its bounces, where
and when it disappears, and what it leaves. Repeat with the meteor flag instead of the bouncer
flag. Use the random-state recorder for question 6.

## Current workaround

Bouncing animations play their stages in place with no motion and no landing. Looping ones never
end. No interim value is invented. Any interim ending rule, such as ending a bouncing animation
after its first pass, would need the lead's approval and a quirk switch marked unspecified.
