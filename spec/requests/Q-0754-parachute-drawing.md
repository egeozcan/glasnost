---
id: Q-0754
title: How a paratrooper's parachute is drawn
status: open
related: [S-0522, S-0109, S-0103, S-0104]
answered-by: []
---

## Need

S-0522 claim 13 attaches the parachute animation to the paratrooper, so it follows the unit down. S-0109 gives the playback: stages, passes, end and follow-up. No spec says how the attached animation is placed relative to the unit. These points are open:

- its screen offset;
- its draw order against the unit;
- whether it uses the unit's house colour.

Claim 13 says that on landing the current pass becomes the last. No spec says how that remaining pass is shown once the unit has landed. The code is the paratrooper drawing in `crates/app/src/mapview`.

## Questions

1. Where is the parachute drawn relative to the falling unit? At the unit's own screen point, or offset? If offset, by how much?
2. Is it drawn before the unit's own sprite (behind it) or after it (in front of it)?
3. Which palette and brightness does it use? The animation palette of S-0103 claim 10, or the unit's palette with house colours?
4. After landing, does the visible parachute stay where the unit landed until its last pass ends? Or does it move with the unit if the unit walks away?

## Possible observation

Take screenshots of S-0522 scenario P8 at the drop, mid-descent and at landing. Use parachute art whose frames mark their own origin, so that the offset and draw order can be read off.

## Current workaround

- The parachute is drawn at the unit's own screen point, after the unit.
- It uses the animation palette.
- It is played with the S-0109 player from its creation.
- It is drawn while the unit's parachute is attached, and it ends when the unit lands.

These choices are marked in the code with this request's ID.
