---
id: Q-0211
title: What do in-bound normal-mode-0 voxel columns draw beyond claim 35's three cases?
status: answered
related: [S-0105]
answered-by: [S-0105]
---

## Need
S-0105 revision 8 replaced claim 35's group grammar for normal mode 0 with a three-row table of
MS-19's cases and states that columns are independent. Other in-bound outputs are open (Open
questions, "Other mode-0 outputs"); they are not unspecified, because the original is
deterministic within a column's data. The revision also dropped the sentence that a mode-0 column
ends "as for the other modes", so it is no longer said whether the "Visible prefix" rule of
`quirks.voxel_topdown_column_origin` (stated for empty-run/voxel-run pairs) governs mode-0 columns.

The engine reads every mode-0 column in `render::voxel::column_steps`, with the grammar of the
previous revision. Outside the three cases that grammar has no citation.

## Questions
1. For an in-bound mode-0 column whose data is not one of claim 35's three cases, which voxels
   are drawn, at which heights, and with which colour and normal? Examples: a column with two voxel
   runs separated by an empty run, and a column whose run lengths do not add up to the section's
   height.
2. Does the "Visible prefix" rule (claim 36's complete-pair count) apply to mode-0 columns? If
   so, what is a pair in mode 0?
3. Does the top-down column origin (`quirks.voxel_topdown_column_origin`) change mode-0 columns
   the way it changes the other modes?

## Possible observation
MS-19's named test model, extended with mode-0 sections whose columns hold two voxel runs, a
column longer and one shorter than the section height, each voxel in a distinct colour.

## Current workaround
`column_steps` keeps the previous revision's mode-0 grammar, which gives claim 35's three rows.
Other columns are drawn by that grammar as an interim rule in `docs/development.md`.
