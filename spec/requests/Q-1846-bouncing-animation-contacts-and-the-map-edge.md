---
id: Q-1846
title: A bouncing animation's contacts on buildings, cliff faces and slopes, and off the map
status: answered
related: [S-0319, S-0317, Q-1762, Q-1834, Q-1836]
answered-by: [S-0319]
---

## Need
S-0319 (revision 9) answers Q-1762 in part. Its Open questions leave the contact items of Q-1762
question 3 and the map-edge items of questions 2 and 4 open. `glasnost-sim` `bounce::step` and
`bounce::land` need a rule for each in every turn of a bouncing animation, so they run on interim
rules taken from S-0317's requests for debris pieces (Q-1834, Q-1836). This request asks for the
bouncing-animation side of those items, which S-0319 names as its own.

## Questions
1. Building band (claim 6): which buildings in a cell do not count (for example a wall, a gate, a
   building being placed or sold, an under-construction one), and do any overlays count as a
   building does (walls, fences, bridges' overlays)?
2. Cliff landing (claim 6): after the move is undone, what velocity does the animation keep, and
   so does claim 7's stop measure see a turned velocity? Is it the velocity before the move with
   the horizontal components turned, or unchanged?
3. Contact on a sloped cell (claim 7): is the rebound the reversal of the component across the
   cell's surface (S-0317 claim 10), or the vertical speed reversed as on a flat cell?
4. Stop decision (claim 7): with a rebounded velocity that is not 0, and for a stop without any
   contact, does the measure S follow S-0317 F6's form as stated?
5. Rising cliff test (claim 6): does an animation moving upward into a higher cell meet the
   condition z₁ > z₀ + u + 1 as stated, and which way does a vertical speed of exactly 0 go?
6. The height z₀ of the cliff test: the flight position's height before the move (as S-0319
   reads it), or the height of the position in play before the move (as S-0317 claim 10 reads it
   for pieces)? They differ by the 10-lepton lift in a falling animation's first turn.
7. Off the map: is an animation that flies off the map removed for that? While off the map, what
   ground height, height level, land type and high-bridge state does its contact test and its
   end read? Is a meteor whose start position (F2) lies off the map placed at all, and does it
   take its creation draws? Does any of this differ outside the playable area but inside the
   map's full cell grid?
8. Impact hits (claim 7): in a cell whose listing an earlier hit changes, and for a listed object
   with no health left, as S-0317's open question for pieces.

## Possible observation
S-0319's scenario BA5 (cliff and building contacts) with a sloped-cell variant, and a meteor
type with a large horizontal speed made near a map edge with the read-only recorder logging its
position each tick, its draws and its end effects (as Q-1836's observation).

## Current workaround
Interim rules, each citing Q-1846 at its use in `crates/deterministic/sim/src/bounce.rs`:
- every ground-layer building in the cell counts for the building band, and no overlay does
  (question 1);
- after an undone cliff move the velocity before the move is kept with each horizontal component
  along which the cell changed reversed (S-0317's Q-1834 interim; question 2);
- a contact on a sloped cell rebounds as on a flat cell, the vertical speed reversed (question 3);
- the stop measure is S-0317 F6 as `crate::pieces::stopped` computes it, with or without a
  contact (question 4);
- an animation is placed and takes its turns wherever it is, a meteor whose start position lies
  off the map included; off the map the ground height and height level are 0, the land type is
  not water and there is no high bridge (S-0317's Q-1836 interim, through the same helpers;
  question 7);
- an impact landing's hits are the debris pieces' ground-impact hits (`pieces::ground_hits`),
  with the same open points as S-0317 (question 8).

Questions 5 and 6 are coded as S-0319 claim 6 states them, not as interims: the rising cliff
condition z₁ > z₀ + u + 1, a vertical speed of 0 meeting neither condition, and z₀ the flight
position's height before the move.
