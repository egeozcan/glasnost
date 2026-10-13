---
id: S-0013
title: voxel lighting table file layout
topic: formats
revision: 1
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-13
measurements: []
requests: [Q-0022]
supersedes: []
---

# S-0013: voxel lighting table file layout

## Summary

The byte layout of the voxel lighting table file and the treatment of incomplete or excessive
content. S-0105 claim 38 owns how the decoded content affects drawing; this spec makes that
content locatable without guessing from file length. Skirmish is the scope; campaigns and
network games are out of scope.

## Terms

- **File position.** A byte position counted from 0 at the beginning of the file.
- **Header.** The first 16 bytes of this file, containing four unsigned integer values.
- **Row.** The 256 palette indices for one shade index, in increasing voxel-colour-index order.
- **Extra value.** The fourth header value, retained as file content but unused for drawing.
- **Complete length.** The length covering the header, the palette and all declared rows,
  whether or not additional bytes follow.

## Behaviour

1. **Header in file order.** The values occupy the positions below. Each is an unsigned
   32-bit integer stored in four bytes, least significant byte first (little-endian).
   There is no separate signature or version value before them.

   | File positions, inclusive | Width | Meaning |
   |---|---|---|
   | 0–3 | 4 bytes | Remap-range start |
   | 4–7 | 4 bytes | Remap-range end |
   | 8–11 | 4 bytes | Row count R |
   | 12–15 | 4 bytes | Extra value |

   Every value can represent 0 through 4294967295. The row count is always the value at
   positions 8–11, not whichever header value happens to fit the file's length. The other
   three values do not determine row locations, row counts or house colours (S-0105 claim 38).
   - Backing: `measurable` (L1, L2)
   - Confidence: high

2. **Embedded palette.** Positions 16–783 contain 256 consecutive colour entries of three
   bytes each: red, green, blue, in that order. Each component is an unsigned 8-bit value,
   0–255, copied without scaling. Entry i begins at position 16 + 3i, for i from 0 to 255.
   This palette is not the palette used for voxel drawing; changing it has no drawing effect
   (S-0105 claim 38).
   - Backing: `measurable` (L1 for the extent and component values, L2 for the ignored palette)
   - Confidence: high (extent and ignored content), medium (component names, which drawing cannot distinguish)

3. **Rows immediately follow.** Row 0 begins at position 784. Rows are uncompressed and
   consecutive, with no intervening bytes: row q begins at 784 + 256q, and its entry for
   voxel colour index χ is the one unsigned byte at 784 + 256q + χ. Both χ and the resulting
   palette index range from 0 to 255. Declared rows have q from 0 to R − 1; R = 0 declares
   no rows. The complete length is 784 + 256R bytes. It is not an acceptance test:
   shorter files supply only part of the content, and longer files have ignored content.
   Claim 6 qualifies this for excessive counts.
   - Backing: `measurable` (L1, L3, L4)
   - Confidence: high

4. **Short or absent file.** On a fresh program start, a missing file or a file shorter than
   784 bytes supplies no row entries. This includes a short header, a complete header with a
   short palette, and a file ending exactly at position 784 with no row bytes. For R at most
   128, once both header and palette are complete, each available byte of the declared rows
   supplies its own entry, even when the last supplied row is incomplete. Every unsupplied
   entry is zero. Claim 6 gives the original's excessive-count exceptions and our engine's
   bounded result.
   The original carries on starting up despite an unsuccessful load; our engine does so by
   default too, with an importer warning (S-0105 claim 38 and
   `quirks.vpl_unloaded_rows_blank`). A zero entry is a transparent palette index, not a
   request to skip writing the voxel (S-0105 claims 19 and 38).
   - Backing: `measurable` (L4 and S-0105 MS-23)
   - Confidence: medium

5. **Long file.** With a row count from 0 to 128, bytes after position
   784 + 256R − 1 have no effect, even when they contain whole additional rows. Reducing R
   without shortening the file therefore removes the later rows from the supplied table.
   File length never repairs or replaces R. Our importer warns about trailing bytes without
   changing the result (S-0105 Import mapping).
   - Backing: `measurable` (L2, L3)
   - Confidence: high

