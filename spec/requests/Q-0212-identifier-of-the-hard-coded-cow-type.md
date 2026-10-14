---
id: Q-0212
title: Which type identifier picks out the hard-coded cow type of S-0107 claim 14?
status: answered
related: [S-0107]
answered-by: [S-0107]
---

## Need
S-0107 claim 14 and `quirks.cow_idle` single out "the hard-coded cow type". The Import mapping's
*Hard-coded identifiers* entry says that claim 14 compares the type identifier case-insensitively,
but the revision of 2026-10-13 no longer gives the identifier itself. The simulation compares infantry type
names in `crates/deterministic/sim/src/infantry.rs` (the idle action) and cites that Import
mapping entry for the name. The literal in the code now has no spec text behind it.

## Questions
1. Which type identifier does claim 14 compare against, as user data spells it?
2. Is the comparison made against the type's identifier (its registry name) and not its display
   name or image name?

## Possible observation
S-0107's cow scenario (claim 14 backing) with an invented infantry type given that identifier in
a test rules file, and a copy of the stock cow renamed: only the type with the identifier should
show the cow's scatter share.

## Current workaround
The simulation keeps its current literal and comparison. The citation points at the Import
mapping entry, which no longer names the identifier. The code is unchanged until the spec names
it.
