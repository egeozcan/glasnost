---
id: Q-0172
title: Approach-position searches outside open level ground, and the failure centre after candidates
status: open
related: [S-0307, S-0215, S-0212]
answered-by: []
---

## Need

Q-0072 is marked answered, but S-0307 revision 3 (Open questions, "Q-0072 question 2,
remaining coverage") and S-0215 revision 15 (Open questions, "Q-0072, question 2") both say
that the approach-position search of S-0307 claims 15 to 17 is specified only for ground
infantry and vehicles on flat height-0 ground without bridges, attacking an object, with no
close-range property. They also say that when the radii of F4 gave candidates and none
qualified, the centre of the nearby-cell search of claim 17 is not established and must not be
assumed to be the target's cell. S-0215 claim 37 part f calls the search for every infantry unit
or vehicle that reaches its last row, so `combat::range::approach_search` has to answer in
every case.

## Questions

1. When the search's radii are not empty and no candidate qualifies, where is the nearby-cell
   search of S-0307 claim 17 centred: on the target's cell, the attacker's cell, the last
   candidate examined, or somewhere else? Does an ordinary actor then get a destination at all?
2. When the target is a cell rather than an object, is the search the same as F4 (with the
   cell's centre as *T* and no building lengthening), or different?
3. For a type with the close-range property, does the search differ?
4. On sloped or raised ground, are candidate points still measured at world height 0, or at the
   cell's own height?
5. Near bridges: are bridge cells and cells under a bridge deck candidates, and which layer does
   the clear-cell test read?
6. Are alternative cells (claim 16) outside the usable area ever chosen?
7. Does S-0307 claim 18's balloon-hover rule apply when the target is a cell, and if so, does
   the cell become the destination?

## Possible observation

RG10 and RG12 of S-0307 repeated with (1) a weapon range so short that every F4 candidate
fails the range test while the radii are not empty, recording the destination cell after the
step; (2) an attack order on a cell; (3) a close-range type; (4) the target on a raised plateau
with slope cells around it; (5) a high bridge between the attacker and the target.

## Current workaround

Every case takes the search of claims 15 to 17 unchanged (candidate points at world height 0,
ground-layer clear-cell test, a cell target's centre as *T* with no lengthening). When the radii
are not empty and no candidate qualifies, an ordinary actor drops its target and its
destination is cleared: no nearby-cell search is made, so no centre is assumed. With empty radii
the nearby-cell search is centred on the target's cell, as claim 17 states. A balloon-hover
vehicle attacking a cell takes that cell as its destination.
