---
id: Q-1941
title: Do claim 48 (c)'s height-0 exemptions from guidance's distance bands and close-approach trigger apply to any aircraft in flight at height 0, or only to the low release?
status: answered
related: [S-0501]
answered-by: [S-0501]
---

## Need

S-0501 revision 59, claim 48 (c), *At height 0*, says of the low-released aircraft that
"Guidance's distance bands and its close-approach trigger (claims 5 and 6) do not apply at
height 0: claim 6's close-approach trigger is a request of guidance, which claim 42 confines to
an aircraft above the terrain."

The subject of *At height 0* is the low release. The reason it gives is general, though, and
claims 5 and 6 themselves carry no height condition:
- Claim 5's approach target replacement applies to "a living aircraft flying toward a
  destination, neither landing nor taking off", with no height test.
- Claim 6's close-approach trigger has no height test either.
- Claim 42 confines guidance to an aircraft above the terrain, for every aircraft.

An ordinary aircraft can also fly at height 0 without landing. Claim 6's holding-height approach
sets the held height, below 768 leptons from the destination, to the destination's height minus
the terrain height there. For an attacker with ammunition left, closing on a target that stands on
level ground, that held height is 0. It has the holding-height mark, so the close-approach trigger
does not apply to it. But claim 5's replacement (0, 0.5 or 0.75) would, unless the reason in
claim 48 (c) carries over to it.

The engine has to decide whether the exemptions belong to the low release only or to any aircraft
at height 0. This changes the target fraction such an attacker holds at the end of each movement
tick, which is part of its state.

## Questions

1. For an aircraft in ordinary flight (not the low release) at height 0 with a destination less
   than 768 leptons away, neither landing nor taking off, does claim 5's approach target
   replacement apply?
2. For such an aircraft without the holding-height mark, does claim 6's close-approach trigger
   apply at height 0?
3. Should claims 5 and 6 state the height condition, or should claim 48 (c) say that the
   exemptions are the low release's only?

## Possible observation

AC24's attacker, which closes on a stationary enemy vehicle on level ground. Log its height
together with the target fraction at the end of each tick inside 768 leptons. If the height reaches
0 there, see whether the logged replacement (0, 0.5 or 0.75) still appears at the end of those
ticks.

## Current workaround

The exemptions are the low release's only, which is the plain reading of claim 48 (c)'s subject.
This is our design until this is answered. Claims 5, 6 and 8 apply as written to every other
aircraft, at any height. AC24 expects logged replacements for an ordinary attacker inside
768 leptons, and this reading keeps them.
Code: `low_release_at_height_0` in `crates/deterministic/sim/src/air.rs` (the low release while it
stays at height 0). The determinism gate's goldens, which include such an attacker, are unchanged.
