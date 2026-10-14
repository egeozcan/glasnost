---
id: Q-0478
title: How can an empty country name prefix value give an empty prefix when an empty value is no entry?
status: answered
related: [S-0413, S-0001, S-0006]
answered-by: [S-0001, S-0413]
---

## Need
S-0413 claims 1 and 4 and its Import mapping say that a country's name prefix key with an empty
value gives an empty prefix, unlike the usual rule that an empty value keeps the value in force.
S-0001 claim 11 says a line whose value is empty after trimming creates no entry and behaves
exactly as if absent. Under that rule the importer (`convert::sim_econ`, `name_affixes`) never
sees an empty prefix value, so it cannot apply S-0413's exception.

## Questions
1. Does an empty prefix line (`Prefix=` or `Prefix=   `) in a rules layer's country section
   replace the held prefix with an empty one, while other empty-valued lines create no entry?
2. If so, which other keys, if any, share this exception to S-0001 claim 11?
3. Does an empty prefix line make an otherwise empty section exist (S-0001 claim 12)?

## Possible observation
None directly: S-0413 says the prefix has no observable effect in play. If a display uses the
prefix, a test country with a base-rules prefix and a map override `Prefix=` would show which
prefix is held.

## Current workaround
The importer follows S-0001 claim 11: an empty prefix line keeps the value in force. The prefix
has no game-state effect.
