---
id: Q-1307
title: What happens to the bunker link when either side leaves play by an untraced route
status: answered
related: [S-0606]
answered-by: [S-0606]
---

## Need
S-0606 claim 15 and the Edge cases leave other removal routes (scripted removal, a trigger deleting an object) untraced and prescribe neither persistence nor clean-up. The engine must not keep a link to a removed object.

## Questions
1. When a bunkered vehicle is removed by a trigger, does the bunker show the walls-down effects and accept a new vehicle?
2. When an occupied bunker is removed by a trigger, where is the vehicle left?

## Current workaround
Interim rule: the link and contact are cleared silently on both sides; the bunker goes back to clearing without effects; the vehicle is not moved.
