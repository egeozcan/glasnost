---
id: Q-0751
title: What a carrying aircraft's drop shot does to its rearm, burst and ammunition
status: answered
related: [S-0522, S-0501, S-0301]
answered-by: [S-0522]
---

## Need

S-0522 claim 11 and S-0501 claim 20 make a shot by an aircraft that carries passengers a drop attempt with no projectile. A successful drop clears the rearm wait and resets the pass counter. The spec does not say what else of the ordinary shot (S-0301) still happens. These points are open:

- the rearm countdown after a failed drop;
- the burst counter;
- the void-shot rule of S-0301 claim 15;
- S-0501 claim 15's pending round;
- the shot's report sound and muzzle animation;
- any draws for the delay.

The code is in `shot` in `crates/deterministic/sim/src/combat/fire.rs`.

## Questions

1. After a refused drop shot, is the rearm countdown set as for an ordinary shot, with its R(0, 2) full-delay draw? Or is it left as it was?
2. Does a drop shot advance the burst counter?
3. Does it count as a fired run for S-0501 claim 15's pending round, taking one round?
4. Does it play the weapon's report sound or muzzle animation?
5. When the drop succeeds, is the rearm wait cleared after the ordinary delay has been set, with its draws taken? Or is no delay set at all?

## Possible observation

Use S-0522 scenario P7: an armed aircraft type that carries passengers, ordered to attack a cell. Run it twice, once over a free land cell (the drop succeeds) and once over water (the drop is refused). For each shot, record:

- the tick;
- the rearm countdown;
- the burst counter;
- the ammunition;
- the random state.

## Current workaround

- A drop shot takes the place of the projectile launch only.
- The rest of the shot runs as usual: the spray draw, the delay and its draws, the burst counter and the report effect.
- A drop shot is never void.
- A successful drop then sets the rearm countdown to the current tick.

These choices are marked in the code with this request's ID.
