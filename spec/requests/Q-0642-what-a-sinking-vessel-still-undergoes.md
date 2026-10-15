---
id: Q-0642
title: What a sinking vessel still undergoes (further hits, healing, a turn under way)
status: answered
related: [S-0504, S-0312, S-0302, S-0303, S-0314]
answered-by: [S-0504]
---

## Need
S-0504 claims 4 and 5 say what a sinking vessel does and that it cannot be targeted or fire.
They do not say what other effects can still reach it while it goes down. A sinking vessel
keeps 1 health point and stays in play, so the hit rules of S-0302 and the area damage of
S-0303 would otherwise apply to it again, a second killing hit would start S-0312's death
effects a second time, and self-healing (S-0314 claims 9 and 10) would raise its health.

## Questions
1. A sinking vessel is caught in an explosion's area (S-0303). Does its health change? Can it
   be killed a second time, and if so, are death effects (debris, a death weapon, crew, the
   destruction raises and credit) repeated, and does it then leave play at once?
2. Does a direct hit on a sinking vessel (for example a projectile already in flight towards
   it when it started sinking) change its health, or show any reaction?
3. Does a sinking vessel of a self-healing type, or one near a healing source, gain health
   while it goes down?
4. A ship that is turning its body when it starts sinking: does the turn stop at once, or
   does the body finish turning to the facing it was turning to?

## Possible observation
Kill a stock surface vessel on water with a weapon whose area also covers it on the next
shots; record its health each tick while it sinks, and whether any further death effects
appear. Repeat with a self-healing vessel type. Kill a ship in the middle of a body turn and
record its facing each tick.

## Current workaround
Interim rules: a sinking vessel takes no further hit (`damage::apply` answers *unaffected*
before any reaction) and does not self-heal (`tick::object_turn` skips the healing step). A
body turn under way when sinking starts is not stopped (the body's facing keeps its own
schedule); only the locomotor stops.
