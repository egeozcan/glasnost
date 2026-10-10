---
id: Q-0022
title: Byte layout of the voxel lighting table file
status: open
related: [S-0105]
answered-by: []
---

## Need

S-0105 claim 38 (answering Q-0014) fixes how the voxel lighting table's
content is used: a row count *R*, a remap range, one more value, a palette
and *R* rows of 256 entries. It leaves "where each part lies in the file,
and in what order" to the public format. Neither OpenRA nor XCC, the
format sources this project may read, has a reader or a description of
this file, so the implementation still cannot locate the row count or the
rows. Code: `fmt::vpl` (`Vpl::parse`).

## Questions

1. In what order, and with what widths and byte order, does the file store
   the row count, the two remap-range values and the extra value?
2. Do the palette (768 bytes) and then the rows follow the header directly,
   with nothing in between?

## Possible observation

A stock file's length minus the header and palette is a multiple of 256;
which header position holds that multiple (and which hold the remap range)
can be read from the file without game data entering the repository, and
confirmed by a named test file with a changed row count (S-0105 scenario
MS-23).

## Current workaround

`fmt::vpl` reads four little-endian 32-bit header values, a 768-byte
palette and rows of 256 bytes. The row count is the first header value that
fits the file's length exactly, or else the largest one for which the file
ends before its last row. A file no header value fits is rejected, and the
viewer then uses an empty table (all rows zero, claim 38) with a note
citing Q-0022.
