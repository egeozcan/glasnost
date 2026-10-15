---
id: Q-0705
title: An airborne air transport's unload transitions
status: answered
related: [S-0507, S-0501]
answered-by: [S-0507]
---

## Need

S-0507 claim 13 says an air transport in *unload*, in the air with no destination, "searches for
a dock or landing spot" when its type is a dropship, and that with a destination it uses a
flight and landing approach whose "exact alternate destinations and transition conditions remain
open". Without a rule an airborne transport given the deploy order may never land.

The code in `crates/deterministic/sim/src/transport.rs` (`process_air_unload`) needs the rule.

## Questions

1. Which search does a dropship use (the landing spot of S-0501 claim 41, or another)?
2. What does a transport that is not a dropship do in the air with no destination?
3. When does an airborne transport with a destination start landing, and when does the 1-tick
   approach processing apply?

## Possible observation

S-0507 scenario T11 with the transport given the deploy order while flying, with and without a
destination, for a dropship type and a type without the flag.

## Current workaround

A dropship in the air with no destination takes the landing spot of S-0501 claim 41 as its
destination. Otherwise an airborne transport waits the standard delay while its flight goes on,
and releases passengers only once landed. The choice is marked in the code with this request's
ID.
