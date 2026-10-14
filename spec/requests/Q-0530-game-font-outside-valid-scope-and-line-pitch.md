---
id: Q-0530
title: What does our engine do with a game font outside S-0014's valid scope, and does the font's line pitch place status lines?
status: answered
related: [S-0014, S-0213, S-0005]
answered-by: [S-0014, S-0213]
---

## Need

S-0014 (revision 1) gives the `fonT` layout and declares several inputs outside its
valid-input scope: a different identifying sequence, a truncated file, a selector greater
than the record count, a record span too small for the width byte and the rows, a width
wider than the row span holds, and a font with no mapped capital X (S-0014 Edge cases;
S-0213 claim 25 says behaviour without a mapped X "remains open"). The importer
(`glasnost_fmt::fnt`) and the status display (`play::say`, `session::draw`) must decide
what to do with such a file. The status display is the only user so far.

S-0014 claim 1 also says the header's line pitch "supplies text-layout metrics under
S-0213 claim 25", but claim 25 never uses it. It places line origins 19 pixels apart in
the original display, and its Import mapping row for `GAME.FNT` lists the mapping, masks,
widths and bitmap height, not the line pitch.

## Questions

1. When the resolved game font is outside S-0014's valid scope, should our engine treat it
   exactly as a missing font (S-0213 Import mapping: a missing-asset diagnostic and the
   status text omitted), or use what it can (for example, treat an out-of-range selector as
   missing, or skip only the malformed records)? Is a font with no mapped capital X usable
   for text that has no missing characters?
2. Does the font's line pitch set the distance between status-line origins, so that a font
   with another pitch moves the lines, or are the 19 pixels of claim 25 fixed whatever the
   pitch? If the pitch has another role in status text (for example the line's height for
   overlap or clearing), which?
3. Are status-line glyphs drawn with their top row at the line origin, or offset from it
   by some amount that depends on the bitmap height or line pitch?

## Possible observation

S-0014 F2 with a test font whose line pitch differs from the stock one, two status lines
shown, and their glyph rows captured; S-0005 Scenario M26 with a truncated copy and a copy
without a mapped capital X.

## Current workaround

- A file that fails S-0014's structural checks, or has no mapped capital X, is not
  loaded. The session reports one diagnostic naming the file and the reason and omits
  status text, as for a missing file (S-0213 Import mapping).
- The line pitch is read and kept but not used. Line origins stay 19 pixels apart
  (S-0213 claim 25), and each glyph's top row is at its line origin, the first line at the
  battlefield's top edge.
