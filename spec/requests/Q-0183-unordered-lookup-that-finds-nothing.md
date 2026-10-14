---
id: Q-0183
title: Does a lookup that sorts an unordered name table and finds nothing forget the remembered occurrence?
status: open
related: [S-0002]
answered-by: []
---

## Need
S-0002 State and ordering, *Name tables during and after a layered load*, has two rows for a
lookup that finds nothing. For an ordered table (row 3) it says "Nothing changes: a remembered
occurrence, if any, stays". For a table that is not ordered (row 5) it says the working order is
sorted, the table becomes ordered, and "nothing is remembered". A table that is not ordered can
hold a remembered occurrence: row 7 lets one outlast an addition, and an addition leaves the
table not ordered. Row 5 can be read two ways: the table forgets that remembered occurrence, or
the lookup only adds nothing to remember and the old one stays, as in row 3. The two readings
give different winners after a layered load in which a line of the second file adds a new name
to a table with a repeated name hash, a later line looks up a name the table does not hold, and
a lookup after that asks for the remembered name. The engine's name table
(`crates/ini/src/table.rs`, `Table::lookup_mut`) has to pick one.

## Questions
1. When a lookup in a table that is not ordered finds nothing, and the table holds a remembered
   occurrence (left there by row 7), does the table still hold that remembered occurrence after
   the lookup?
2. If the answer depends on anything else (for example on whether the remembered occurrence's
   hash is the one looked up, which row 1 already covers), which cases differ?

## Possible observation
A variant of S-0002 scenario D10 (a): the test file of row D-l with two copies of the mission's
section, and a loose copy of the mission information file whose lines for the mission's section
come first, followed by one new section (added below a forgetting peak, so the copy written into
stays remembered and the table becomes unordered), followed by a line for a second new section.
That line's section lookup finds nothing in an unordered table. The briefing shown then tells
the two readings apart, when the hashes are chosen so that re-sorting returns the other copy.

## Current workaround
The engine reads row 5 literally as forgetting: after the sort, the table holds no remembered
occurrence. Row 3 (ordered table) keeps it. The test `table::tests::model_transitions` pins this
reading, and the docs/development.md interim table lists it.
