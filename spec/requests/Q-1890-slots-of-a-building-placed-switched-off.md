---
id: Q-1890
title: Slot animations of a building the map places switched off, and same-tick disablings
status: answered
related: [S-0108, S-0405, S-0009]
answered-by: [S-0108]
---

## Need

S-0108 r75 now says which events disable and re-enable a building in claim 13's sense:
switching off and on (claim 28), a pulse and its end (claims 13 and 28), the map's placement
of a building that needs an engineer and is not manned (claim 26), and every ownership change
of such a building (claim 26). A disabling freezes every running *freeze-unpowered* slot
whether or not the type needs power; a start while disabled freezes such a slot only when the
type needs power (claim 13).

The renderer (`crates/app/src/mapview/slot_play.rs`, `Slots::update` and `Slots::enablings`)
plays these rules. Two cases have no rule:

- A building that the map places switched off (S-0009 claim 40, S-0405 claim 18: the
  switched-on flag 0 on a type with a base drain above 0 or the needs-power flag). It was never
  switched off in play, so claim 28's "switching off disables" does not obviously apply, and
  S-0009 claim 40 hands the slot consequences to S-0108 claim 13, which does not name this
  case. It matters for a type with a base drain above 0 and no needs-power flag: claim 13's
  start rule does not freeze its set-up slots, so whether they hold depends on whether the
  placement counts as a disabling.
- Two of these events on the same building in the same tick, one disabling and one
  re-enabling: for example a switch-off on the tick of an ownership change, or a pulse that
  starts or ends on that tick. The result depends on the order in which they apply.

## Questions

1. A building of a type with a base drain above 0 and no needs-power flag, placed by the map
   with the switched-on flag 0 (so it starts switched off): do its *idle* and *active-1* to
   *active-4* slots, counting *freeze-unpowered*, hold the image frame they were created with
   from its placement on, or do they play? When it is later switched on, does anything change
   for them?
2. The same for a type with the needs-power flag (claim 13's start rule already freezes its
   set-up slots while it is not in operation): when it is switched on in a house with low power,
   so that it is still not in operation, do its slots resume (as a re-enabling would, with
   `quirks.enable_unfreezes_unpowered`), or stay frozen?
3. When a building is disabled and re-enabled in the same tick (a switch-off on the tick of its
   ownership change; a pulse that ends on the tick it is switched off; a pulse that starts on the
   tick of an ownership change), are its *freeze-unpowered* slots frozen or playing after that
   tick?

## Possible observation

A skirmish map with a building of a draining type without the needs-power flag placed with the
switched-on flag 0, and one with the flag set, both with a looping *active-1* slot animation:
record the slot's image frame on each of the first 60 ticks, then switch both on and record 60
more. For question 3, switch off a building that needs an engineer on the tick an engineer
captures it, and record its slot's frame for 30 ticks after.

## Current workaround

Interim, marked in `Slots::update` as Q-1890's: a building first seen switched off or under a
pulse counts as disabled from then on, so its running *freeze-unpowered* slots hold, whatever
its type; its later switch-on is a re-enabling. Same-tick events apply in a fixed placeholder
order (`Slots::enablings`): the ownership change, then the switch, then the pulse.
