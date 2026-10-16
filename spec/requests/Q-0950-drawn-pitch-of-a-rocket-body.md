---
id: Q-0950
title: How a rocket missile's pitch shows in its drawn body
status: answered
related: [S-0105, S-0106, S-0506, S-0102]
answered-by: [S-0105]
---

## Need

S-0506 gives a rocket missile a nose pitch *p* (Terms; formulas F3 to F7) that changes through
its phases: tilt turns it between the set's initial and final pitch, ascent and descent follow
it, cruise levels it out. S-0105 claim 3 draws a body at its 32-step facing only and says
"ramps and pitch add tilt, S-0106"; S-0106 claim 5 gives flying and rocket bodies "no tilt from
slopes" and says their orientation "comes from rules of the aircraft specs that this spec does
not fix"; S-0105's open question on cache keys says the flying and rocket body keys are not
determined. No spec says whether or how *p* is drawn.

Lead review of 2026-10-15 (scene `v3-rocket-in-flight-3`): a rocket that climbs after launch is
drawn horizontal, nose along its heading, while it moves up the screen, so its facing does not
look right for its flight direction. Our drawing (`mapview::build::vehicle_voxels`, via
`Live::mobile`) passes only the body facing, so a rocket never shows its pitch.

## Questions

1. Is a rocket missile's body drawn pitched by its nose pitch *p* (tilt, ascent, cruise,
   descent), and if so by what transform: a rotation of the body about its lateral axis by *p*,
   composed with the heading rotation of S-0105 claim 3?
2. If the body is pitched, how many distinct pitch images does it have (is *p* quantised for
   drawing, as the heading is to 32 steps), and what is the quantisation rule?
3. Is the answer the same for the shadow of a rocket (S-0106 claim 11, which leaves a rocket's
   shadow open), and for the rocket's image reuse key (S-0105 claim 26, open for the rocket
   locomotor)?
4. Does an aircraft with the flying locomotor show any pitch or roll in its body (take-off,
   landing, banking)? If it does not, say so; this request only needs the rocket.

## Possible observation

Screen captures of one rocket per tick of its flight (S-0506 scenario R10 or a variant), with
the simulation's pitch recorded beside each capture, over flat ground, to read the drawn nose
angle against *p* and count the distinct images.

## Current workaround

The rocket is drawn with its 32-step heading only, level (no pitch image). Interim rule in
`docs/development.md`, section *Lead visual review (M6c, row 4)*.
