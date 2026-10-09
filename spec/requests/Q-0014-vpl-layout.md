---
id: Q-0014
title: Byte layout of the voxel lighting table file
status: open
related: [S-0105]
answered-by: []
---

## Need
S-0105 claim 19 and Import mapping say the voxel lighting table's header
gives the remap range and the number of rows, that it carries a palette
not used for drawing, and that each row has 256 entries, but leave the
byte layout to a formats spec. Neither OpenRA nor XCC reads this file.
Code gap: `glasnost_fmt::vpl`.

## Questions
1. The header: how many bytes, which fields (remap range start and end,
   row count, others), in which order and byte order?
2. Where the palette and the rows lie, and whether the row count in the
   header is always the number of rows stored.

## Possible observation
None needed in the game: a formats spec from the file itself.

## Current workaround
`fmt::vpl` reads the file as four little-endian 32-bit header values,
a 768-byte palette and then rows of 256 entries, accepting it only when
one header value gives a row count that matches the file length exactly.
Marked provisional; files that do not match are rejected and voxels are
then drawn without the table (raw colour indices).
