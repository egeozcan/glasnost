---
id: Q-0131
title: A building's upgrade count after one of its upgrades is sold
status: answered
related: [S-0406, S-0408]
answered-by: [S-0408]
---

## Need

S-0406 claim 12 keeps an upgrade count per building (raised by 1, or by a level field *n*, with
a cap), separate from the upgrades recorded in the three positions; the room rule of a later
upgrade reads the count. S-0408 claim 5 says that a sale removes the most recently added upgrade
and the building is "otherwise unchanged", but not what happens to the count. After a sale the
building could have room again, or not.

The engine needs this in `econ::sell::sell_upgrade`.

## Questions

1. After the sale of an upgrade added without a level field, can the same building take a new
   upgrade of that type again (is its room restored)?
2. After the sale of an upgrade added with a level field *n* above 1 (S-0406 claim 12: the count
   is raised by *n*), what is the count: lowered by 1, by *n*, set to the number of upgrades still
   recorded, or unchanged?
3. On a building of the wall tower type whose count is above its type's upgrade count, the same.

## Possible observation

A test building type with upgrade count 2 and an upgrade type; add two upgrades, sell one, then
try to place a third. Repeat with a level-field upgrade type and with the wall tower type.

## Current workaround

The count drops by 1, never below 0, whatever raised it. Listed in `docs/development.md`'s
provisional table.
