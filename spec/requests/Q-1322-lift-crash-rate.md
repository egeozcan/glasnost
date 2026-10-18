---
id: Q-1322
title: Starting crash rate of a dropped unit, and drop before growth
status: open
related: [S-0607, S-0502]
answered-by: []
---

## Need
S-0502 claim 18 says a magnetic-lift fall has "a growing crash rate". The engine needs the first value and the order of the drop and the growth within one update.

## Questions
1. Is the first rate the type's own jumpjet crash value (the S-0502 import mapping), or a fixed starting value?
2. In each update, is the height drop taken with the rate before it grows by 1, or after?

## Possible observation
Record a dropped unit's height every tick from the start of its fall on clear land (scenario L4).

## Current workaround
The rate starts at the type's crash value, narrowed to single precision. Each update drops by the current rate and then adds 1 to it.
