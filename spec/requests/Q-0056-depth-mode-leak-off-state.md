---
id: Q-0056
title: The off state of the depth-mode leak switch
status: answered
related: [S-0105]
answered-by: [S-0105]
---

## Need
Every quirk switch of an implemented spec should have a named field that
selects the original's behaviour (on) or the alternative the spec gives
(off). Our renderer now keeps the depth-buffered mode between fresh voxel
draws as S-0105 claim 32 and state table S2 give it
(`render::voxel::VoxelCanvas`), so the on state of
`quirks.depth_mode_leak` is drawn. The switch has no field yet, because
S-0105 offers two off states and does not say which one the switch
selects (S-0105, Edge cases and *Questions for the project lead*). It is
listed in `app::mapview::quirks::NOT_IMPLEMENTED`.

## Questions
1. **Which off state?** With `quirks.depth_mode_leak` off, does a fresh
   draw of a depth-option type's shadow
   (a) switch the depth-buffered mode on as in S2 and then off again once
   the shadow is finished ("switch the mode off after every shadow"), or
   (b) leave the mode as it is ("never switch it on for shadows")?
2. **What the shadow's own draw clears.** The two readings differ beyond
   the leak itself, through claim 34. Under (a) the shadow of a
   depth-option type is made with the mode on, so it sets the depth plane
   to 0 over its clear area. Under (b) it is made with the mode off and
   leaves the depth plane as it is, unless a window is already open. Is
   that difference intended?
3. **Between draws.** Under S2 a depth-option part switches the mode off
   once it is finished. So with the switch off, under either reading, is
   the mode on only during the fresh draw of a depth-option part (and,
   under (a), of a depth-option shadow) and off between draws? Then no
   fresh draw of a type without the depth option is ever made in the
   depth-buffered mode, and its shadow never clears the depth plane.

## Possible observation
None: both are alternatives of our own, not behaviour of the original.

## Current workaround
No field: the on state (the original's behaviour) always applies.
