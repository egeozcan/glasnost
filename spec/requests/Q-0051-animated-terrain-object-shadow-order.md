---
id: Q-0051
title: Order of an animated terrain object's body and shadow in the object pass
status: open
related: [S-0102]
answered-by: []
---

## Need
S-0102 claim 23 (revision 4) draws an animated terrain object in the object
pass instead of the cell pass, and claims 10 and 13 make both its body
(−*L(z)* − 12, upright) and its shadow (−*L(z)* − 3, flat) depth readers
there. Readers store no depth, so where the shadow image frame overlaps the
body image frame, the order of the two decides what shows: a shadow drawn
after the body darkens the body's pixels (as claim 28 says for a sprite
unit's own shadow), and a shadow drawn before it is painted over. No claim
states the order for an animated terrain object, and claim 16's "with
their shadows" covers only terrain objects that are not animated. Code:
`app::mapview::build`, `Builder::terrain_objects`.

## Questions
1. In the object pass, is an animated terrain object's shadow drawn right
   after its body (so it darkens the body where they overlap), right
   before it, or elsewhere (for example in a separate step of the drawn
   frame)?
2. Does the answer differ for an ore mine (the spawns-resources flag, drawn
   16 rows higher) and for an animated object whose destruction sequence
   has begun?

## Possible observation
A stock terrain type marked animated placed by the map on flat ground,
in a pose where its shadow image frame overlaps its body image frame
(read the shape file): whether the body pixels inside the shadow's
outline are darkened (S-0102 claim 26's reading, as in D30).

## Current workaround
The body is drawn first and the shadow right after it, as for a sprite
unit's own shadow (claim 28), and both items are marked provisional with a
note citing this request.
