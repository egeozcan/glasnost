---
id: Q-0796
title: When a cloak generator's growth or shrinking becomes pending
status: answered
related: [S-0516]
---

## Need

S-0516 revision 7 (claim 20) defines an idle generator as one with neither growth nor
shrinking pending. The restart eligibility table depends on that state. The code in
`crates/deterministic/sim/src/cloak/field.rs` (`turn`, `restart_neighbours`) records the
state per generator (`cloak::FieldPhase`). The claim states these transitions:

- growth on placement;
- growth after a restart;
- growth ends at the update after the full radius is reached;
- shrinking when the generator goes out of operation;
- shrinking ends at the update after radius 0.

It does not state the others.

## Questions

1. A generator goes out of operation while its field is still at radius 0 (placed
   unpowered, say). Is shrinking pending, growth still pending, or neither?
2. A shrinking generator comes back into operation. The table says it stays ineligible
   while shrinking. Does it shrink on to radius 0 and its stop update before growing? Or
   does growth become pending at once, or at some later update, from its current radius?
3. A generator is idle, in operation and below its full radius (after a growth cycle that
   ended early, or after its type radius grew). Does growth become pending by itself? If
   so, on which update? Or does it stay idle until a restart?
4. A zero-radius generator is removed, sold, destroyed or captured. Does its immediate
   collapse perform the neighbour-restart test with *r* = 0? That is reach
   Δ*X*² + Δ*Y*² < 16. Or does a zero type radius never initiate a restart? The claim
   excludes only "by shrinking".

## Possible observation

- One generator placed with low power. Restore the power at once, then again after its
  field would have reached its full radius. Record the radius and covered cells per tick.
- A generator cut from power for two ticks, then powered again. Record its radius per tick
  and whether a neighbour's immediate collapse restarts it during those ticks.
- A zero-radius generator sold beside an idle generator at full radius 3 cells away.
  Record the neighbour's radius before and after.

## Current workaround

1. Neither: the generator is idle at radius 0.
2. It shrinks on to radius 0 and its stop update, then is idle. Growth then becomes
   pending as in 3.
3. Growth becomes pending at its next update.
4. A zero-radius generator's immediate collapse initiates no restart.

These choices are marked in the code with this request's ID.
