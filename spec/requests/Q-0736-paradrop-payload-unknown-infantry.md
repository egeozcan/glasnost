---
id: Q-0736
title: A paradrop payload entry that names no infantry type
status: open
related: [S-0522, S-0513]
answered-by: []
---

## Need
S-0522 claim 2 sends one paradrop plane per entry of the chosen payload list, carrying *n*ᵢ
units of type *t*ᵢ. The Import mapping parses the infantry list as "list of infantry type IDs".
Neither spec says what happens when an entry names something that is not an infantry type: an
unknown name, or the name of a vehicle or aircraft type.

Our importer resolves the list once (`superweapons::SwRules::payloads`) and must decide what to
do with such an entry.

## Questions
1. Does an entry naming no infantry type still send its plane? If so, with no passengers, or with
   a fallback type?
2. Does such an entry shift the pairing of the later entries with their counts?
3. Is the answer the same for the spy plane count of S-0513 claim 43, which uses only the list's
   length?

## Possible observation
Use S-0522 scenario P2 with a first side group's list of three entries whose middle name is not
an infantry type, counts 1, 2 and 3. Record the planes that arrive and their paratroopers.

## Current workaround
Interim rule (docs/development.md, *Simulation (M6b: superweapons)*):
- The entry sends no plane and the importer warns.
- The other entries keep their own counts.
- The spy plane count still uses the list's full length.
