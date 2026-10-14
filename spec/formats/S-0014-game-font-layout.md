---
id: S-0014
title: Game bitmap font file layout
topic: formats
revision: 1
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-14
measurements: []
requests: [Q-0501]
supersedes: []
---

# S-0014: Game bitmap font file layout

## Summary

The game's bitmap font contains a full 16-bit text-unit mapping and fixed-size,
one-bit glyph records. This spec supplies its file layout and the mask interpretation
needed by status text, answering Q-0501 questions 1 to 4.

S-0005 claim 18a owns file identity and archive precedence. S-0213 claim 25 owns
status-text metrics, line splitting, control characters and when fallback is used.
Skirmish presentation only is covered; network games and campaigns are out of scope.

## Terms

- **Text unit:** one unsigned 16-bit value in the text, not a Unicode scalar value
  assembled from several units.
- **Selector:** an unsigned 16-bit mapping value, with zero denoting absence and
  positive values denoting one-based glyph numbers.
- **Row span:** the imported number of bytes allocated to each bitmap row.
- **Record span:** the imported number of bytes allocated to each glyph record.
- **Mask:** a rectangular binary image whose set pixels paint the chosen foreground.

## Behaviour

1. **Header.** The covered file variant begins with a 28-byte header. Its first
   four bytes, in file order, are decimal 102, 111, 110, 84, spelling `fonT`.
   The remaining six values are unsigned 32-bit integers in little-endian byte
   order, with these meanings:

   | First byte, counted from 0 | Length in bytes | Meaning | Unit |
   |---|---|---|---|
   | 0 | 4 | identifying bytes above | bytes |
   | 4 | 4 | uninterpreted metadata value | integer |
   | 8 | 4 | row span B | bytes per row |
   | 12 | 4 | common bitmap height H | pixels |
   | 16 | 4 | line pitch | pixels |
   | 20 | 4 | glyph record count N | records |
   | 24 | 4 | record span S | bytes per record |

   The metadata value at file byte 4 is retained without assigning it a rendering
   role; individual glyph widths come from claim 3. Bitmap height and line pitch
   are distinct: the former describes stored rows, the latter supplies text-layout
   metrics under S-0213 claim 25. No palette is part of this header.
   - Backing: `measurable` (F1, F2) for the layout and established metrics.
     `unmeasured` for the metadata's meaning, because no effect of that value has
     been traced and a confirming observable is not yet known.
   - Confidence: high for the layout and established metrics; low for any
     interpretation of the uninterpreted metadata

2. **Mapping and covered range.** Immediately after the header are exactly
   65536 selectors, each an unsigned 16-bit little-endian integer: 131072 bytes
   in total. Selector number c belongs to text unit c, for every integer c from
   0 to 65535 inclusive. There is no first-character or last-character header
   value and no contiguous populated-range assumption. Zero selectors can occur
   anywhere in that range. Positive selectors may repeat, allowing aliases.
   A selector v selects zero-based glyph number v − 1. Selection does not reduce
   a text unit to eight bits or combine surrogate pairs. S-0213 claim 25 owns
   the text-unit-zero terminator and the special handling of tabs and breaks.
   - Backing: `measurable` (F1, F2)
   - Confidence: high

3. **Glyph records and pixels.** The mapping is followed immediately by N records
   of S bytes each, starting at file byte 131100. There is no per-glyph position
   table. Each record starts with one unsigned byte giving that glyph's width W
   in pixels. The following H rows each occupy B bytes. The height is common to
   all records: there is no per-glyph height or vertical displacement value.
   Records must have room for the width byte and all H × B row bytes.
   Any remaining bytes within S are not part of the bitmap.

   Rows run from top to bottom. Bytes within a row run left to right, and each
   byte's bits run from most significant to least significant, giving eight
   successive pixels. This is one bit per pixel, not four-bit or eight-bit palette
   data. Only the first W pixels of each row belong to the visible mask. The row
   span stays B even for a narrow glyph; unused bits at the right do not enlarge
   its width. The width must fit within 8B pixels. A zero-width glyph paints no
   pixels in our engine. The original has a cosmetic bug: a zero-width glyph can
   paint its first column when the first bit of a stored row is set.
   - Backing: `measurable` (F1, F2) for storage and original drawing. Suppressing
     zero-width pixels is a presentation design rule, not an oracle prediction.
   - Confidence: high

4. **Missing entry and fallback image.** For the full mapping in claim 2, a
   character counts as missing exactly when its selector is zero, not because
   its bitmap is blank or its stored width is zero. A positive selector denotes
   a present entry. The generated fallback is the complement of the mapped
   capital X, text unit 88, within that entry's stored width and the common
   bitmap height. Its width is the stored width of that X entry. There is no
   reserved fallback record and no substitution of the question-mark entry.
   The valid status-font scope here requires a mapped capital X. S-0213 claim 25
   owns when this generated image replaces a missing character and its advance.
   - Backing: `measurable` (F2; S-0213 claim 25)
   - Confidence: high

5. **Transparency and foreground.** A zero mask bit leaves the destination pixel
   unchanged. A one mask bit paints the status line's selected foreground.
   It does not select a palette entry. There is no font palette to import.
   S-0213 claim 25 owns the choice of foreground for each status notice, including
   the route notice's unmodified player-colour definition and the display-format
   conversion. The fallback uses the same foreground in our engine.
   The original has a cosmetic bug: fallback drawing complements the even-numbered
   bits of its packed 16-bit foreground, counting from bit 0, instead of using
   the ordinary foreground; there is no quirk switch for this drawing-only defect.
   - Backing: `measurable` (F3; S-0213 claim 25). The fixed fallback colour is a
     presentation design rule, not an oracle prediction.
   - Confidence: high

