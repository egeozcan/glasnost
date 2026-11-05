---
id: Q-2674
title: How a unit or building type's display name is found in the string table
status: answered
related: [S-0004, S-0005, S-0006, S-0411, S-0512]
answered-by: [S-0004]
---

## Need

The name tag of M8 A4 round 8 note 54 (Q-2673), the sidebar's icon cards when an icon has no
image and its tooltips show a type's name. S-0004's sidebar keys give the display name as
`UIName` "(resolved to text)", S-0512's Import mapping gives a superweapon's `UIName` as a
"string-table label", and S-0006 claim 13 says a mode's label that the string table lacks shows
"the string table's placeholder text for a missing label". S-0005 says which file the string
table is and where it ranks. No spec gives how a type's label is matched against the table's
entries or what shows when the label is absent, empty or not in the table, so our interface shows
the type's section name (its rules identifier) wherever a display name belongs.

## Questions

1. Is a type's `UIName` value always a string-table label, or can it be literal text? Is it
   trimmed or cut to a length before the lookup?
2. Is the label matched against the table's entries case-sensitively, and are leading or trailing
   spaces in either significant?
3. When a type has no `UIName`, an empty one, or one the string table lacks, what does the
   interface show for it (in a sidebar tooltip, for example): nothing, the label itself, the
   section name, or the table's placeholder text for a missing label? What is that placeholder,
   described by its form (for example "the label with a fixed prefix"), not by quoting game text?
4. When the string table has an entry for the label, does the shown text keep the entry's line
   breaks or other control characters, and is it cut to a length?
5. Does a later rules layer's `UIName` (a map's rules section, S-0006) replace the label used for
   the display name in the same way as other keys?

## Possible observation

A probe string table (presentation only, as S-0411's and S-0512's scenarios use) with entries
whose labels differ from three probe types' `UIName` values only in letter case, in a trailing
space, and not at all; a fourth type with no `UIName` and a fifth whose label the table lacks.
Record the sidebar tooltip text of each type.

## Current workaround

The type's section name (`mapview::live::type_name`), shown in the sidebar and in the round 8
name tag. Cited `design: M8 A4 round 8 note 54` where the tag uses it.
