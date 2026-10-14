---
id: Q-0419
title: Which country a house defined by a map belongs to, for its inherited colour
status: open
related: [S-0103, S-0009]
answered-by: []
---

## Need
S-0103 claim 14 says a map-defined owner inherits its *country's* imported colour definition, and that its own map section can replace that choice with a colour selector. It does not say how the country of a house that the map's house list names is determined. The map viewer's all-owners view and its sandbox game (`app::mapview::build::map_owner_colours`) need that country to colour such a house when its own section has no valid colour selector.

## Questions
1. For a house named by the map's house list, which data selects its country: a key of the house's own map section, the house's name, or something else? If a key, what is it, how is its value matched to a country (letter case, length, numbers), and what applies when it is absent, empty or names no country?
2. When an object's owner is not named by the house list but matches a country's name, does the owner take that country's colour (as the sandbox game assumes, following S-0009's Import mapping that the neutral and special houses answer to their country names)?
3. When an owner is neither a listed house nor a country name, which colour does it show outside a skirmish?

## Possible observation
S-0103 scenario PM's isolated map-entry probe, extended with houses whose names differ from every country name: give each a country by the candidate data, leave its colour selector absent, and record which ramp its remap pixels follow.

## Current workaround
An owner takes the inherited colour of the country with its own name (exact match); a house the list names can replace it with its own section's valid `Color` selector. An owner with neither keeps the viewer's provisional grey and the scene note that its colour is ours. Recorded in `docs/development.md` under the open requests.
