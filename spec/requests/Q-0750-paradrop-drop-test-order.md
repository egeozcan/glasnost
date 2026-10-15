---
id: Q-0750
title: In which order a paradrop drop attempt judges its conditions
status: open
related: [S-0522, S-0208, S-0209, S-0212, S-0202]
answered-by: []
---

## Need

S-0522 claim 9 lists five conditions that must all hold for a drop to succeed:

- the S-0209 entry outcome under S-0208 claim 19's landing-entry rules;
- an S-0208 claim 3 standing position on the cell's ground record;
- that position inside the usable area;
- the deck-entry mark on a bridge cell;
- S-0212 claim 25's nearby-cell qualification.

Claims 9 and 16 say that an attempt which reaches the centre-class row of the standing-position table takes its R(0, 3) draw, even when it fails. The spec does not say which conditions are judged before the standing-position choice. So we cannot tell whether a drop that fails one of the other conditions still takes the draw. The code is `landing` in `crates/deterministic/sim/src/paradrop.rs`.

There is also a gap on bridge cells. The standing position is chosen from the ground record, but the paratrooper is then on the deck. The spec does not say which record holds its standing-position mark while it falls and after it lands.

## Questions

1. Take a drop point whose desired point is of the centre class, onto a cell where the attempt is refused. Observe it with the random-state recorder. Is one R(0, 3) taken in each of these cases?
   - (a) The drop is refused because another unit occupies the cell (the entry outcome is not clear).
   - (b) The drop is refused because the standing position is outside the usable area.
   - (c) The drop is refused on a bridge cell without the deck-entry mark.
   - (d) The drop is refused because the unit's class cannot enter the cell.
2. A drop lands on a high-bridge cell. Which record holds the paratrooper's standing-position mark while it falls and after it lands, the ground's or the deck's?

## Possible observation

- For question 1: S-0522 scenario P6 with the random-state recorder. Make one refused attempt per refusal kind, each with a centre-class drop point.
- For question 2: a drop onto a free bridge deck, then a second drop from the opposite side aimed at the same cell's ground record.

## Current workaround

- The standing-position choice comes first, so every refusal in question 1 takes the draw.
- After that, the tests run in this order: entry outcome, usable area, deck-entry mark, qualification.
- On a bridge cell the paratrooper goes on the deck layer, and its mark goes in the deck record.

These choices are marked in the code with this request's ID.
