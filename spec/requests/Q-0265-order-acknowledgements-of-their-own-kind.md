---
id: Q-0265
title: Which spoken acknowledgement does each order kind with an acknowledgement of its own play, and what plays when an object is selected?
status: open
related: [S-0214, S-0202]
answered-by: []
---

## Need

S-0214 claim 14 says that harvest, attack, move and attack move, enter, capture and unload orders
have acknowledgements of their own kind, and that every other order is acknowledged with one of the
type's selection responses (F5). Its Import mapping gives only `VoiceSelect`. S-0214 Terms says
selecting an object also plays a selection response, which that spec does not specify.
`glasnost_audio::Audio::acknowledge` needs both.

## Questions

1. For each of the kinds harvest, attack, move, attack move, enter, capture and unload, which list
   of the type supplies the acknowledgement (the key the importer reads), and what plays when that
   list is empty (nothing, or a selection response)?
2. Is the response chosen as in F5 (one raw value of the unsynchronised stream, *i* = *x* mod *n*)?
3. When the local player selects objects (a click or a band), which object speaks, is the response
   chosen as in F5, and is anything played when several objects are selected at once or when the
   same object is selected again?

## Possible observation

S-0214 Scenario T2 with test types whose lists are given distinct sounds, and with the random-state
recorder reading the unsynchronised stream.

## Current workaround

Orders of the seven kinds play no acknowledgement and take no draw. Selecting an object plays
nothing. Every other order kind is acknowledged per S-0214 claim 14 and F5. Documented in
docs/development.md, "Audio (M5b)".
