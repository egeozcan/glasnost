---
id: Q-2097
title: S-0216 claim 6 and Q-1711 question 5 after the arena records' revision of 2026-10-29
status: open
related: [S-0216, Q-1711, Q-1713, Q-1502, M-0001, M-0002, M-0011, M-0015, M-0017]
answered-by: []
---

## Need
S-0216 (revision 20) claim 6, question 1, rests on a sentence the arena records no longer hold:
"The records state that every unit stood on its stated cell, so the units meant for those cells
stood on them". Its Open questions (*Q-1711 question 5, for the records*) repeat it: "The records
say that in every game every unit stood on its stated cell", and they leave open whether the
records' units were map entries or play-time placements.

The records as revised on 2026-10-29 say otherwise:
- M-0001 and M-0002 Setup: "A unit that its stated cell does not take is placed on a free
  neighbouring cell (M-0017 Setup)". M-0001's Notes add, for the cliff cells of Q-1502: "A unit
  stated on such a cell that the cell does not take stands on a free neighbouring cell".
- M-0011 and M-0015 Setup: "The units are not map entries. They are created on their stated cells
  once the scenario has loaded, before tick 0, so they are in play in the tick-0 state", and the
  game can refuse a unit on its stated cell.

The measurement wins (PROVENANCE §5), so S-0216 claim 6 and its Open questions need a revision.
Our code keeps to the approved revision where it can; the arena tool (`crates/tools/src/oracle.rs`)
now follows the records' placement rule by an interim of ours (Q-2096).

## Questions
1. With the records' revision, does S-0216 claim 6 still describe the infantry on the cliff cells,
   or do those units now stand on a neighbouring cell (so that claim 6's questions 2 and 3 apply to
   no unit of the records)? Claim 6 speaks of infantry; with our interim test (Q-2096) a vehicle
   is refused too: unit 16 of every side-A group of 17 or more units, whose stated cell holds a
   neutral building. Does claim 6, or its revision, cover vehicles?
2. Does the records' "created on their stated cells once the scenario has loaded, before tick 0,
   not map entries" settle Q-1711 question 5: are those units play-time placements in S-0216's
   sense (Terms), so that claims 1 to 5 (facings, the infantry standing-position draw, the draws
   and their order, the behaviour before the first update, the entry test) apply to them? If not,
   which of those claims do not apply, and what replaces them?
3. If they are play-time placements: is the *hunt* the records' units start in given by the placing
   step as a queued behaviour (S-0216 claim 4), or made current at placement?

## Possible observation
None needed beyond the records, if the operator can state the harness's placement method (Q-2096
asks for the rest).

## Current workaround
The arena keeps A1's interim for everything Q-2096 does not cover: every unit at its cell's centre
point, no standing-position draw, *hunt* current from placement. S-0216 as approved stays built for
the play-time placements of our own harnesses (`testkit::spawn_in_play`).
