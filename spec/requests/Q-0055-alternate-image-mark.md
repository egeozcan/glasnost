---
id: Q-0055
title: Where a tile file carries the alternate-image mark
status: answered
related: [S-0010]
answered-by: [S-0010]
---

## Need
S-0010 revision 9 (claim 13, Terms) adds the *alternate-image mark*: "a
yes-or-no mark in the tile file, taken from the plain image's tile cell at
the cell's sub-tile". A tile cell with the mark does not use the variant
pattern; it shows image 1 when the map cell's alternate-image state is set
and image 0 (the plain image) when it is clear, and the state is clear at
map load. The spec says the tile file format is public, but it does not say
which value of a tile cell's record is the mark, so the importer
(`convert::theater`, the variant choice `variant_image`) cannot read it.

## Questions
1. Which value of a tile cell's record in the tile file is the
   alternate-image mark: one bit of the per-cell flag value (which one), or
   another field? Is it the mark exactly when that value is non-zero?
2. Is the mark read from the plain image's file only (as the Terms say),
   whatever the variant files' tile cells carry?

## Possible observation
A loose copy of a stock one-cell tile file that has variants, with the
candidate value changed, placed on a map in a pattern of cells whose
pattern choice would show a variant: with the mark the cells all show the
plain image.

## Current workaround
The importer reads no mark: every tile cell uses the pattern of claim 13,
as before revision 9. Since the state is clear at map load and no spec
sets it, a marked cell would show the plain image; until this is answered
it may show a variant instead. Placeholder at `convert::theater::variant_image`.
