---
id: Q-0695
title: When a hovering vehicle chooses the first heading point of a new move
status: answered
related: [S-0503, S-0417, S-0212]
answered-by: [S-0503]
---

## Need

S-0503 claim 5 says a hovering vehicle "chooses a new heading point when it starts a move and each
time it arrives at its heading point". Its State and ordering table has the row "moving, no heading
point | tick | ρ = 1; search may give a heading point", which makes the choice on the vehicle's
next tick, after setting ρ to 1.

Claim 4 says that the ρ = ½ which S-0417 claim 5 sets after a factory exit instruction "can
persist during the exit move" until a listed reset. The exit instruction gives the hover its
destination and then sets ρ = ½ in the factory's processing. If the first heading point is chosen
only on the hover's next tick, by the table row, that tick sets ρ = 1 first, and ρ = ½ never
reaches the exit move. The two passages only agree if the first heading point is chosen when the
move is given, before the factory sets ρ = ½.

The code in `crates/deterministic/sim/src/hover.rs` (`take`) needs the moment.

## Questions

1. When a hovering vehicle without a heading point is given a destination, does it choose its
   first heading point at once (path search and step check of claim 12 at the moment the move is
   given), or on its next tick through the "moving, no heading point" row?
2. In the factory exit of S-0417 claim 5, does the hover have a heading point when the factory
   sets ρ = ½?

## Possible observation

S-0503 scenario H8: record ρ, or the step lengths of the hover's first ticks out of the factory,
against a hover given the same destination by a person's order. Steps of half length (before a
stalled-start reset) show that ρ = ½ reached the move.

## Current workaround

The first heading point is chosen at once, when the move is given (`hover::take`, through the
same choice as an arrival), so the ρ = ½ of a factory exit persists as claim 4 says. The "moving,
no heading point" row then applies only when that choice gave no heading point. The choice is
marked in the code with this request's ID.
