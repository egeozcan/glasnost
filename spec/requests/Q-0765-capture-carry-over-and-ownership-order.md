---
id: Q-0765
title: Order of tag carry-over and ownership change in an engineer capture
status: open
related: [S-0524, S-0509, S-0011]
answered-by: []
---

## Need
S-0524 claim 10 says both the tag carry-over and the ownership change follow the two entry raises and precede the engineer's destruction raise, but prescribes no order between them. The capture seam `capture_seam` in `crates/deterministic/sim/src/mp_triggers.rs` must choose one for S-0509's capture when it is built. An order is observable if the ownership change itself raises a tag (for example a house-owned trigger event checked during the change) or if the building's old tag reacts to the change.

## Questions
1. When an engineer whose tag has the carry-over flag captures a tagged building, does the building already carry the engineer's tag when its owner changes?
2. Does the ownership change itself raise any event on the building's tag (old or new), and which tag receives it?
3. With the carry-over flag clear, does the building's old tag see the ownership change?

## Possible observation
Give the building's tag a trigger whose event fires on a change of owner (if the game has one) and the engineer's tag a different trigger on the same event. Capture with and without the carry-over flag and record which counter fires.

## Current workaround
Carry-over first, then the ownership change.
