---
id: Q-3054
title: Does a gas cloud made on a high bridge's deck sink through the deck?
status: answered
related: [S-0613, S-0102, S-0610]
answered-by: [S-0613, S-0306]
---

## Need
GitHub issue #424: in play, a gas cloud made by a unit on a high bridge is drawn under the
bridge. The drawing is not at fault: the particles really are under the deck. S-0613 claim 9
takes a gas particle's height above the terrain "at its own position; a particle is never on a
bridge deck", so a particle made at deck height (S-0613 F3, deck = terrain + 416) keeps sinking
under claims 9 and 11 toward 5 leptons above the ground under the bridge. Claim 10 only deflects
it sideways when its probe crosses the deck height. The engine needs to know whether this is the
original's behaviour or a gap in the spec, and what the cloud the issue shows is made of.

Evidence from the local real-install lane (`crates/app/tests/r8_lane.rs`, scene `gas`; numbers
are heights in leptons, read from our simulation, not from the original): one gas system made on
a high bridge's deck cell at deck height. Its particle's height above the terrain is 416 at
creation, 326 after 30 ticks, 116 after 90 ticks and 11 after 120 ticks, that is, it passes
under the deck within a few ticks and settles toward the ground. Over the same ticks the
particle stays over the deck cell. S-0613 claim 13 (hits use the ground layer, so objects on the
deck are never hit) and the draw order (S-0102) both treat it as under the bridge, and the client
draws it at that height.

## Questions
1. When an infantry unit standing on a high bridge's deck is killed by a weapon that leaves a gas
   cloud, does the cloud stay at the deck's height, or does it sink through the deck to the
   ground under the bridge, as S-0613 claims 9 to 11 read today?
2. If it stays on the deck: is its height then taken above the deck while it is over a deck cell,
   and what happens when it drifts off the bridge's cells (does it fall to the ground at once,
   sink at the usual rate, or stay at deck height)?
3. Is a cloud over a deck drawn above the bridge (with the deck's objects) or under it?
4. The cloud in the issue is released by a chaos drone standing on the deck. S-0613's Open
   questions leave open whether that weapon makes gas particles at all, or only an animation and
   an area berserk effect. Which is it? If it is an animation, where is it drawn when the drone is on a deck:
   at the deck's height, or under the bridge?

## Possible observation
S-0613 scenario G4 (a) already puts a cloud under a high bridge with infantry on the deck. The
mirror case settles questions 1 to 3: the virus sniper kills an infantry unit standing in the
middle of a high bridge's deck; record the cloud's on-screen height relative to the deck and to
the water or ground under it, for a few seconds, and whether a second infantry unit on the deck
next to it is hit. For question 4: a chaos drone attacks while standing on a high bridge's deck,
and the same record is made of its cloud.

## Current workaround
The code follows S-0613 claims 9 to 11 as written: particles made at deck height sink under the
deck and are drawn and damage as ground-layer particles. No code is changed for #424 until this
is answered.
