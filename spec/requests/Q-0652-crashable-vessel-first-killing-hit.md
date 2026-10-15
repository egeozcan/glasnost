---
id: Q-0652
title: Whether a crashable naval type that meets the sinking conditions at its first killing hit sinks or is removed
status: answered
related: [S-0312, S-0504]
answered-by: [S-0312]
---

## Need

S-0312 revision 22 claim 15 says: "A vehicle already sinking and whose type cannot crash stays in
play, including the zero-health aftermath of claim 32. For a crashable type, failure to start a
crash instead removes it immediately, even when it was sinking." Claim 10 says a killed vehicle
that meets the sinking conditions (naval, not underwater, not organic, heavy enough, on water,
without the bomb mark) sinks instead of exploding and stays in play at 1 health point.

For a vessel that is already sinking when it takes a killing hit, the code now follows claims 15
and 32: a crashable type is removed at the end of the hit's effects, a non-crashable one stays in
play (`combat::death::kill`). The two claims do not say what happens at the first killing hit of
a crashable naval type that meets claim 10's conditions. It is not "already sinking" before the
hit, so claim 15's first two sentences do not cover it, and it is on the water, so it is not above
the terrain and cannot start a crash. It is unclear whether claim 10's sinking keeps it in play,
or whether claim 15's crashable rule removes it at once. The gap is in `combat::death::kill`, in
the branch where `naval::sinks_at_kill` holds.

## Questions

1. A naval type with the crashable flag, otherwise meeting claim 10's sinking conditions, takes its
   first killing hit on water. Does it start sinking at 1 health point and stay in play until the
   descent of S-0504 claim 4 removes it? Or is it removed at the end of that hit's effects?
2. If it sinks, does it show claim 9's explosion and destruction animations (and take their
   draws), or none, as for a non-crashable sinking vessel?

## Possible observation

S-0312 scenario D23's crashable variant, but with the first killing hit given to a surface vessel
that meets claim 10's conditions (not a culled host): record whether it stays in play, its health,
its height over the following ticks, and the animation list picks at the hit.

## Current workaround

A crashable naval type that meets claim 10's conditions at its first killing hit sinks like any
other such vessel: 1 health point, no claim 9 animations or picks, in play until its descent ends.
The crashable removal applies only to a vessel that was already sinking before the hit (claim 15's
"even when it was sinking"). Recorded in docs/development.md, M6a naval units interim rules.
