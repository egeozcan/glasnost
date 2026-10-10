---
id: Q-0008
title: Building secondary sprites at later type reads, and the art of the kinds S-0007 leaves out
status: answered
related: [S-0007]
answered-by: [S-0007]
---

## Need
The importer looks up each type's art per S-0007 for the asset viewer and,
later, the renderer. Two cases have no specified result. Code gap:
`glasnost_convert::art`.

## Questions
1. S-0007 open question 1: when a later type read of a building uses an art
   section that does not set one of the eight secondary sprite names of claim
   11 (deploying, roof deploying, door, under-door, under-roof-door, rubble,
   special overlay, bib), does the building keep the sprite an earlier read
   found, or has it none afterwards? Does an empty value behave like an
   absent key here (as it does for the build-up name)?
2. S-0007 open question 3: how do projectiles, particles and voxel
   animations name their art files (image key, its default, theater
   handling, fallbacks, voxel or sprite), and when are the names built?
3. Whether weapons, warheads, particle systems, superweapons and tiberiums
   load any art file of their own through the image name, or none.

## Possible observation
For question 1: a test map whose override section gives a building an image
key naming a second art section that lacks the bib name, where the first
art section has one; a screenshot shows whether the bib is drawn.

## Current workaround
For question 1 the importer records the secondary sprite as not determined
(a note in the converted art, no file name) whenever a later read's art
section lacks a name an earlier read's section had. For questions 2 and 3 no
art is looked up for those kinds.
