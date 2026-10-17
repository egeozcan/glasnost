---
id: Q-1155
title: Successors of a particle whose kind differs from its system's kind
status: answered
related: [S-0616, S-0613]
answered-by: [S-0616]
---

## Need

S-0616 claim 3 says a particle's own behaviour kind chooses its update and its move, whatever
the kind of the system that holds it, "with the system's turn order". It does not say whose rule
makes successors when such a particle dies. S-0613 claim 7 gives a gas system's dead gas
particles their successor (at the dead particle's position plus the next-particle displacement),
and S-0616 claim 8 gives a smoke system's dead smoke particles two successors (at x ± oₓ,
y ± o_y). Fire, spark and railgun turns remove dead particles without successors (claims 10, 15
and 17). The code is `gas::second_pass` (gas turns) and `particles::split_and_remove` (smoke
turns).

## Questions

1. A smoke particle with a next type dies in a gas system (for example the first particle of a
   gas-kind warhead system whose held type is a smoke type). Does it get S-0613 claim 7's one
   successor, S-0616 claim 8's two successors with their draws, or none?
2. A gas particle with a next type dies in a smoke system (a smoke-kind system whose held type
   is a gas type). Does it get claim 8's two successors, S-0613 claim 7's one, or none?
3. A smoke or gas particle with a next type dies in a fire, spark or railgun system. Are
   successors made there at all?

## Possible observation

S-0616 scenario P3 part b with test rules: a gas-kind system holding a smoke type with a short
lifetime and a next type, and a smoke-kind system holding a gas type with a next type; count the
particles after the first one dies and the raw values taken in that turn.

## Current workaround

Cited as "Q-1155 interim" in `gas.rs` and `particles.rs`: a successor rule applies only when the
dead particle's kind matches the turn's kind (S-0613 claim 7 for gas particles in gas turns,
S-0616 claim 8 for smoke particles in smoke turns); any other dead particle is removed without
successors and without draws.
