---
id: Q-2983
title: How are the turret and barrel of an aircraft with the turret flag posed and reused under pitch, bank or tumble?
status: answered
related: [S-0105, S-0113]
answered-by: [S-0105]
---

## Need

S-0105 claim 7 says *D* is "the body transform its locomotor gives". Claims 8 and 10 build the turret and barrel on it: *T = D · X(d) · Rz(θ_rel)* and *B = T · Ry(φ)*. Claims 41 and 42, and S-0113 claim 20, give *D* for a rocket's pitch, a flying body's pitch and bank, and a crashing body's tumble. None of them says anything about a turret or barrel part. Claim 26 gives the body keys under these attitudes but not the turret or barrel keys.

Render issue #82 (GitHub, render audit #154) found that the client drew the turret and barrel of an aircraft type with the turret flag with the body's own transform. It kept them under the level turret key, so a banked image was shown again when the aircraft flew level, and the reverse. The client needs to know the intended pose and reuse.

## Questions

1. Does claim 8 apply to an aircraft or rocket type with the turret flag, with *D* the transform of claim 41, claim 42 or S-0113 claim 20? Or is a turret or barrel part drawn some other way under these attitudes, or not drawn?
2. A flying body is drawn on the flight drawing heading, which is its aiming facing (claim 42), not its body facing. What is *θ_rel* for its turret then: the turret facing less the drawn heading (always 0 when the turret facing is the aiming facing), the turret facing less the body facing, or something else? The same question applies to a crashing flying body under S-0113 claim 20.
3. Under each of these attitudes, which components does claim 26's turret key hold, and which does its barrel key hold? Is a turret or barrel image reused across different pitch, bank or tumble states? Or is it drawn fresh, as a crashing body is?
4. Does claim 11's turret-first test use the turret facing alone under these attitudes, as on flat ground?

## Possible observation

Give an aircraft type with the flying locomotor the turret flag, and give it turret and barrel models (test data only). Set a nonzero drawing pitch and bank angle. Capture it flying level, then turning (banked), then flying level again. Compare the turret and barrel pixels with the body's in each state, and with the first level capture. Repeat with the turret facing turned away from the aiming facing, if the game allows that for the type.

## Current workaround

The client does the following, marked `design: render (#82)` in `crates/app/src/mapview/build.rs` (`vehicle_voxels`) and `crates/render/src/voxel.rs` (`turret_on_body`):

- **Pose.** It builds *T* and *B* on the attitude's *D* as claims 8 and 10 do, and takes *θ_rel* against the facing the body is drawn with:
  - for a flying or crashing body, the heading, so the turret faces it;
  - for a rocket, the body facing.
- **Reuse.** It keeps the turret and barrel images apart under the F1 table steps of the attitude angles:
  - for a flying body, the pitch and bank steps;
  - for a rocket, the drawn pitch's step;
  - for a crashing body, the tumble steps.

  The turret key takes the drawn heading in place of the body facing.

The interim is recorded in `docs/m8-claims/render-issues.md` (batch R3, #82).
