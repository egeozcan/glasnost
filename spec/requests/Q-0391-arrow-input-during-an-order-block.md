---
id: Q-0391
title: Does an arrow key pressed during an order block scroll the view?
status: open
related: [S-0201, S-0101, S-0213]
answered-by: []
---

## Need

S-0201 claim 14 says that edge scrolling is suppressed during an order block, and that an arrow
key already held before the block keeps scrolling in unfrozen cycles. It leaves open whether new
arrow input is accepted during the block. The game client (`game::client`, `session::scroll_cycle`)
must decide what an arrow pressed, or released and pressed again, after the block began does.

## Questions

1. During an order block, does pressing an arrow key that was not held when the block began
   scroll the view in the following unfrozen cycles?
2. If an arrow held before the block is released during the block, does the view stop scrolling
   in that direction at once, and does pressing it again during the block scroll again?
3. When the block ends with an arrow pressed during it still held, does the view scroll from the
   first unblocked cycle?

## Possible observation

Start a skirmish and record the view position each cycle while pressing arrow keys at known
cycles before, inside and after the start-of-game order block.

## Current workaround

The client accepts arrow presses and releases during the block as at any other time, and only
edge scrolling is off (`scroll::edge_enabled`).
