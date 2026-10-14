---
id: Q-0471
title: Is the infantry pip-colour omission shift a quirk switch or a fixed departure?
status: open
related: [S-0003, S-0006, S-0111]
answered-by: []
---

## Need
S-0006 revision 14 (claim 5, "Pip colours: a display-only departure") and S-0003 revision 17
(claim 16) say our engine keeps the held passenger pip colour and occupant pip image frame when a
layer's infantry section omits the key, with no quirk switch. S-0111 revision 7 still specifies
`quirks.pip_colour_default_shift` (Edge cases, default on, the original's shift) and its Import
mapping rows point to that switch. S-0003 asks for the two to be confirmed together. The
viewer's list of rendering quirk switches (`app::mapview::quirks`) and the pip import must follow
one of them.

## Questions
1. Is `quirks.pip_colour_default_shift` withdrawn, so the omitted-key shift is fixed with no
   switch as S-0006 claim 5 says?
2. If so, will S-0111's Edge cases and Import mapping be revised to match?

## Possible observation
None needed: this is a policy question between two specs. S-0006 scenario AA covers the
original's behaviour.

## Current workaround
Neither field is imported and the pip scale is not drawn, so no behaviour depends on the answer.
The viewer keeps S-0111's switch name in its list of unimplemented quirk switches until S-0111
changes.
