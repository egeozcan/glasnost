---
id: Q-0021
title: Depth bias of overlay shadows, and the standing-pose state of vehicles drawn as sprites
status: open
related: [S-0102, S-0112]
answered-by: []
---

## Need

The map viewer draws every overlay's shadow (S-0112 claim 8) and every
vehicle drawn as a sprite (S-0102 claims 21 and 22). Two things it needs
are not given by an approved spec:

1. S-0102 claim 13 lists depth biases for building, terrain-object,
   animation, projectile and unit shadows, and for overlays themselves
   ("overlay, flat type, ore and gems, wall, rock": −2 − 15 × ℓ; "any
   other upright type": −17 − 15 × ℓ), but no row for the shadow image
   frame of an overlay. S-0102 claim 27 says cell-pass overlay shadows are
   writers, and claim 12 says shadows drawn as sprites use the flat
   profile, so only the bias is missing. Code: `app::mapview::build`,
   `Builder::overlays`.
2. S-0102 claim 22 picks a sprite vehicle's image frame by a table whose
   rows include "its standing-pose state is off" (frame *d*) before the
   rows for *F*<sub>s</sub> > 0 and the walking frames. The claim says
   when the standing-pose state is on or off is not settled (Open
   questions). A vehicle placed by the map that has not moved, fired or
   been destroyed reaches that row, so its first drawn frame depends on
   it. Code: `app::mapview::build`, `Builder::vehicle_items`.

## Questions

1. What depth bias does the shadow image frame of an overlay take (for
   each kind of overlay: ore and gems, walls, crates, bridge decks, other
   flat and upright types), and does it depend on the cell's height level
   or on the bridge deck height?
2. Is a vehicle drawn as a sprite in the standing-pose state when it has
   just been placed by the map (no tick run yet), and after it stops
   moving? What turns the state on and off?

## Possible observation

1. A flat test map with an ore overlay and a fence-like upright overlay
   whose shadows fall over a terrain object's shadow (−3) and a building
   shadow (−4): whether each pixel is darkened once or twice (S-0102 claim
   27) brackets the bias; repeat at height level 1.
2. A map-placed sprite vehicle type whose art gives distinct standing
   frames (a named loose art file): the frame shown at the first drawn
   frame, and after one move and stop.

## Current workaround

1. The overlay shadow takes its overlay's own bias with the flat profile,
   and the viewer marks it provisional (note citing Q-0021).
2. A placed sprite vehicle shows the *F*<sub>s</sub> > 0 row if its type
   has standing frames, otherwise the walking row's first frame (the last
   row of the table); the viewer marks it provisional (note citing
   Q-0021).
