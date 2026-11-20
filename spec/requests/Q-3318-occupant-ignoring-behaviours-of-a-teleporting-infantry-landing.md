---
id: Q-3318
title: Which behaviours make a teleporting infantry unit's landing choice ignore occupants
status: answered
related: [S-0208, S-0603, S-0215]
answered-by: [S-0208]
---

## Need

S-0208 r75 claim 19 says: "Occupants are ignored when it is *capture*, *sabotage*, *enter* or
*eaten*, and the destination is a vehicle or aircraft in *C*, or the building occupying *C*."

S-0603 r15, *Changes for other specs*, asks S-0208 (named there as approved revision 75) to replace
that sentence with "Occupants are ignored when it is *capture*, *enter*, *eaten* or *patrol*":
"*sabotage* is not one of the four behaviours, *patrol* is". It also asks for a new scenario W11 (c)
part with a saboteur type that respects occupants, and for a reworded claim 19 start sentence.

The two approved texts disagree for *sabotage* and *patrol*. S-0208 r75 was approved on the same
day as S-0603 r15, after it, without the change. The engine follows S-0208 r75 today
(`teleport::infantry_landing`).

## Questions

1. Is S-0603 r15's list (*capture*, *enter*, *eaten*, *patrol*) the one in force for S-0208 claim
   19, so that a saboteur landing on a building it is to sabotage respects occupants and a
   patrolling teleporting walker with an object destination ignores them?
2. If so, which destination objects count for *patrol*: the same vehicle or aircraft in *C*, or the
   building occupying *C*?

## Possible observation

S-0208 scenario W11 (c) with the saboteur part S-0603 r15 describes.

## Current workaround

S-0208 r75's list is kept (*capture*, *sabotage*, *enter*, *eaten*); nothing changes until S-0208
takes the change. S-0603 claim 16 needs only *eaten*, which both lists share.
