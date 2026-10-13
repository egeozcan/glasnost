---
id: Q-0050
title: The off states of two rendering switches (infantry height light, sprite vehicle facings)
status: answered
related: [S-0102, S-0104]
answered-by: [S-0102, S-0104]
---

## Need
Every quirk switch of an implemented spec should have a named field that
selects the original's behaviour (on) or the alternative the spec gives
(off). For two switches whose on state our renderer draws, the stated off
state leaves one case open, so no field is offered yet: infantry follow
the on state of `quirks.infantry_height_light_limited`
(`render::light::object_brightness`), and sprite vehicles the on state of
`quirks.sprite_vehicle_eight_facings_only` (`app::mapview::build`). Both
are listed in `app::mapview::quirks::NOT_IMPLEMENTED`.

## Questions
1. **`quirks.infantry_height_light_limited` off (S-0104 claim 13, Edge
   cases).** The off state says infantry add (*h* ÷ 208) × λ wherever they
   are, "and the shadowed-cell penalty still applies at *h* ≤ 208". For
   infantry not on a bridge deck, in a shadowed cell, with *h* = 208
   leptons exactly (so *h* ÷ 208 = 1), is the brightness *U* + extra light
   + λ − 500 (the penalty added to the height term) or *U* + extra light
   − 500 (the penalty in place of it, as in the on state's case split)?
   Below 208 leptons the two readings agree.
2. **`quirks.sprite_vehicle_eight_facings_only` off (S-0102 claim 22, Edge
   cases).** The off state uses, for a facing count *n* that is a power of
   two from 2 to 256, "the *n*-step facing of the body facing, rotated so
   that part 0 is map north-west as for 8". For 8 that is
   (*f*<sub>8</sub> + 1) mod 8, a rotation by one step (an eighth of a
   turn). For *n* ≥ 8 an eighth of a turn is *n*/8 steps. For *n* = 2 and
   *n* = 4 an eighth of a turn is half a step or a quarter of a step:
   which facing part does a body facing give then (for example: the
   *n*-step facing of the body facing turned an eighth of a turn
   clockwise, with the glossary's rounding of halves)?

## Possible observation
None: both are alternatives of our own, not behaviour of the original.

## Current workaround
No field: the on state (the original's behaviour) always applies.
