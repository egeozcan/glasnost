---
id: Q-0560
title: When does a walking infantry unit move between a cell's ground and bridge-deck layers?
status: open
related: [S-0208, S-0209, S-0212, S-0207]
answered-by: []
---

## Need

S-0209 Terms say an object on a bridge deck belongs to the deck layer, and that a walker's ask
reads the deck record of a cell exactly 4 height levels below its reference level, its reference
being its cell's level plus 4 "while it is on a bridge deck". S-0208 claim 1 puts the walking
target 416 leptons above *H* on a deck, and claim 3 chooses the position in the record "for the
surface the walker is on". S-0212 claim 10 gives the layer of the cell a path step enters. For
ground vehicles S-0207 claim 5 says exactly when the vehicle gets onto and off a deck (on the cell
change, by the levels of the cells left and entered). No claim says when a walker's own layer
changes: when it chooses a walking target on a deck (or off one), when its position moves into
that cell (S-0209 claim 13's entering moment), or when it arrives on its target. The layer decides
which list the walker is in, its reference level for the next ask, which record its old mark is
cleared from, and how it is drawn.

`crates/deterministic/sim/src/walk.rs` (`entered_layer`, `changed_cell`) had walkers never leave
the ground layer, so infantry crossing a high bridge walked at ground height under the deck.

## Questions

1. On which tick does a walker stepping from ground at deck level onto a high-bridge cell become
   a deck object (listed in the deck layer, reference level plus 4): the asking tick on which it
   chooses its target there, the tick its position first lies in that cell, or its arrival tick?
   The same for stepping off a deck onto ground at deck level.
2. A walker's straight line to a diagonal neighbour can pass briefly through a third cell. If its
   position enters such a cell, which layer is it listed in there when that cell carries a deck,
   and when it does not?
3. On a deck spanning uneven ground, S-0209 Terms give deck objects but the ground record for a
   walker asking about the next deck cell 1 level above or below its own (S-0209 Open questions).
   Which record is the walking target's position then chosen and reserved in, and is the target
   at *H* or *H* + 416 there?

## Possible observation

S-0209 scenario C22 with a soldier walking from a plateau across a high bridge and off the other
end, logging every tick its cell, height, listing layer and the marks of both records of the
cells it passes.

## Current workaround

A walker's ask reads the objects and record of the next cell by S-0209 Terms' layer rules with its
reference level; it chooses and reserves its position in that record, and its target is 416
leptons above *H* when that record is the deck's (S-0208 claims 1 and 3). The walker's layer
changes when its position moves into another cell (S-0209 claim 13): in its target's cell it takes
the target's layer; in any other cell it stays on the deck only if it was on one and that cell
carries a deck, and is on the ground otherwise. On uneven ground under a deck the record layer
decides (so a deck walker asking about a deck cell 1 level off reads and reserves the ground
record and targets *H*), pending question 3.
