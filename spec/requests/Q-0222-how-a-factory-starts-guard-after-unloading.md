---
id: Q-0222
title: How a factory's queued guard becomes current when its unloading ends
status: open
related: [S-0417, S-0215]
answered-by: []
---

## Need

S-0417 ends a ground factory's unloading in the closing stage: the factory falls back to rest
and queues *guard*, and "unloading ends when *guard* starts" (claim 2, table S1). A naval
factory's second processing also queues *guard* (claim 9). S-0215 claim 11 makes a building's
queued behaviour current only at a per-tick start that finds the building's ready mark set, and
says that an unarmed building's own *guard* processing never sets it (S-0215 Open questions:
"what makes a queued behaviour of an unarmed building current is not established"). Factories
are usually unarmed. S-0417 does not say that the closing stage sets the ready mark, or that
*unload* sets it.

If nothing sets the mark, the factory stays in *unload* for good, every later release passes to
another factory or comes back busy, and vehicle production stops.

S-0406 claim 15 says the analogous thing for construction: the queued *guard* becomes current
at the building's second per-tick start of the same tick.

## Questions

1. On the tick a factory's closing stage (or a naval factory's second unloading processing)
   queues *guard*, when does that *guard* become current: at the second per-tick start of the
   same tick, at the next tick's first start, or later?
2. Does any other factory processing (an earlier unloading stage) set or clear the ready mark?

## Possible observation

S-0417 scenario U2 with a recorder of the factory's current and queued behaviour per tick:
the tick the closing stage finds the door closed, and the first tick *guard* is current.

## Current workaround

The processing that queues *guard* also sets the ready mark, so *guard* starts at the second
per-tick start of the same tick, as S-0406 claim 15 states for construction. Interim row in
`docs/development.md` (Q-0222).
