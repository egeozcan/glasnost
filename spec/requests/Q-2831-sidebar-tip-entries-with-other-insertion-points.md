---
id: Q-2831
title: What a sidebar tip shows when its entry's insertion points differ
status: open
related: [S-0411]
answered-by: []
---

## Need

S-0411 r57 claims 20 and 22 fill two string-table entries with values: the
money-format entry (`TXT_MONEY_FORMAT_2`, a text point and then a number
point: the display name and the price) and the power entry
(`TXT_POWER_DRAIN`, two number points: output and drain). Terms gives how
the points are marked and that they are filled in the order they stand.
Open question 11 leaves open what shows when an entry's points differ from
Import mapping's. A mod's string table can hold such an entry, so the
sidebar (`crates/app/src/game/sidebar_tip.rs`, `fill`) must write something
for it.

## Questions

1. The money-format entry holds only one point, or none (for example a
   text with just a text point, just a number point, or no percent sign at
   all). What does an icon tip show? Is the missing value left out, or is
   something else written in its place?
2. The money-format entry holds its points in the other order (a number
   point first, then a text point). Is the price written at the first point
   and the name at the second, or the name at the first point whatever its
   letter?
3. The money-format entry holds more points than two (for example a third
   number point, or a second text point). What does each extra point
   show?
4. The power entry holds one number point, or three, or a text point. What
   does the power meter's tip show at each point?
5. An entry holds a percent sign followed by another letter, or by a second
   percent sign, or a percent sign at the very end of its text. Is that
   shown as it stands, or changed?

## Possible observation

S-0411 scenario S16 with a string table whose `TXT_MONEY_FORMAT_2` and
`TXT_POWER_DRAIN` entries are replaced, one run per case, by invented texts
with the point layouts above (each point marked as Terms gives), reading the
tip text of a unit icon and of the power meter.

## Current workaround

Our engine's interim, recorded as design in `sidebar_tip::fill`: each text
point takes the next text in the order the claims list them (the display
name) and each number point the next number (the price; output, then
drain), whatever the order in which the two kinds stand. A point with no
value left writes nothing, values left over are not written, and a percent
sign followed by anything other than `s` or `d`, or at the end of the text,
is shown as it stands.
