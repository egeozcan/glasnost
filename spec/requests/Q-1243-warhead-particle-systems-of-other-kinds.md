---
id: Q-1243
title: What a warhead-created particle system of a non-gas kind does after its first particle
status: open
related: [S-0613, S-0303, S-0203]
answered-by: []
---

## Need

S-0303 claim 20 and S-0613 claim 4 say every warhead that names a particle system creates it at
the detonation point, owned by the source house, and that it receives one particle of its held
type at once, for every system behaviour kind. S-0613 then covers only the gas kind: a gas
system with unlimited lifetime stays in play, inert, after its last particle is gone. For the
smoke, fire, spark and railgun kinds, "other system behaviour kinds may emit further particles
and are outside this spec". No approved spec says how those systems update, how their particles
move and die, or when the systems leave play. Kept for ever with lifetime −1, every hit of
such a warhead would add one more object to the update order.

The code is `crates/deterministic/sim/src/gas.rs` (`warhead_system`), called from
`crates/deterministic/sim/src/combat/area.rs` (`detonate`).

## Questions

1. Does a smoke, fire, spark or railgun particle system created by a warhead take turns, and if
   so, what does its turn do to its particles (movement, lifetime, removal)?
2. When does such a system leave play: when its particles are gone, when its system lifetime
   runs out, or never?
3. Does a particle system with no behaviour kind created by a warhead stay in play for ever?

## Possible observation

G3 with the system kind varied (S-0613's G3-system test file): record system existence and
particle counts until each system disappears.

## Current workaround

Only gas systems (and systems with no behaviour kind, which stay inert) are kept as state. For
another kind the first particle's creation draws are taken (S-0202 claim 15), and neither the
system nor the particle is kept, so they take no object numbers and no turns. A note names the
seam. Docs: docs/development.md, *Simulation (M7a: gas clouds)*.
