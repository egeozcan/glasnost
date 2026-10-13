---
id: Q-0210
title: Which section covers which among equal nearest depths beyond F8's two cases, and in the depth-buffered mode?
status: open
related: [S-0105]
answered-by: []
---

## Need
S-0105 revision 8 turned claim 23 into an overlap priority and F8 into two bound tie cases
(A, B equal; A, B equal and greater than C). It leaves every other equal-depth configuration
open (Open questions, "Other section ties"). The engine still has to order the sections of every
part it draws. `render::voxel::section_order` keeps the exchange order of the previous revision,
which gives F8's two rows but has no citation for any other case.

Claim 24 also says that in the depth-buffered mode the sections are drawn "in exactly the reverse
of the claim 23 order (ties included)". Claim 23 no longer defines a drawing order, only an overlap
priority outside the depth-buffered mode, and F8 says its rows are "not required sorting steps".
So the depth-buffered drawing order of tied sections is defined only through F8's two rows.

## Questions
1. Three or more sections with exactly equal nearest depth, in file order A, B, C (and a group of
   four): which covers which where their pixels coincide, outside the depth-buffered mode?
2. Two separate tied groups in one part, for example A = B > C = D in file order A, B, C, D: what
   is the overlap priority, lowest to highest?
3. A tied pair after a smaller depth, for example z(A) < z(B) = z(C) in file order A, B, C: does
   B or C have the higher priority?
4. In the depth-buffered mode, with a type that has the depth option, for the configurations
   above and for F8's two rows: in what order are tied sections drawn, as seen where a voxel of
   each passes the depth test at the same comparison depth?

## Possible observation
MS-5b's named test models, extended with three and four flat sections whose transformed bounding
boxes have the same largest z, each in a different colour on the same pixels, drawn once without
and once with the depth option.

## Current workaround
`section_order` keeps the previous revision's exchange order for every tie
(`quirks.voxel_section_sort_ties` on; off: a stable sort). That order gives F8's two rows. Outside
them the result is an interim rule in `docs/development.md`, not a claim about the original. The
depth-buffered mode draws the reverse of the same order.
