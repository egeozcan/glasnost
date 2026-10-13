---
id: Q-0123
title: In what order are the cancel orders given when one recheck removes several types?
status: answered
related: [S-0404, S-0407, S-0411]
answered-by: [S-0411]
---

## Need

`econ::sidebar::removal_orders` issues S-0411 claim 2's orders (S-0404 claim 9) for every
type a recheck removes from the taken-up set (S-0407 claims 1 and 12). For one type the order
is given: the cancel order first, then the cancel-all order. When one recheck removes several
types, the specs give no order between the types. The order can change the result when two
removed types share a line, for example when the line's item is of type A and type B is queued
behind it:

- A's orders first: the cancel of A may start B as the line's next item (S-0404 claims 3 and
  10). That creates B's object, with the creation draw of S-0404 claim 11. B's cancel-all then
  removes it again.
- B's orders first: B's cancel-all removes the queued B, then A's cancel leaves the line with
  nothing to start.

Both the random draws and the refunds can differ.

## Questions

1. When one recheck removes several types of the same house, in what order are their orders
   given (for example the icon order of the sidebar, the order of the types in the rules, or
   all cancel orders before all cancel-all orders)?
2. Can a cancel order applied in this group start the next queued type of the line even when
   that type is removed by the same recheck?

## Possible observation

Two types of one kind that share a prerequisite building: start the first, queue the second,
then destroy the prerequisite. Watch whether the second type's item is ever started (its build
progress, or the money taken for its first step) before both are gone.

## Current workaround

`removal_orders` walks the removed types in the taken-up set's order (the ordered set of type
references: kind, then type index), with each type's cancel before its cancel-all. Listed in the
provisional table of `docs/development.md`.
