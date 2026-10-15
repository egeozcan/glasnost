---
id: Q-0700
title: Where a let-out passenger is put and which way it faces
status: open
related: [S-0507, S-0208]
answered-by: []
---

## Need

S-0507 claim 10 puts a let-out passenger at the centre of the serving neighbour cell: an
infantry unit "in a free infantry position of that cell", a vehicle "at a nearby location where
it can stand, searched from that centre; exact search ties are not specified here". It does not
say which free position an infantry unit takes when several are free. Claim 13 releases an air
transport's passenger "at its own cell" without naming the position, the facing, or what happens
when the passenger cannot be placed.

The code in `crates/deterministic/sim/src/transport.rs` (`put_out`) needs the rule.

## Questions

1. Which free infantry position (S-0208 table P) does a let-out infantry unit take: the centre
   first, the lowest-numbered free one, or another order?
2. What search places a let-out vehicle (the nearby-cell search of S-0212 claim 25, and with
   which settings), and how are its ties broken?
3. In claim 13 (air transport landed), where in the cell is the passenger put, which way does it
   face, does it receive any order or scatter request, and what happens when it cannot be placed?

## Possible observation

S-0507 scenarios T3 and T10 (T11 for the air case): let out infantry into a neighbour cell with different
positions already taken and record which position each takes; let out a vehicle with the serving
cell's centre blocked; record the facings and orders after an air release.

## Current workaround

Infantry take the first free position in the order 0 to 4 (centre first). A vehicle is put at the
centre of the serving cell (the entry test has already found that cell open). In claim 13 the
passenger is put the same way in the transport's own cell, keeps the transport's body facing and
gets no order; a passenger that cannot be placed stays inside. The choice is marked in the code
with this request's ID.
