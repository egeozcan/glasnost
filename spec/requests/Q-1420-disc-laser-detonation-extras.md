---
id: Q-1420
title: Does a disc laser's final detonation show an explosion animation or throw debris
status: answered
related: [S-0615, S-0303]
answered-by: [S-0615]
---

## Need
S-0615 claim 16 says the final step makes "a detonation at the target's target point (S-0303)".
S-0303 claims 2, 21 and 22 tie the explosion animation (F5), the nullify animation, warhead debris
and the light flash to a *projectile's* detonation, and a disc laser has no projectile
(S-0615 claim 12). The engine has to know whether the disc's detonation is area damage alone or
also shows these. This decides what a Floating-Disc-style hit looks like and, through F5's draws,
the random stream. Code: `crates/deterministic/sim/src/disc.rs`, `step`, the final step.

## Questions
1. When a disc laser's final step detonates, does the warhead's explosion animation appear at the
   target point? If it does, is it chosen as in S-0303 F5 (damage amount, the warhead's animation
   list, land type at the target point, splash list over water), and does it take F5's draws?
2. Is warhead debris (S-0303 claim 22) thrown, and does the light flash appear where configured?
3. When the area damage is nullified (S-0303 claim 15), does the nullify animation appear?
4. Does the detonation select a special hit (S-0514 claim 20), for example a takeover or a seize,
   when the disc weapon's warhead has one, or is it always ordinary area damage?

## Possible observation
A disc weapon with a warhead whose animation list has one distinctive entry and whose debris list
is non-empty, fired at a vehicle on clear land and on water; record animations, debris and the
stream position around tick T + 17 (S-0615 scenario DL1 with the stream recorder).

## Current workaround
Ordinary area damage only (`area::detonate`): no explosion animation, no debris, no flash, no
special-hit selection, so no F5 draw.
