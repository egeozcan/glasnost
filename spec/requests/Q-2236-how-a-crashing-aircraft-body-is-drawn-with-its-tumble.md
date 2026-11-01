---
id: Q-2236
title: How a crashing flying-locomotor aircraft's body is drawn with its tumble angles
status: open
related: [S-0113, S-0105, S-0106, S-0504, S-0312, S-0501]
answered-by: []
---

## Need
S-0113 claim 16 and F5 give a crashing object's tumble: S-0312 claim 16's three
draws set its roll and pitch rates, and each angle then adds its rate in every
turn, with no limit. S-0113 claim 18 says the flying locomotor draws the pitch
and roll only while the aircraft is crashing. S-0105 claim 42 gives a
non-crashing flying body's transform *Rz(θ_flight) · Ry(α) · Rx(β)* and says it
does not replace S-0312's crash attitude rules, and S-0504 F5 gives how the
driving and ship locomotors compose pitch and roll (lift, shifts, roll
outermost). No claim says how the flying locomotor composes a crashing body's
tumble angles with its heading.

M8 round 7 note 40 ("planes use the landing animation when they die ... in the
orig. game, they have a dying animation") came from our client drawing a
crashing aircraft level, since the tumble rates were not reported to the view.
The client now draws the tumble, so it needs the composition.

## Questions
1. For a crashing aircraft with the flying locomotor, in what order are the
   heading rotation, the roll and the pitch applied to the body, and about which
   axes (the body's own forward and lateral axes, as for S-0504 F5, or others)?
2. Which heading is used while it crashes: the flight drawing heading of
   S-0105 claim 3 (its aiming facing) or its body facing?
3. Is the body also lifted or shifted as S-0504 F5's lift and shifts give for
   the driving and ship locomotors?
4. Does the crashing body still take S-0105 claim 42's drawing pitch *α* or
   bank *β* on top of the tumble, or only the tumble angles?
5. Does the shadow of a crashing aircraft follow the tumble, or only its
   heading as in S-0105 claim 42 and S-0106 claim 11?
6. Is a crashing body drawn fresh every frame, or are its images reused
   (S-0105 claim 26)?

## Possible observation
S-0113 scenario RK9's aircraft run, with screenshots each tick from the crash
start: a test aircraft type with an asymmetric body model, shot down flying
straight along one map axis, then along a diagonal, recording the roll and
pitch the recorder reports beside the drawn outline and the shadow.

## Current workaround
Our design, marked `// design:` in `crates/render/src/voxel.rs`
(`crashing_body_pose`) and `crates/app/src/mapview/build.rs`: S-0504 F5's
transform with the flight drawing heading (the aiming facing), the tumble angles
only (no drawing pitch or bank), and the shadow on the heading alone, as for a
flying body. The images are drawn fresh under their exact angles, as for any
rocking body (S-0105 claim 26). Hovering and jumpjet bodies already use F5 under
the same kind of design rule (S-0113 Open questions).