## Formulas

All quantities below are non-negative integers. Byte positions are counted from
file byte 0. Arithmetic is exact integer arithmetic with no rounding or floating
point. The divisions use the explicitly stated floor operation.

- Selector c begins at file byte 28 + 2c.
- Glyph j begins at file byte 131100 + jS, for 0 ≤ j < N.
- Pixel (x, y) uses the byte at 131100 + jS + 1 + yB + ⌊x / 8⌋,
  for 0 ≤ x < W and 0 ≤ y < H. Within that byte its bit number is
  7 − (x mod 8), counting the least significant bit as bit 0.
- The minimum record span is 1 + HB bytes.
- The header, mapping and records occupy 131100 + NS bytes in total.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Header length | 28 | bytes | Claim 1 |
| Text-unit mapping count | 65536 | selectors | Claim 2 |
| Mapping length | 131072 | bytes | Two bytes per selector |
| Pixel packing | 8 | pixels per byte | Most significant bit first |
| Fallback source | 88 | text-unit value | Capital X, not a fixed record number |

The dimensions, glyph count, widths and record span are imported, not engine constants.

## State and ordering

No simulation state or tick ordering is prescribed. Record and row order are data-file
order, not requirements on an implementation's internal representation.

## Edge cases and quirks

The cosmetic fallback-colour correction is in claim 5 and S-0213 claim 25.
Claim 3 also suppresses the original's stray zero-width pixels. Neither drawing-only
correction introduces a quirk switch.

This contract covers the `fonT` file variant and structurally valid records with
selectors either zero or within 1 to N. A different identifying sequence is outside
this contract, not another spelling of this variant. Truncated files, out-of-range
selectors, dimensions that cannot contain the mask, and a missing mapped X are
outside its valid-input scope; no original rendering result is asserted for them.

## Import mapping

| Input | Imported concept | Form |
|---|---|---|
| Game font file selected by S-0005 claim 18a | Header, mapping and masks | Binary layout of claims 1 to 3 |
| Record width byte | Individual glyph width | Unsigned integer pixels |
| Common bitmap height and line pitch | Bitmap rows and text-layout metrics | Unsigned integer pixels, distinct roles |
| Mapping entry for text unit 88 | Fallback source | Claim 4; generated image, not another file record |
| Status notice's foreground | Colour painted by set mask bits | S-0213 claim 25; not stored in the font |

No rules-file key controls this binary layout. Missing-file handling remains
S-0213 claim 25's contract.

## Measurement scenarios

**F1 — File-layout audit (claims 1 to 3).**
- Setup: one unmodified installation, with no loose overrides. No map, houses,
  game speed or running game is needed.
- Procedure: obtain the font selected by S-0005 claim 18a and inspect its header,
  selector counts and record boundaries; do not copy glyph data into a report.
- Observable: identifying bytes, table length, maximum selector, record count,
  record span, row capacity and total length. One inspection suffices.
- Expected: the identifying bytes match claim 1, the full mapping occupies
  131072 bytes, all nonzero selectors refer to records, all widths fit their rows,
  and the records fit exactly the header's counts. A differing header sequence,
  incompatible table length or incompatible record boundaries falsifies the layout.
- Tool: read-only archive extraction and a binary-format checker.

**F2 — Mapping and mask orientation (claims 1 to 4).**
- Setup: a tiny purpose-made skirmish map with a distant bystander computer player,
  stock data, game speed 2, 800 × 600, director and unrelated mods off. Named test
  files are one valid font copy and status-text resources used by a repeatable
  notice; restore them after each run. Follow S-0213 claim 25's status scenario
  and void rule. Keep positive widths comfortably fitting the line; the separate
  zero-width control below also fits.
- Procedure: supply asymmetric synthetic masks spanning multiple bytes and rows,
  different widths with the same row span, two aliases, a unit above 255 with a
  distinct low-byte mapping, a zero selector and a positive-width blank record.
  In a second copy change the mapped X's mask and width. Include a separate mapped
  zero-width record with its first row bit set, leaving a positive-width mapped X.
  Run each copy once.
- Observable: screen-captured masks, following-character origins and fallback shape.
- Expected: top-down rows and most-significant-bit-first pixels match the synthetic
  masks; aliases match, the high text unit is not truncated, the blank mapped entry
  stays blank, and the zero selector uses a complemented X with X's width. Changing
  X changes the fallback. The original's zero-width control can paint a stray
  first-column pixel while retaining zero stored width; our engine paints none.
  Any stable opposite oracle result falsifies the relevant original-behaviour claim.
- Tool: scripted input and lossless screen capture; no simulation recorder is needed.

**F3 — Transparent mask and foreground (claim 5).**
- Setup: F2's skirmish setup and named test font, with a repeatable route-refusal
  notice, ordinary glyphs and a missing character. Use two player-colour choices
  and both a dark and a bright visible background.
- Procedure: capture one notice for each colour/background combination, away from
  overlap by unrelated interface text.
- Observable: pixels under set and clear mask bits, including fallback pixels.
- Expected: clear bits preserve the background; set ordinary bits use S-0213
  claim 25's foreground, with no font-palette lookup or battlefield lighting.
  Oracle fallback pixels exhibit the cosmetic packed-colour defect recorded in
  claim 5; our engine deliberately uses the ordinary foreground. Opaque clear
  bits or palette-derived ordinary foreground falsifies claim 5.
- Tool: lossless screen capture with pixel comparison.

## Open questions

- Behaviour of a font without a mapped capital X remains outside the valid-font
  scope, as in S-0213 claim 25. It does not block importing the installed font.
- Q-0501 question 5 belongs to S-0213 claim 25, not this layout contract.
