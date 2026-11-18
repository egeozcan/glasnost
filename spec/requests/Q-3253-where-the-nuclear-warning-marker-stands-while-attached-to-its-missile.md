---
id: Q-3253
title: Where the nuclear warning marker stands while it is attached to its missile
status: answered
related: [S-0513, S-0109]
answered-by: [S-0513]
---

## Need

S-0513 claim 11 places the warning-marker animation "at the target cell's centre, attached to a missile": in a silo launch to the upward missile, in a one-time launch to the falling missile. It ends when that missile leaves play.

S-0109 claim 22 says an attached animation's position "is the owner's position plus the displacement it had from the owner when it was attached, in leptons and including height, so it follows the owner".

Read together, the marker would move with its missile:

- **One-time launch.** The falling missile starts high above the target (S-0513 claim 10) and falls straight down. Its displacement from the target centre at attachment is mostly height. The marker would then start at the target centre and sink below the ground as the missile falls, by as much as the missile's whole fall.
- **Silo launch.** The upward missile starts at the silo and rises. The marker would rise with it, offset by the distance from the silo to the target.

Neither seems to be what "at the target cell's centre" means. But claim 11 does not say the marker stays there while attached, and S-0109 claim 22 does not exempt it. The marker's position is game state in our engine (it is saved and hashed; S-0109 claim 28 gives an animation scenario-stream draws while it plays). It also decides where the marker is drawn.

## Questions

1. While the warning marker is attached to its missile, does it stay at the target cell's centre? Or does it follow the missile by S-0109 claim 22, keeping the displacement it had when it was attached?
2. If it follows the missile, at what moment is the displacement taken? In a silo launch, is it taken at the silo's first update, when the upward missile has just been placed?
3. Is the marker in the ground render layer, as S-0109 claim 22 says of attached animations, or in its type's own layer?

## Possible observation

Scenario SW4 with one more record: the marker's drawn position at the end of each tick from its creation until its missile leaves play. Record it once for a one-time launch and once for a silo launch whose silo stands a few cells from the target.

## Current workaround

Our engine's interim rule, until this is answered:

- The marker is tied to its missile for its lifetime only. It is created at the target cell's centre, at its height, and stays there.
- At its first update after its missile has left play, it is deleted, and that update does nothing else (S-0109 claim 22's removal rule).
- It does not take the missile's position.
