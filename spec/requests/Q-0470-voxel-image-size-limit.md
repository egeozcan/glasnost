---
id: Q-0470
title: Size limit for enlarged voxel images
status: answered
related: [S-0105]
answered-by: [S-0105]
---

## Need

S-0105 claim 22 (revision 12) enlarges a voxel part or shadow image to hold every
rendered voxel, and F6 gives the copy rectangle that reaches the screen. Nothing wraps
or clips at a 256-pixel edge any more. The copy rectangle's size follows from the
transformed section bounding boxes (F6: *w = trunc(x_max − x_min) + 8*). A model whose
bounds or HVA transforms are malformed, very large, infinite or not a number gives a
copy rectangle with a side of millions of pixels or more, or one that saturates. The
engine cannot allocate such an image. The spec gives no limit, and no rule for what is
drawn when the copy rectangle is too large to hold.

This is a rule for our engine's corrected presentation, not for the original's
behaviour (the original's 256-pixel wrap is a lead-confirmed departure).

The gap is in `crates/render/src/voxel.rs` (the image region in `draw_model` and
`draw_shadow`).

## Questions

1. Should the engine have a largest image side, and what value? Is a limit that no
   well-formed model reaches enough?
2. When a copy rectangle is larger than that limit, which pixels are kept: a window of
   the copy rectangle (and where is it placed), nothing at all (the part is not drawn),
   or something else?
3. Should the importer warn about a model whose part or shadow images would exceed the
   limit, given that claim 22's Import mapping now says no oversized-copy condition
   warns?

## Possible observation

None in the original. This is an engine rule for a lead-confirmed departure.

## Current workaround

Interim rule in `voxel.rs` (`IMAGE_LIMIT`): each image keeps at most a 4096 × 4096
window of its F6 copy rectangle, centred on the voxel-image origin shift (128, 128).
Copy-rectangle pixels outside the window are not drawn. Inside the window every pixel
is exact: depth-mode coverage still sees every voxel, including those outside the
window. `PartImage::copy_size` still reports the unclipped *w × h*. No model whose
copy rectangle fits in the window is affected.
