---
id: Q-0490
title: Building bodies when the shared depth image file is missing or unreadable
status: open
related: [S-0102]
answered-by: []
---

## Need

S-0102 revision 13 (claim 31, F5, Import mapping) gives every building body on a
foundation under 8 cells wide the companion depth image from image frame 0 of
`BUILDNGZ.SHA`, and clips body pixels outside its stored rectangle. Its Import
mapping row says that "missing-file recovery is not specified here". The map
viewer and the live game (`app::mapview::build`, `body_companion`) must still
draw buildings when an install, a mod or a test setup has no such file, or when
the file is there but its image frame 0 cannot be read.

## Questions

1. With `BUILDNGZ.SHA` absent from every place the game looks for files, does the
   game still start a skirmish? If it does, how are eligible building bodies
   drawn: with the plain upright profile of F2, not at all, or otherwise?
2. If the file exists but has no image frames, or its image frame 0 has an empty
   stored rectangle (width or height 0), are eligible bodies drawn with the plain
   upright profile, not drawn at all (every pixel outside coverage, as F5 read
   literally gives), or otherwise?
3. If the file is present but cannot be decoded as a shape file (truncated or not
   a shape file at all), which of the outcomes in question 2 applies?

## Possible observation

A skirmish in a test install with a loose replacement for `BUILDNGZ.SHA` that is
(a) left out, (b) a valid shape file with zero frames, (c) one frame with an
empty rectangle, (d) truncated bytes. Place a narrow building on bare ground,
with an infantry probe walking in front of and behind it, as in D39.

## Current workaround

When the file is missing or cannot be parsed, or its image frame 0 cannot be
decoded, an eligible body is drawn with the plain upright profile of S-0102 F2
(no companion), marked provisional and given a scene note naming this request.
Bodies on foundations 8 or more cells wide are unaffected (F5: no companion).
An image frame 0 that decodes with an empty rectangle is used as F5 says:
nothing of the body is covered, so nothing of it is drawn.
