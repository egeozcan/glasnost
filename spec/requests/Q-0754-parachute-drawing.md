---
id: Q-0754
title: How a paratrooper's parachute is drawn
status: answered
related: [S-0522, S-0109, S-0103, S-0104]
answered-by: [S-0522]
---

## Need

S-0522 claim 13 attaches the parachute animation to the paratrooper. S-0109 claim 22 says how an attached animation is placed: at its owner's position plus the displacement at attachment (none here), in the ground render layer, and it disappears at once when its owner is removed. Two points are still open:

- its draw order against the paratrooper itself, since both are in the ground layer;
- whether it uses the unit's house colour.

The code is the attached-animation drawing in `crates/app/src/mapview/live.rs`.

## Questions

1. Within the ground layer, is the parachute drawn before the paratrooper's own sprite (behind it) or after it (in front of it)?
2. Which palette and brightness does it use? The animation palette of S-0103 claim 10, or the unit's palette with house colours?

## Possible observation

Take screenshots of S-0522 scenario P8 at mid-descent, with parachute art whose frames overlap the paratrooper's sprite and use house-colour indices.

## Current workaround

- The parachute is drawn with the ordinary animation ordering of the ground layer.
- It uses the animation palette.
- It follows its paratrooper as S-0109 claim 22 says, and when the paratrooper lands its current pass becomes the last (S-0522 claim 13).

These choices are marked in the code with this request's ID.