6. **Excessive row count.** Let B = (256R) modulo 4294967296, in bytes. The original's
   requested amount is B, not the exact product. With a complete header and palette,
   B below 2147483648, and no more than 32768 bytes supplied by that request, it uses
   exactly the first min(B, L − 784) row bytes, where L is file length in bytes. Unsupplied
   entries remain zero. In particular, counts above 128 but below 8388608 still use an
   available prefix of at most 128 rows normally. A count of 16777216 requests zero bytes
   and supplies no entries, even when the file contains rows.

   When B is at least 2147483648, the original's result depends on how the file is stored
   and can disturb unrelated game state. Supplying more than 32768 requested row bytes
   can also disturb unrelated game state. These results are unspecified and must not be
   tested in the oracle. The first problematic count is 8388608, before the unsigned
   product first wraps at 16777216.

   Our engine never wraps the requested length: it uses at most the first min(R, 128)
   rows, preserving any partial row and filling unsupplied entries with zero. Remaining
   bytes are ignored. Any R above 128 produces an importer warning. This is the safe
   deterministic departure shared with S-0105 claim 38, not a quirk switch. S-0105 needs
   the corresponding correction to its original excessive-count boundary in this round.
   - Backing: `measurable` (L5) for the safe excessive-count and zero-byte wrapped cases,
     and our engine's result. The unsafe original cases are `unmeasured`: their result
     is unspecified and a test could disturb unrelated game state.
   - Confidence: high (requested amount, safe prefix and our engine's rule), medium
     (classification of storage-dependent unsafe results)

## Formulas

All arithmetic here is exact integer arithmetic, with no floating-point calculation or rounding.
Positions and lengths are in bytes; counts are in rows or colour entries as indicated.

- Four little-endian bytes b₀, b₁, b₂, b₃ represent
  b₀ + 256b₁ + 65536b₂ + 16777216b₃ (claims 1 and 6).
- Palette entry i occupies positions 16 + 3i through 18 + 3i (claim 2).
- Row entry (q, χ) occupies position 784 + 256q + χ (claim 3).
- Complete length = 784 + 256R bytes (claim 3).
- For our engine, let L be file length in bytes. Supplied row bytes are zero in number
  when L < 784; otherwise their number is min(L − 784, 256 × min(R, 128)). This is a
  count of consecutive bytes starting at position 784, not a whole-row rounding (claims 4–6).

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Header extent | 16 | bytes | Claim 1 |
| Header-value width | 32 | bits | Four bytes each, claim 1 |
| Embedded palette extent | 768 | bytes | 256 entries of three bytes, claim 2 |
| First row position | 784 | bytes from file beginning | Claims 2 and 3 |
| Row extent | 256 | bytes per row | Claim 3 |
| Retained row limit | 128 | rows | Departure shared with S-0105 claim 38 |

## State and ordering

File order fixes which entries survive truncation. No order of internal import work is required.
The table is loaded once at program start, not again at each game start (S-0105 claims 19 and 38).

## Edge cases and quirks

`quirks.vpl_unloaded_rows_blank` is owned by S-0105. Default original: missing entries are zero
and start-up continues. Its fixed alternative rejects an incomplete table with an error.
The excessive-count departure of claim 6 applies independently of this switch: it never permits
unsafe loading. No new switch is introduced here.

## Import mapping

The input is `voxels.vpl`, resolved by S-0005's file precedence. No INI key supplies the
header or selects its interpretation. The positions in claims 1–3 supply the corresponding
concepts directly. Drawing uses the decoded rows as S-0105 specifies, with palettes from S-0103.

Warnings name the file and the relevant file position or missing content:

| Input condition | Result with the default quirk | Warning location |
|---|---|---|
| File missing | All entries zero | Requested file |
| Fewer than 16 bytes | No rows supplied | End of file, incomplete header |
| 16–783 bytes | No rows supplied | End of file, incomplete palette |
| Complete header and palette, fewer than 256R row bytes | Available entries used, rest zero | End of file |
| R below 32 | Undeclared rows zero | Positions 8–11 |
| R above 128 | Claim 6's bounded result | Positions 8–11 |
| Bytes after all declared rows | Trailing bytes ignored | First trailing byte |

The short-content comparison uses exact 256R, never a wrapped product. Ignored header values
and palette components are not validated against drawing's remap range or palette. Trailing
bytes warn but do not change the imported table. These warnings implement S-0105's importer
warnings and claim 38.

## Measurement scenarios

No M-ID is claimed. L1 is file inspection only. L2–L4 use named test copies, the exception in the
measurement setup, never a changed executable. Run each oracle variant from a fresh program start,
with director and mods off except the named test data; remove that data afterwards. Use a tiny flat
skirmish map, one human player and one distant bystander computer player, game speed 2, 800×600.
The observed area is the parked test vehicle's cell and the surrounding three-cell margin.
Apply the bystander's void rule. Use S-0105 MS-23's colour/normal probe model with separated pixels,
no overlapping voxels, and cache disabled. Capture all 32 body facings and compare pixel-for-pixel
with the stock-table control. One run per variant suffices: no random statistic is involved.
A screen recorder and a read-only archive extractor/file inspector suffice; the combat bench
is not needed. Any harness changes are proposals to the user, not edits to the harness.

**L1 — File extents and value forms (claims 1–3).**
- Setup: the stock voxel lighting table from each of the base and expansion archives, extracted
  read-only; no game run or map is needed.
- Procedure: count bytes, decode the four header values as claim 1 gives, inspect the component
  range and the remaining consecutive row bytes. Record the file length, header values and
  positions, and whether the two files match; keep raw component lists outside the spec.
- Expected: the row count at positions 8–11 makes the stock length equal 784 + 256R.
  Positions 16–783 form 256 three-byte colours, with component values reaching beyond 63.
  Exactly R blocks of 256 bytes follow. The two stock copies match.
- Falsifier: a length inconsistent with that count, row bytes beginning elsewhere, or a
  four-byte colour form needed to account for the content. Component names cannot be
  established from drawing, since this palette is ignored; colour triples in the file are the
  observable for their interpretation.

**L2 — Row-count location and unused content (claims 1, 2 and 5).**
- Procedure: make named loose test copies of the stock file: A changes only positions 8–11
  to the little-endian value 20; B changes positions 0–3, 4–7 and 12–15 each to 20 and
  replaces positions 16–783 with zero bytes, leaving R and all rows unchanged.
- Observable: the probe pixels in each capture, compared with a control image calculated
  from the unchanged file and a second control with rows 20–31 zeroed.
- Expected: A matches the zeroed-row control, even though later row bytes remain in the file.
  B matches the unchanged control, including its house colours. This also discriminates against
  choosing a header value by file length.
- Falsifier: A retains the later shades, B loses them, or B changes any pixel.

**L3 — Individual entries, consecutive rows and long files (claim 3 and claim 5).**
- Procedure: choose a probe colour χ and shade q that occur in the control captures. Make
  copy C with only the byte at 784 + 256q + χ replaced by a different nonzero palette index
  outside the house-colour range. Make copy D by appending one 256-byte test row to the stock
  file without changing R. Make copy E by appending eight rows with distinct entries and
  increasing R by eight, remaining within 128 rows. Record all changed positions and lengths.
- Observable: changed probe pixels and image equality.
- Expected: C changes exactly the pixels using that shade and colour, to the chosen index
  through the object's drawing palette. D and E match the control: D declares no extra rows,
  and E's extra shade rows are not selected by drawing (S-0105 claim 38). On our importer,
  D warns at the first appended byte, while E has no trailing-content warning.
- Falsifier: C changes another shade/colour combination or nothing at all, D/E changes
  the drawn vehicle, or our importer gives the wrong trailing-content warning. Together with L1, the single-byte change tests uncompressed row data.

**L4 — Truncation boundaries (claim 4 and claim 3).**
- Procedure: create copies ending after 1, 16, 783 and 784 bytes, plus one ending after
  784 + 256 × 19 + 128 bytes. Leave the retained bytes unchanged.
- Observable: successful start-up and probe images; replay imports on our engine and record warnings.
- Expected: the first four copies give all-zero rows. The last retains rows 0–18 and entries
  0–127 of row 19; entries 128–255 of that row and all later rows are zero. Our engine warns
  about incomplete content as the Import mapping gives. S-0105 MS-23's missing-file variant
  confirms the absent-file case without assuming a loose-file deletion hides archive copies.
- Falsifier: any of the first four copies supplies a row, the last loses a complete earlier row
  or supplies an entry after its end, or start-up stops under the default quirk.

**L5 — Excessive counts without unsafe oracle loads (claim 6).**
- Setup and procedure: set only positions 8–11 of stock copies to little-endian 200 and,
  separately, 16777216. Confirm first that the stock file contains at most 128 rows. Run
  those two variants in the oracle: the latter requests exactly zero bytes, so it cannot
  supply excessive content. Test counts 0, 129, 200, 8388607, 8388608, 16777216 and
  4294967295 on our importer separately, with no row bytes, a partial first row, and 129
  complete rows of named test data. Never run negative-request or excessive-content cases
  in the oracle, including a count of 8388608 even with a short file.
- Observable: oracle images; imported entry values, retained row counts and warnings
  from our engine, across two repeated imports of every fixture.
- Expected: the 200-count oracle variant matches the stock control. The 16777216-count
  variant has all-zero entries and the same blank look as L4's 784-byte file. Our engine
  retains exactly the prefix defined by the supplied-byte formula for every count, with
  zeros for unsupplied entries; R above 128 warns and no import reads beyond the file or
  retains more than 128 rows. Results repeat.
- Falsifier: either oracle variant differs from its stated control, or our importer wraps
  the count, uses a different prefix, lacks the warning, fails to complete, or changes
  result on repetition. The unsafe original behaviour has no predicted image and is not
  a measurement target.

## Open questions

None blocking Q-0022. The embedded palette's component names cannot be distinguished by the
original's drawing, because that content is unused; claim 2's confidence reflects this limitation.
