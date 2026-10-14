---
id: Q-0234
title: How the items of the virtual-purifier, extra-credit and computer-timer integer lists are read
status: open
related: [S-0003, S-0402, S-0412]
answered-by: []
---

## Need

S-0402's Import mapping reads `AIVirtualPurifiers` in `[General]` as a "list of integers", and
S-0412's reads `MultiplayerAICM` and `AIHateDelays` in `[General]` the same way. S-0003 now
says how two integer lists are read: the debris maxima (claim 20) and the auto-deploy delays
(claim 22, the answer to Q-0171), both with decimal prefixes only, an unreadable non-empty item
giving 0 in its place, empty items taking no place and a 511-byte text limit. No spec says
whether these three lists use that form too. The importer's general integer-list form
(`convert::sim`, `Form::Ints`) reads them.

## Questions

For each of `AIVirtualPurifiers`, `MultiplayerAICM` and `AIHateDelays`:

1. Is each item read with S-0003 claim 3's integer form (with its hexadecimal forms), or as a
   plain decimal prefix as in claims 20 and 22? What do `$10`, `10h` and `0x10` give?
2. Does a non-empty item with no readable number (for example `abc`) give 0 and keep its place,
   take no place, or keep the value held for that position?
3. Is the text limit of the value the common 127 bytes or 511 bytes?
4. Does an empty item (`5,,7`) take a place?
5. Does a missing or empty key keep the list in force, and does a present non-empty value
   replace it?

## Possible observation

For `AIVirtualPurifiers`: a computer house at difficulty index 1 with a refinery, the list set
to `abc,1,2`, then `$10,1,2`, `10h,1,2` and `5,,1`; the number of virtual purifiers counted for
that house's harvest income (S-0402 claim 12) shows which item stands at that place. For
`MultiplayerAICM`: the same values and the extra credits a computer house at difficulty
index 1 receives (S-0412 claim 3). `AIHateDelays` has no effect in our engine; its form only
matters if it is ever read.

## Current workaround

Each item is read with S-0003 claim 3's integer form within the common 127-byte limit; empty
items take no place; an item the integer form cannot read gives 0 and keeps its place (the
former workaround of Q-0171). Interim row in `docs/development.md`.
