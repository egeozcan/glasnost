---
id: Q-0335
title: Does selecting another house's object take the selection response draw
status: open
related: [S-0701, S-0202]
answered-by: []
---

## Need

S-0701 claim 17 says that selecting an object takes one raw value of the unsynchronised stream
to choose the selection response. Claim 16 says only the local player's objects respond. Claim 31
says that selecting a *passive* house's object takes the raw value and then gives no response.
The spec does not say whether the draw is also taken when the local player selects an object of
an ordinary other house, such as an enemy unit clicked to inspect it. `Audio::selected` in
`glasnost_audio::events` has to decide, because the draw moves the unsynchronised stream. The
order responses and feedback chances of later events depend on that stream.

## Questions

1. When the local player selects an enemy or allied (not passive) house's object whose type has a
   selection list, is one raw value of the unsynchronised stream taken?
2. Does the answer differ when that type's selection list is empty?
3. Does it differ for an object that cannot be selected as part of a group (a building, for
   example)?

## Possible observation

Follow S-0701 scenario D3, but select an enemy unit instead of the passive house's object. Then
compare the selection response that follows for one of the player's own units with the response
in a run that leaves out the enemy selection.

## Current workaround

The draw is taken for every selected object whose type has a non-empty selection list, whoever
owns it. Only the local player's objects then respond. This is the passive-house rule of claim 31
applied to every other house. Documented in docs/development.md, "Audio (M5b)".
