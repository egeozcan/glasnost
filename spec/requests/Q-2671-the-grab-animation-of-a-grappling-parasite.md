---
id: Q-2671
title: The grab animation of a grappling parasite, and what a held ship shows of its parasite
status: answered
related: [S-0614, S-0113, S-0109]
answered-by: [S-0614]
---

## Need

The lead's play note 45 of 2026-11-05 (M8 A4 round 8): "ship attacked by the giant squid, squid
cannot be seen". S-0614 claim 3 gives what the lead saw: an attached parasite "stays off the map,
travels with the host", so our view draws nothing of the squid while it holds the ship. What the
held ship does show is the host's roll (S-0113 claim 14), the wakes at each push start and the
three splash effects at each strike (S-0614 claim 10), and these are drawn.

S-0614's Import mapping also names a grab animation: "The grab animation of claim 10 is not read
from a key: the original selects the animation type identified by a fixed engine constant
(presentation). When no such animation type exists, no grab animation is shown and the cycle runs
as described." No claim says which animation type that is, when it is created, where it is
placed, on which layer, how long it plays, or whether its creation takes draws from either random
stream. The Import mapping's "no such animation type" case is about an install that lacks the
type; ours is a gap in the spec, which does not identify the type, and we cannot fill it without
that type, so our simulation and view create none. That animation is the only picture of the squid
itself while it holds a ship, so without it the lead sees a ship rocking and splashing with no
attacker.

## Questions

1. Which animation type is the grab animation, identified by what the player sees (its image
   file name as the art and rules give it, or the animation section it is), and is it the same
   for every grappling parasite type?
2. When is it created: at the latch (the grab's first turn), at the end of the grab, at every push
   start, or at every strike? Is it created once per hold or once per cycle?
3. Where is it placed: at the host's position (at which height), attached to the host so that it
   moves and is removed with it, or unattached? Does it face or follow the host's facing or roll?
4. Which layer and translucency is it drawn with, and does a cloaked or submerged parasite change
   whether it is shown to the host's owner or to other players?
5. How long does it play (its loop count and any loop delay), and is it removed when the parasite
   leaves the host (claim 13), when the host sinks, or when the parasite is killed?
6. Does its creation take draws from the scenario random stream (so that it belongs to the
   simulation), or only from the cosmetic one?

## Possible observation

S-0614 scenario P6's set-up (a grappling parasite latching onto a ship on open water), recorded
frame by frame from the latch to the third strike and then through a release by moving the host
onto a non-water cell: every animation created near the host, with its tick, image file, frame,
screen position against the host's, and layer; repeated with the parasite owned by the viewer and
by another player.

## Current workaround

None created: the view draws the host's roll, the wakes and the splashes, and nothing for the
parasite while it is attached (S-0614 claim 3); the grab animation waits on this request's answer
naming its type. Recorded as M8 A4 round 8 note 45 in `docs/m8-claims/a4-play-notes.md`.
