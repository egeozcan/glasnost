---
id: Q-0326
title: The rules keys of the weapon responses, the undeploy response and the death response
status: open
related: [S-0701]
answered-by: []
---

## Need

S-0701's Import mapping describes three groups of response keys without spelling them out:

- the four single weapon responses, "named like the attack list key with the weapon and rank
  inserted";
- "the undeploy counterpart" of `VoiceDeploy`;
- "the death response key".

The importer must read exact key names, because S-0001 claim 13 matches keys by exact case. So
`glasnost_audio::names` cannot read them from a description.

## Questions

1. What are the exact key names of the primary and secondary weapon responses, normal and elite?
2. What is the exact key name of the undeploy response?
3. What is the exact key name of the death response list of claim 20?

## Current workaround

None of these keys is read. As a result:

- an attack order falls back to the attack list (F6);
- an unload or deploy order of deployed infantry plays nothing;
- a death plays only the type's death sound list (`DieSound`), never a death response.

Documented in docs/development.md, "Audio".
