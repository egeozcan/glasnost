---
id: Q-0182
title: Draws of a start event's particles and smudges at map load
status: answered
related: [S-0109, S-0202, S-0312, S-0009, S-0010]
answered-by: [S-0202]
---

## Need
Q-0046 question 2b is answered only in part. S-0109 claim 14 gives the middle event's gates at
map load (μ = 0 from claim 25's sprite, the particle request when a particle type is given and
its count is positive, the height test below 30 leptons for scorch marks and craters, the
scorch-or-crater choice draw, and the order: particle requests before the choice). It leaves two
parts to effect specs that are not approved: the draws within each particle's creation, and a
scorch mark's or crater's type selection, placement and any further draws. S-0202 claim 10 and
its open question say these must not be treated as taking zero draws. `glasnost_sim::load::
anim_creation_draws` (tile animations, S-0010 claim 19) and `overlays::cell_animation` (an
overlay's cell animation, S-0009 claim 12) therefore take no draw for a start event that may
produce a middle event, and note it in the world's notes.

## Questions
1. A tile animation or an overlay's cell animation created at map load whose type has μ = 0 and
   names a particle type with a positive count: which scenario-stream draws does the creation of
   each requested particle take, in which order, and does any of them depend on the cell, the
   theater or the map state at load time?
2. When such an animation passes the height test and requests a scorch mark or a crater (after
   the choice draw, if both flags are on): does the type selection follow S-0312 claim 26, and if
   so in which form (plain, with the building test, or large, with the wide-and-tall preference)?
   If not, which draws do the selection and placement take?
3. Does a crater request at map load remove resource levels from the cell (S-0109 claim 14)
   before or after the overlay data of the map is read, so that a later step of S-0009's load
   sees the cell with or without them?

## Possible observation
S-0202 scenario R8 (part F) with a tile animation type of one frame that names a particle type
with a count of 1 or 2, and one with both the scorch and crater flags on, on flat morphable
ground, read with the random-state recorder after the map's load.

## Current workaround
`AnimRules::start_effects` marks every animation type with a chain reaction, a scorch mark, a
crater or particles (the importer carries no image-frame counts, so μ is not known). An animation
of such a type started at map load takes its stage-delay and bouncing draws (S-0202 claim 10)
and no start-event draw; the world's notes name this request. A chain reaction on a cell holding
no resource takes no draw (S-0202 claim 10); one on a resource cell (S-0401 claim 20) is the
economy topic's seam.
