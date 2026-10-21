---
id: Q-1562
title: How a particle type's colour list reads a malformed triple
status: answered
related: [S-0202, S-0616]
answered-by: [S-0202]
---

## Need

S-0616 claim 29 and F16 now use every entry of a particle type's
`ColorList`, not only their number. S-0202's Import mapping gives the
form ("comma-separated parenthesised RGB triples, at most 511 text
bytes; integer components reduced modulo 256"), but not how a group
that is not a clean triple is read. The importer has to turn each group
into three colour components.

## Questions

1. A group with fewer than three components, such as `(10,20)`: is it an
   entry? If so, what are its missing components?
2. A group with more than three components, such as `(1,2,3,4)`: are the
   extra components ignored, or is the group not an entry?
3. A component that is not an integer, such as `(1,x,3)` or `( ,2,3)`:
   is it 0, does it keep a previous value, or is the group dropped?
4. Text outside the parentheses, or an unclosed final group such as
   `(1,2,3),(4,5`: is the unclosed group an entry?
5. Is a component read with leading spaces, a sign or trailing text
   (`( 7,+8,9z)`) as the integer reader of S-0003 claim 3 reads it?

## Possible observation

Give a spark particle type each malformed list in turn, with the two
start colours black (so F16 takes list entry 0 as the particle's own
colour), and look at the colour of its pixels and how many colour steps
it passes through.

## Current workaround

`glasnost_convert::sim::colour_list`: every closed parenthesised group
is one entry. Its first three comma-separated parts are the components,
each read by the S-0003 integer reader and reduced modulo 256. A part
that is missing or is not an integer is 0. Parts after the third are
ignored, and an unclosed group is not an entry.
