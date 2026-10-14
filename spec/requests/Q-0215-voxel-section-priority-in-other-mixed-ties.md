---
id: Q-0215
title: Voxel section overlap priority for mixed-depth ties outside F8's table
status: open
related: [S-0105]
answered-by: []
---

## Need

S-0105 F8 (revision 9, answering Q-0210) fixes the painter priority and
the depth-mode contributing order of a part's sections for six file-order
and nearest-depth configurations, and for a part whose sections all share
one nearest depth. It says that other mixed-depth tie configurations
remain open and that a stable tie rule must not be inferred from the table
(S-0105 Open questions, *Other section ties*). Our renderer has to order
every part's sections, so `render::voxel::section_order` needs a rule for
the remaining configurations, under `quirks.voxel_section_sort_ties` on.

## Questions

Names stand for sections in file order, as in F8. For each configuration,
which section wins a pixel covered by all of them in the normal mode
(painter priority, lowest to highest), and which in the depth-buffered
mode at one equal comparison depth (contributing order)?

1. A, B, C with A > B = C.
2. A, B, C with A = C > B.
3. A, B, C with A = C < B.
4. A, B, C with A = B < C.
5. A, B, C, D with A = B = C > D, and with A > B = C = D.
6. A, B, C, D with A = C > B = D, and with A = D > B = C.
7. Is there one rule, stated over file order and nearest depths, that gives
   F8's six rows, its all-equal case and all of the above, for any number of
   sections? If the outcome depends on more than the file order and the
   relation between depths (for example on how many sections lie between
   the tied ones), what else does it depend on?

## Possible observation

MS-25's setup with test bodies of three and four sections whose nearest
depths are arranged as listed, each section a different colour, covering a
common pixel; one capture per configuration in the normal mode and one with
the depth-buffer option.

## Current workaround

`section_order` keeps the exchange order of the previous S-0105 revision,
which gives all six F8 rows and every all-equal part, and an unconfirmed
order for the configurations above. The depth-buffered mode draws its
reverse. The placeholder is listed in docs/development.md under *Open
requests and their placeholders*.
