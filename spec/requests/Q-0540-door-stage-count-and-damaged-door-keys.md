---
id: Q-0540
title: Which art keys give a building's door-stage count and its damaged-door flag?
status: answered
related: [S-0108, S-0417, S-0102]
answered-by: [S-0108]
---

## Need

S-0108 claim 23 and Formulas F8 compute the door image's frame from the door state, the door
progress *q* and the door-stage count *D*, and add *D* for a damaged damaged-door type. The
Import mapping's building art keys table names `DoorAnim` for the door sprite, but its rows for
the door-stage count and for "whether the type uses a separate door image range when damaged"
give no key name, only a form and an absent-key default (integer 0; boolean false). The
converter (`crates/convert/src/art.rs`, building art) cannot read either value, so every
building gets *D* = 0 and no damaged-door flag. With *D* = 0 claim 23 requests frame 0 in every
door state, so the door image does not move while the factory opens and closes its door.

## Questions

1. Under which key, in which art section, does a building type's door-stage count appear? Is
   its form a plain integer read as S-0003 reads integers?
2. Under which key does a type mark that its door has a separate image range when damaged? Is
   it a boolean read as S-0003 reads booleans?
3. If either value is not an art key at all (for example a fixed count, or a value derived from
   the door sprite's image-frame count), what gives it?

## Possible observation

S-0108 scenario B10 with a test art section that sets a candidate key to a known stage count on
a factory whose door sprite has distinct frames, recording the door frame drawn at each tick of
an unload.

## Current workaround

Neither value is imported. Every building has *D* = 0 and is not a damaged-door type, which are
the Import mapping's absent-key defaults, so the door image is drawn with frame 0 throughout
unloading (claim 23 with *D* = 0). The deploying, roof-deploying and under-door sprites of
claim 22 are drawn as specified.
