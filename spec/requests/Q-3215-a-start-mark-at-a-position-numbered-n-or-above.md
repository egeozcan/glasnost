---
id: Q-3215
title: Does "player at position X" read a start mark at a position numbered N or above?
status: open
related: [S-0009, S-0011]
answered-by: []
---

## Need
S-0009 claim 29 (r45) lists three cases that are unspecified in the original. The first is "a
house that holds a position numbered N or above", where N is the length of the start list
(claim 17). For it, "our engine ignores the mark … concerned". Its References rule says "player
at position X" (claim 21, S-0011 claim 23) means the house whose house number equals the mark
of position X, with no limit on X. On a map with 4 start waypoint markers, 2 players and pair 11
= (0, 1), positions 4 and 5 carry the marks 0 and 1 and lie above N − 1 = 3. The engine (sim
batch S27, `starts::player_at`) has to decide whether "player at position E" names house 0 there.

## Questions
1. Does "ignores the mark" mean the mark is ignored for every purpose, so that "player at position
   E" is empty in the example? Or is it ignored only for which position a house holds, so that the
   References rule still names house 0?
2. Does the answer differ for a mark at position N or above that names no house, or a passive
   house?

## Possible observation
None in the original for question 1: the case is unspecified there, and the engine's rule is a
departure. If the spec side has an observation of what the original does with such a
reference, the test is scenario M13b variant E with pair 11 = (0, 0), N = 4 and the reveal
trigger on "player at position E".

## Current workaround
Our engine's interim rule, noted in `docs/dev/simulation.md` (*sim issues S27*): the mark is
ignored for every purpose. The late-phase marks of positions N … 15 are kept as −1 in
`World::start_marks`, so "player at position E" is empty in the example.
