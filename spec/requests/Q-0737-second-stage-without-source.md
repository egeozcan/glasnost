---
id: Q-0737
title: A second-stage (warp) order when the first stage has never picked a source
status: open
related: [S-0512, S-0513]
answered-by: []
---

## Need
S-0512 claim 10 says a second-stage order:
- starts the warp from the first stage's last picked source;
- restarts the first stage.

With `quirks.second_stage_order_unchecked` on, the order is not checked. So a raw or
trigger-made order can reach the second stage before any source was ever picked. No spec says
what the warp uses as its source then, or whether the first stage still restarts.

## Questions
1. With no source ever picked, does the warp act on some default cell (which one?), or on
   nothing?
2. Is the first stage's charge restarted anyway?

## Possible observation
Use a named map that grants a one-time warp record and a ready first stage that never fired.
Send the warp order (by a trigger or through the radar). Record what moves and the first
stage's charge.

## Current workaround
Interim rule: the order does nothing at all (no warp and no restart), and the engine notes it.
