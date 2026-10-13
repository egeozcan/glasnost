---
id: Q-0111
title: The shared state of non-legal cells during a whole-map re-cover or spy-satellite reveal
status: answered
related: [S-0409]
answered-by: [S-0409]
---

## Need
S-0409 claim 15 has the whole-map re-cover and the spy satellite's reveal
take back every re-applied sight and every hostile gap field (S1 rows 2 and
5), reset the legal cells, and apply them again (rows 1 and 3). With
`quirks.offmap_cells_shared` (claim 14) a gap field whose cells include
non-legal cells also counts on the one shared state of the non-legal cells,
which "a whole-map re-cover … resets legal cells only". Taking back a field
with row 5 while the spy satellite is active lowers the shared count and can
make the shared state explored for good (claim 14); re-applying it with row
3 raises it again. The spec does not say whether the shared state takes part
in the taking back and re-applying of a whole-map operation, or whether the
satellite counts as active during the taking back of the reveal that its
activation causes. `econ::shroud` (the whole-map operation) needs a rule.

## Questions
1. During a whole-map re-cover or reveal, are the non-legal cells of the
   gap fields taken back and applied again on the shared state (rows 5 and
   3), or left alone?
2. If they take part: during the reveal that the satellite's activation
   causes, does row 5 already see the satellite as active?

## Possible observation
S-0409 scenario V8 part H, with the generator still running while the
satellite is gained and then lost, reading the border pieces afterwards.

## Current workaround
The shared state is left out of whole-map operations: it is neither taken
back, nor reset, nor applied again. Only a field that really starts or stops
changes it.
