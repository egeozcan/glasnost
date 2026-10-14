---
id: Q-0216
title: Where a top-down normal-mode-0 span's empty count lands, against its colours
status: open
related: [S-0105]
answered-by: []
---

## Need

S-0105 claim 35 (revision 9) gives two normative statements for top-down
columns of normal mode 0 that cannot both hold. `render::voxel::column_steps`
has to place the colours of such a column.

- The claim's output table, row 3 (MS-19 (b), upside down): the standard
  column of *e* empty cells and voxels (colour *e*, normal *Z*), (colour *X*,
  normal *Y*) in a grid of height *e* + 2 shows *X* at cell *e* and *Y* at
  cell *e* + 1. Its forward value sequence is *e, 2, e, Z, X, Y, 2*.
- The colour-height relation: top-down a span is *(e, b, aₙ₋₁ … a₀, n)* in
  forward file order, numbered from the column's top end, and colour *aₗ*
  lies at traversal distance *Cᵣ₋₁ + eᵣ + l* from the far face, at grid cell
  *Nz − 1 − d*. For the same column the last five values give *e* (the
  colour value *e*), *b = Z*, *a₁ = X*, *a₀ = Y*, *n = 2*, so *Y* is at cell
  1 and *X* at cell 0.

MS-26's second row (invented values) agrees with the relation and not with a
reading that matches the table: forward sequence 1, 91, 21, 20, 2, 0, 90,
11, 10, 2, *Nz* = 5, top-down output cells 4, 3, 1, 0 showing 10, 11, 20,
21. A reading that applies each span's empty count after its colours (which
gives the table's row 3) puts 20 and 21 at cells 2 and 1.

Every other vector of the claim and of MS-26 (rows 1 and 3 to 6, both
poses where given) comes out the same under either reading.

## Questions

1. Top-down, in normal mode 0, does a span's empty count move the
   following colours before they are placed (the relation), or does it move
   only the colours of the spans after it (table row 3)?
2. If the answer differs between the topmost span and later spans (one
   rule that fits every vector given is: the topmost span's empty count is
   applied after its colours, every later span's before its colours), please
   state the rule for any number of spans.
3. If table row 3 is the one in error, what does MS-19 (b) show upside down?

## Possible observation

MS-19 (b) upside down and MS-26's second row top-down, captured on the
same build; the cell positions of *X*, *Y*, 20 and 21 settle the question.

## Current workaround

`column_steps` keeps the previous revision's top-down grouping, which
applies each span's empty count after its colours. It meets claim 35's table,
MS-26 rows 1 and 3 to 6, and the 260-colour column; it puts MS-26 row 2's
second span (20, 21) at cells 2 and 1 instead of 1 and 0. The placeholder is
listed in docs/development.md under *Open requests and their placeholders*.
