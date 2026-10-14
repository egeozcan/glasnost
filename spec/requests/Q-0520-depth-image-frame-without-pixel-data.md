---
id: Q-0520
title: Shared building depth image whose image frame 0 has a rectangle but no pixel data
status: open
related: [S-0102]
answered-by: []
---

## Need

S-0102 revision 14, claim 33 separates two cases for image frame 0 of `BUILDNGZ.SHA`:
an image frame 0 that "cannot be decoded" (our engine warns and draws eligible bodies
with the plain upright profile of F2), and a "valid, decodable empty frame rectangle"
of width or height 0 (empty coverage, no body pixels, no warning). D42 builds its
empty-rectangle fixtures with "a nonzero pixel-data position", which suggests that a
position of 0 is a separate case.

A shape file frame header can also give a non-empty rectangle (width and height both
above 0) with a pixel-data position of 0. The shape file readers we follow for the format
(XCC, OpenRA) treat a frame with position 0 as a frame with no pixels and decode it as
nothing. Claim 33 does not say whether such an image frame 0 counts as undecodable
input or as a valid frame that covers nothing. The renderer (`app::mapview::build`,
`read_companion`) has to pick one, and the two give different pictures.

## Questions

1. When image frame 0 of the shared file has a non-empty rectangle and a pixel-data
   position of 0, does the original reach play? If it does, are eligible building
   bodies drawn at all, and do they write depth?
2. For our engine's safe departure in claim 33, is this input "an image frame 0 that
   cannot be decoded" (warn, no companion, plain upright F2), or a valid frame whose
   coverage is empty (no body pixels, no body depth, no warning), or something else?

## Possible observation

D42's empty-rectangle variant, with image frame 0's rectangle 16 × 16 on a 16 × 16 canvas
and its pixel-data position set to 0, with no pixel payload after the frame header.
Record whether loading reaches play and, if it does, the body pixels and the infantry
probe crossing.

## Current workaround

Treated as undecodable input: an eligible body (foundation under 8 cells wide) is drawn
with the plain upright profile of S-0102 F2, keeping its draw point and bias and writing
depth. The body is marked provisional, and a scene note names this request.
