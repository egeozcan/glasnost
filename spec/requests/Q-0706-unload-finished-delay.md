---
id: Q-0706
title: The delay of a vehicle transport's finished unload phase
status: answered
related: [S-0507, S-0215]
answered-by: [S-0507]
---

## Need

S-0507 table U gives the delays of the settling, turning and letting-out phases of a vehicle
transport's *unload*, and says the finished phase queues *guard* with the allowance to interrupt
movement, but gives no delay for that processing.

The code in `crates/deterministic/sim/src/transport.rs` (`process_unload`) needs the rule.

## Questions

1. With which delay does the finished processing end: the standard delay of S-0215 claim 6, 1
   tick, or another?

## Possible observation

S-0507 scenario T10: the draw log and the next processing tick after the last passenger is out.

## Current workaround

The standard delay (S-0215 claim 6). The choice is marked in the code with this request's ID.
