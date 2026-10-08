# Spec requests

When an implementer session needs behaviour that no approved spec covers, or finds a spec ambiguous or contradictory, it writes a request here instead of guessing. The project lead passes requests to the spec side and, where useful, to the oracle (measurement).

## Format

One file per request: `spec/requests/Q-xxxx-short-name.md`, with the next free number.

```markdown
---
id: Q-0001
title: What happens to a reloading unit that starts moving
status: open            # open | answered | declined
related: [S-0042]       # specs this touches, if any
answered-by: []         # S- or M-IDs once answered
---

## Need
What the engine must decide, and where in the code the gap is.

## Questions
1. Concrete, behavioural, answerable by observing the game.
   (Example: "Does the reload timer keep counting while the unit moves?")

## Possible observation
Optional: a scenario that would settle it in the running game.

## Current workaround
What the code does meanwhile (usually: an explicit `todo!()` or a quirk switch
marked unspecified). Never an invented "probably right" value.
```

## Rules

- Ask **what the game does**, never how its code does it. Do not name classes, functions or data structures of the original, and do not quote numbers you remember.
- Requests are the only files under `spec/` that implementer sessions create. Once a spec answers a request, the lead sets `status: answered` and fills in `answered-by`.
