---
id: S-0012
title: Map block encodings
topic: formats
revision: 2
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-14
measurements: []
requests: []
supersedes: []
---

# S-0012: Map block encodings

## Summary

The byte encodings of a map's current terrain stream, overlay type and data streams, and preview image: joining the text, decoding it, expanding compressed chunks, and interpreting their bytes. S-0009 owns their meaning in a skirmish; S-0010 owns the interpretation of tile numbers and sub-tiles. Network games, campaigns and the four legacy terrain encodings are outside this spec.

## Terms

*Cell*, *height level*, *overlay*, *overlay data value*, *sub-tile*, *theater*, *importer warning*, *departure*, *quirk switch* and *measurement setup* are as in the glossary. *Legal cell*, *selected cell*, *initial fill*, *ice-growth flag* and *no-tile marker* are S-0009's.

- **Packed text:** the joined entry values of one encoded map section.
- **Packed bytes:** the bytes obtained from packed text, before compression is expanded.
- **Chunk:** one independently compressed part, with its two length values and its compressed bytes.
- **Expanded bytes:** the concatenation of the uncompressed contents of the chunks. Chunk boundaries are not cell-record or pixel boundaries.
- **Literal:** a byte copied directly from the compressed file to the expanded result.
- **Copy distance:** the number of bytes back from the next output position to the start of a copy.
- **Copy position:** a position counted from byte 0 of the current chunk's expanded result.
- **Control byte:** a compressed byte that specifies literals, a copy, a repeated byte, or the end of a chunk's compressed content.
- **Complete chunk:** a chunk with both length values, all its advertised compressed bytes, a valid end marker, and exactly its advertised expanded length.

Byte positions below count from 0. All arithmetic below is exact integer arithmetic, with no floating-point calculations. A byte is an unsigned integer from 0 to 255 unless a signed interpretation is explicitly given. A packed-text character means one file byte, not a Unicode character (S-0001).

Warnings, quirk alternatives and safe departures are our engine's design requirements, not claims that the original diagnoses bad files. They have high confidence and `unmeasured` backing as design choices; engine tests in the scenarios check conformance to those requirements. Original-game backing is listed separately where a scenario cannot isolate a part.

## Behaviour

1. **Joining the text.** The game reads entries in their section's enumeration order, as defined by S-0002, not in numerical order of their keys. The usual numbered keys are labels, not a sorting instruction: for distinct keys with distinct name hashes, renaming them without changing their enumeration order does not change the result. Renaming that introduces a duplicate or hash collision still follows S-0002's lookup rule. Each enumerated key retrieves its value by the name-resolution rules of S-0002, including duplicate keys. Retrieved values are limited to their first 127 characters, trimmed, then joined without a separator. A base-64 group may cross a line boundary. The game writes successive decimal labels starting at 1 and at most 70 encoded characters per entry, but the reader does not require those labels or that length.
   - Backing: `measurable` (T1) for reader behaviour. The writer's labels and line-length convention are `unmeasured`: T1 checks installed files but does not exercise an original map-writing operation in the scoped skirmish setup.
   - Confidence: high

2. **Text-to-bytes encoding.** The alphabet, in increasing digit value, is uppercase A through Z, lowercase a through z, digits 0 through 9, then `+` and `/`: 64 digit values, from 0 to 63. Four characters encode three bytes by F1. For a last group with two digits and `==`, only its first byte is emitted; with three digits and `=`, its first two bytes are emitted. There is no alternate alphabet using `-` and `_`.

   The original processes groups of four *characters*, not groups of four valid digits. Inside one complete group, characters outside the alphabet other than `=` are skipped, and `=` ends that group's interpretation. The digit values collected before that point occupy the highest six-bit places of a 24-bit integer; the unfilled places are zero. It emits exactly one byte for zero, one or two collected digits, two bytes for three digits, and three bytes for four digits. With no collected digits that byte is 0; with two digits it is ⌊(64·d₀ + d₁) / 16⌋. These counts apply whether collection ends at `=` or at the end of the four-character group. Interpretation resumes with the next four-character group, even after a group containing `=`. This differs from globally removing invalid characters or ending the whole block at the first `=`. A standard group is four alphabet digits, or a final group of two digits followed by `==` or three digits followed by `=`. Other complete groups are nonstandard. Our engine reproduces this for complete groups with an importer warning for nonstandard groups (`quirks.map_base64_groups`). The alternate behaviour rejects the block's text on any nonstandard group; it treats the block as absent and warns.

   A final group shorter than four characters has an unspecified result in the original. Our engine discards only those remaining characters and warns; bytes from preceding complete groups remain available. A normally encoded block ends on a complete four-character group.
   - Backing: `measurable` (T1 and T2) for complete groups. The original's incomplete-final-group result is `unmeasured`: T2 can record failures, but the same file need not yield reproducible bytes, so it cannot establish a byte rule for that case.
   - Confidence: high for the alphabet and normal groups; medium for malformed groups

3. **Chunk framing.** Packed bytes are a sequence of chunks. At a chunk's starting position, positions 0 and 1 give the compressed length *C*, and positions 2 and 3 give the expanded length *U*, both unsigned 16-bit little-endian integers. The following *C* bytes are the compressed content. The next chunk starts *4 + C* bytes after this chunk's start. There is no file-wide length, chunk count, checksum or alignment gap in this stream. Each chunk starts with empty copy history: it cannot refer to another chunk's expanded bytes. The game writer divides the uncompressed stream into parts of at most 8192 bytes; a reader must not assume that every chunk expands to exactly 8192 bytes.

   Terrain and preview chunks use the distance-and-copy encoding in claim 4. Both overlay streams use the literal-and-copy encoding in claim 5. Even an incompressible part uses the relevant compressed encoding, not an implicit raw-byte alternative. In a valid chunk the compressed end marker lies within the *C* bytes and the expansion has length *U*. Short framing, disagreeing lengths and invalid compressed content are covered by claims 9 and 10.
   - Backing: `measurable` (C1, C2 and C3) for chunk framing, valid expansion and within-chunk copy positions. The writer's 8192-byte partition limit is `unmeasured`: C1 checks shipped examples but cannot test every writer output, and no scoped original-writing scenario is proposed. The requirement not to copy unavailable history is also an engine safety requirement under claim 10.
   - Confidence: high

4. **Distance-and-copy encoding.** The compressed terrain and preview contents use the tables D1 and D2 under Formulas. A literal contributes its byte unchanged. A distance copy contributes bytes from the current chunk's expanded result, beginning *D* bytes before the next output position. Copies may overlap the output they produce: a distance of 1 repeats the last byte, and a distance of 2 repeats the last two bytes. After a distance copy, the encoded trailing-literal count *S* contributes the next *S* compressed bytes literally. The state for interpreting the next control byte depends on that count, as table D2 specifies. The end marker contributes no output.

   At the start of a chunk only, a first control byte *t* above 17 introduces *t − 17* literal bytes, followed by the post-literal state. Otherwise the first byte is interpreted in the ordinary state. In particular, the original places even an initial run of one, two or three literals in the post-literal state: a subsequent control below 16 then names a three-byte copy, not a two-byte copy. This rule concerns file bytes, not how an encoder ought to choose its runs.
   - Backing: `measurable` (C1 and C2) for valid controls, including both below-16 copy contexts with sufficient history. The initial one-to-four-literal run followed by a below-16 control and non-end controls 16 … 31 are `unmeasured`: within supported chunks they require unavailable history, so C2 cannot distinguish their distances or lengths by a valid original image. Failure recording and engine rejection do not establish those original formulas.
   - Confidence: high for the literal runs and copy controls exercised by ordinary files; medium for the below-16 copy contexts, non-end controls 16 … 31, and the initial-short-run case

5. **Literal-and-copy encoding.** The compressed overlay contents use table L under Formulas. Copies name either a distance behind the next output byte or an absolute copy position within the current chunk. An overlapping copy extends from the bytes being emitted, rather than from a snapshot of the output before the copy. A repeated-byte run contributes the given byte its stated number of times. Control 128 ends the compressed content and emits nothing. There is no special raw-data mode or copy history shared between chunks.
   - Backing: `measurable` (C1 and C3)
   - Confidence: high

6. **Terrain cell records.** Expanded terrain bytes hold successive 11-byte records with the following order of values in the file. There is no gap between values or records.

   | Position within a record | Width in bytes | Interpretation | Meaning |
   |---|---|---|---|
   | 0 | 2 | signed 16-bit little-endian | cell x |
   | 2 | 2 | signed 16-bit little-endian | cell y |
   | 4 | 4 | signed 32-bit little-endian | tile number before S-0009's translation |
   | 8 | 1 | unsigned | sub-tile number |
   | 9 | 1 | signed 8-bit | absolute height level |
   | 10 | 1 | unsigned | ice-growth flag value |

   Two coordinate values both equal to zero end the terrain stream. Only these four bytes are needed for the end marker, not a complete 11-byte record. Bytes after that coordinate pair have no effect. A valid stream need not contain every legal cell, nor need its records follow the map's visiting order. S-0009 claim 7 owns omitted cells, duplicate records, selected-cell aliasing and records that select no legal cell; claim 8 owns tile translation. The 32-bit tile number is not a 16-bit number followed by two unused bytes. The signed level and tile interpretations are F2's.
   - Backing: `measurable` (R1 and R2) for the record layout, nonnegative coordinate/tile values, signed height and terminator. Signed interpretations of negative coordinates and negative tile numbers are `unmeasured`: no scenario here separates signed from unsigned consumption without encountering an unspecified tile result or a selected-cell alias. The last byte's gameplay meaning is delegated to S-0009; R1 checks its file position, not ice growth.
   - Confidence: high

7. **Overlay arrays.** Each expanded overlay stream holds 262144 bytes: one byte for each of the 512 × 512 stream positions. Byte position 512·y + x names position (x, y), for integer x and y from 0 to 511. The type array's byte is the unsigned overlay list index, with 255 meaning no overlay. The data array's byte is an unsigned overlay data value, including at positions whose type is 255. There is no coordinate prefix, end marker or per-row delimiter. The map's own width and height do not shorten these arrays. Only legal cells receive their effects, and type placement precedes application of data as S-0009 claims 11 to 14 specify.

   Once the first 262144 expanded bytes have been consumed, additional bytes have no effect. A short type stream leaves the missing positions without overlay; a present, short data stream gives the missing positions data value 0. An absent stream is different from a present short data stream: absent data leaves the placement-produced data in force (S-0009 claim 13). Our importer warns about a short or overlong array.
   - Backing: `measurable` (R3 and B1)
   - Confidence: high

8. **Preview pixels.** Expanded preview bytes are RGB triples, with one byte per channel, all unsigned. Pixel (column *a*, row *b*) begins at byte position 3·(*b·w + a*) for the preview's imported width *w*. Its bytes are red, green, blue in that order. There is no palette, per-row delimiter, row-end gap or alpha channel. Exactly 3·*w·h* bytes are used, where *h* is the imported height. Later expanded bytes have no effect. The image's placement, preview markers and its use on the loading screen are S-0009 claims 24 and 25's.

   A preview ending before a pixel's third byte does not paint that pixel. The original keeps the complete pixels already painted and leaves the remaining pixels unspecified. Our engine keeps those complete pixels and paints all remaining pixels black, with an importer warning, as S-0009 claim 24 specifies. An overlong preview is ignored beyond its required bytes and receives an importer warning. Invalid dimensions are handled under Edge cases.
   - Backing: `measurable` (R4 and B1) for RGB layout, complete-pixel prefix preservation and the consumer limit. Not painting an incomplete pixel is `unmeasured`: its original background is unspecified, so B1 cannot distinguish an unchanged pixel from an accidental matching partial paint. Remaining original pixels have no deterministic colour claim.
   - Confidence: high for the layout; medium for the short-stream handling

9. **Truncated framing and terrain values.** If a chunk header has fewer than four bytes, or its compressed content has fewer than *C* bytes, the original exposes none of that chunk's expanded content. Already available expanded bytes from earlier complete chunks are retained. Our engine does the same, stops that block at the incomplete chunk, and warns; it does not search later bytes for another header. An otherwise valid compressed chunk can itself contain a short terrain record, short array or shortened preview. That is not a framing error.

   After at least one complete record has been read, a partial coordinate pair is formed by placing the available bytes in the low-order positions of a four-byte coordinate pair whose remaining bytes are zero. A partial tile number is handled by F3 whenever a complete coordinate pair selects a legal cell, including in the first record. The next coordinate pair starts with four zero bytes; before reading a tile number for a legal selected cell, its four bytes represent 65535. A missing one-byte value keeps the cell's value in force. Thus one or two coordinate bytes leave y = 0; three may give a nonzero y from its low byte and select a legal cell. Zero available tile bytes give 65535; one, two or three replace that many low-order bytes of the preset tile number (F3). The resulting tile number is then translated normally. Complete records already applied remain in force. Our engine reproduces these deterministic partial-value effects by default and warns (`quirks.partial_terrain_values`). With that switch off, an incomplete multi-byte value is treated as absent: a partial coordinate pair ends the stream, and a partial tile number uses the preset 65535. Complete one-byte values still take effect. Approved S-0009 revision 14's Terms and claim 7 already distinguish partial coordinates from partial tiles, including first-record partial tiles, and provide the quirk pointer and cut-off measurements. Its current revision 15 draft retains those rules. Joint sign-off checks claim 7 and scenario M4 against this claim, F3 and R2; the earlier revision-12 conflict is resolved. Missing one-byte values retain the selected cell's values in force, which may come from an earlier record, not necessarily from the initial fill.

   Ending terrain bytes between records without the coordinate end marker has the same terrain result as an end marker there; the importer warns about the missing marker. A complete end marker is allowed to be the last four expanded bytes, even when it crosses a chunk boundary. There is one exception to the partial-coordinate rule: if fewer than four expanded bytes are available for the very first coordinate pair, the original's coordinates are unspecified. Our engine applies no terrain record from such a block and warns, a safe departure without a switch. An absent terrain section never enters this case.
   - Backing: `measurable` (B1 and R2) for framing, partial coordinates, zero/two/three available tile bytes, missing sub-tile/height and end-marker handling. The exact one-tile-byte result is `unmeasured`: its value is at least 65280, and R2 has no stock tile or deterministic original picture that identifies that number rather than another unusable number. Retention of a missing ice-growth byte is `unmeasured`: the scenarios do not isolate ice regrowth. The unspecified first coordinate pair is `unmeasured`: B2 records outcomes but predicts no reproducible original coordinates.
   - Confidence: high for framing; medium for partial terrain values

10. **Bad compressed content.** The original does not reliably reject a missing compressed end marker, a copy before the beginning of the current chunk, expansion beyond its working limit, or disagreement between actual and advertised expansion lengths. Such files can read unrelated bytes, crash or yield unspecified output. They are not a means of supplying more map content.

    Our engine never uses undefined bytes to fill a chunk. A chunk is rejected when a control needs unavailable input, a nonempty copy begins before its expanded byte 0 or at a not-yet-produced position, it has no end marker, or the expansion length differs from *U*. Our supported chunks have 1 … 8192 expanded bytes and at least one compressed byte; other sizes are rejected and warned about. A compressed end marker before the end of the advertised compressed content also rejects that chunk. These are safe deterministic departures, not quirk settings. Keep the expanded bytes from earlier complete chunks and treat the rejected chunk and the rest of that block as absent content. This is a valid-prefix rule, not all-or-nothing rejection of the map. Apply claims 7 to 9's short-content rules to that prefix. Every such rejection receives an importer warning naming the affected block and chunk, without changing another block's decoding.
    - Backing: `unmeasured` for the original's unsafe-output range: B2 is exploratory and cannot establish a reproducible byte result or exhaust all failure modes. The deterministic safety outcomes are design requirements, tested on our engine in B2. Original prefix retention for incomplete framing, rather than unsafe complete chunks, is `measurable` under claim 9 (B1).
    - Confidence: high for the safety rule; medium for the range of original failures

## Formulas

### F1: text digits to bytes

For digit values *d₀, d₁, d₂, d₃*, the 24-bit unsigned integer is

> V = 262144·d₀ + 4096·d₁ + 64·d₂ + d₃.

The three byte values are

> ⌊V / 65536⌋, ⌊V / 256⌋ mod 256, V mod 256.

The divisions round down; all values are nonnegative integers. Missing digit places within a complete group hold zero. Claim 2 determines how many of these bytes are emitted. Base-64 digit places are measured in groups of six bits, and outputs in bytes.

### F2: little-endian and signed numbers

For *n* bytes *b₀* through *bₙ₋₁* in file order,

> unsigned value = Σ bᵢ·256ⁱ, for i from 0 through n − 1.

A signed value is the unsigned value when it is below 2⁽⁸ⁿ⁻¹⁾, otherwise the unsigned value minus 2⁽⁸ⁿ⁾. This gives two's-complement signed values. These are exact integer sums with no rounding. Coordinate units are cells, level units are height levels, and the tile value is an index or marker, not a byte count.

### F3: partial terrain tile number

Let *k* be the number of available tile-number bytes, from 0 to 4, and let *B* be the little-endian unsigned number formed by those bytes (zero for *k* = 0). The unsigned number before its signed interpretation and tile translation is

> T = B + 256ᵏ·⌊65535 / 256ᵏ⌋.

The division rounds down. For *k* = 0 this is 65535, for *k* = 1 it is 65280 + B, and for *k* ≥ 2 it is B. All arithmetic is exact nonnegative integer arithmetic; F2 then gives the signed 32-bit interpretation. This describes a short file value, not a program-memory layout.

### D1: distance-and-copy compressed controls

In this table *t* is the control byte. *q* is the next byte, or *a* and *b* are the next two bytes in file order. Where the table gives an extended length, zero bytes after the control add 255 each until a nonzero byte *v* occurs. Let *z* be that number of zero bytes. Those bytes and *v* are part of the length encoding, before any copy-distance bytes. All lengths, distances and literal counts are integer counts of bytes. Divisions round down; remainders are nonnegative.

| Control and context | Output length N | Copy distance D or literal interpretation | Trailing literals S |
|---|---|---|---|
| First control only, t > 17 | t − 17 | next N bytes literal; next context is post-literal | not encoded separately |
| t < 16, ordinary context, t > 0 | t + 3 | next N bytes literal; next context is post-literal | not encoded separately |
| t = 0, ordinary context | 18 + 255·z + v | next N bytes literal; next context is post-literal | not encoded separately |
| t < 16, post-literal context | 3 | 2049 + ⌊t / 4⌋ + 4·q | t mod 4 |
| t < 16, trailing-literal context | 2 | 1 + ⌊t / 4⌋ + 4·q | t mod 4 |
| 16 ≤ t < 32, t mod 8 > 0 | 2 + (t mod 8) | 16384 + 16384·(⌊t / 8⌋ mod 2) + ⌊a / 4⌋ + 64·b | a mod 4 |
| 16 ≤ t < 32, t mod 8 = 0 | 9 + 255·z + v | same distance as preceding row, using the two bytes after the extended length | a mod 4 |
| 32 ≤ t < 64, t mod 32 > 0 | 2 + (t mod 32) | 1 + ⌊a / 4⌋ + 64·b | a mod 4 |
| t = 32 | 33 + 255·z + v | same distance as preceding row, using the two bytes after the extended length | a mod 4 |
| 64 ≤ t ≤ 255 | 1 + ⌊t / 32⌋ | 1 + (⌊t / 4⌋ mod 8) + 8·q | t mod 4 |

The two rows for controls 16 … 31 instead mark the end when their distance expression is exactly 16384. In that case they emit no copy and no trailing literals. The normal end encoding is the three bytes 17, 0, 0. For a non-end copy, the following *S* bytes are literals, not new controls.

A non-end copy from either of those two rows needs a distance of at least 16385 bytes. It therefore cannot occur in a valid chunk under claim 10's supported limit of 8192 expanded bytes. Giving the table row does not permit borrowing bytes from earlier chunks or exceeding that limit; such content is rejected by our engine. The end marker remains valid within the limit.

### D2: context of the next distance-and-copy control

| Encoded item just completed | Next context |
|---|---|
| Start of chunk, no initial literal run | ordinary |
| Ordinary literal run or initial literal run | post-literal |
| Distance copy with S = 0 | ordinary |
| Distance copy followed by S = 1, 2 or 3 literals | trailing-literal |
| End marker | no further controls in this chunk |

These contexts determine the interpretation of controls below 16 only. Larger controls have the same meaning in every context, apart from the first-control-only row of D1.

### L: literal-and-copy compressed controls

*t* is the control byte. *q* is the next byte. *P* or *N* encoded in two bytes uses F2's unsigned little-endian rule. File order within a row is the control, the length when separately encoded, the copy position or repeated byte, then any literal bytes.

| Control | Output |
|---|---|
| 0 … 127 | Copy N = 3 + ⌊t / 16⌋ bytes at distance D = 256·(t mod 16) + q |
| 128 | End marker, no output |
| 129 … 191 | The next N = t mod 64 bytes literally |
| 192 … 253 | Copy N = 3 + (t mod 64) bytes from the following two-byte position P |
| 254 | Read two-byte length N and one byte v; emit N copies of v |
| 255 | Read two-byte length N and two-byte position P; copy N bytes from P |

Nonempty copies are valid only when their initial source byte has already been produced in this chunk. A zero-length repeated-byte run or absolute copy emits no bytes and needs no source byte. The arithmetic here is exact integer arithmetic, with divisions rounded down.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Packed-entry reader limit | 127 | characters per entry | Claim 1 |
| Writer's text line limit | 70 | characters per entry | Reader does not require this limit |
| Chunk length values | 2 each | bytes | Compressed length, expanded length |
| Writer's chunk size | 8192 maximum | expanded bytes | A shorter last chunk is normal |
| Terrain record length | 11 | bytes | End marker needs only 4 bytes |
| Overlay stream side | 512 | positions | Both axes |
| Overlay stream length | 262144 | bytes | Each of the two arrays |
| Preview channel count | 3 | bytes per pixel | Red, green, blue |

These are encoding constants, not copied map contents. Compression-control numbers and their units are in tables D1 and L.

## State and ordering

| Block state | Event | Result |
|---|---|---|
| No expanded content yet | First complete chunk | Its bytes become the beginning of the block |
| Expanded prefix available | Next complete chunk | Its bytes extend the same stream; a record or pixel may cross the boundary |
| Expanded prefix available | Short header, short compressed content, or rejected chunk | Keep prefix, stop this block, warn |
| Terrain content being interpreted | Coordinate pair (0, 0) | Later expanded terrain bytes have no effect |
| Overlay content being interpreted | 262144 bytes consumed | Later expanded bytes have no effect |
| Preview content being interpreted | Required pixel count consumed | Later expanded bytes have no effect |

Only file-visible ordering is fixed here: joined text has one order, and expanded chunks concatenate in that order. The tables and formulas carry the backing of the claims that name them. Text decoding and compression expansion add no random draws. Backing: `unmeasured`, because these scenarios observe final content, not stream advances isolated from all other loading effects. Confidence: high. Effects of overlay placement and terrain recalculation, including their draws, retain S-0009 and S-0010's ordering.

## Edge cases and quirks

- **Line limit, `quirks.map_pack_line_limit`, default original.** Retain only the first 127 characters of each retrieved entry value (claim 1). The alternate behaviour uses the full parsed value allowed by S-0001. Warn when the default loses characters. Re-splitting a valid long value into shorter entries is a repair that leaves its intended bytes unchanged. Backing: `measurable` (T1). Confidence: high.
- **Malformed complete base-64 groups, `quirks.map_base64_groups`, default original.** Claim 2's four-character interpretation, including skipped characters and exactly one output byte for zero, one or two collected digits. The alternate is strict rejection described there. Backing: `measurable` (T2). Confidence: medium.
- **Partial terrain values, `quirks.partial_terrain_values`, default original.** Claim 9 and F3. The alternate accepts only complete multi-byte values, retaining the same complete prefix. Backing: claim 9's part-specific backings (R2 for the measurable parts). Confidence: medium.
- **Absent versus empty content.** A block yielding no packed bytes has no content to apply. In particular, an empty or absent overlay data section does not zero the placement-produced data; a present nonempty packed stream whose first chunk is incomplete yields a short expanded stream and does zero missing data positions. This reproduces the original's distinction between no packed bytes and a nonempty packed stream. If strict text rejection treats the section as absent, that is the alternate text rule of claim 2. Backing: `measurable` (B1). Confidence: medium.
- **Invalid preview dimensions.** For nonpositive width or height, or dimensions whose required byte count cannot be represented by a signed 32-bit integer, the original may fail image creation or have an unspecified result. Our engine shows no embedded preview and warns, without rejecting the map's terrain or overlay blocks. This is a departure, not a switch. Rectangle parsing and absent preview size remain S-0009's. Backing: `unmeasured` for the original's unspecified result: B2 records possible failures but supplies no reproducible image hypothesis. The engine rule is a design requirement tested in B2. Confidence: medium for the original, high for the design rule.
- **Corrupt chunks.** Claims 9 and 10 keep only complete prior chunks. No partial copy or fabricated byte enters gameplay. The importer warns about framing errors, invalid control input, invalid copy references, unsupported chunk sizes, length disagreement and missing or misplaced compressed end markers. This is a departure wherever the original's result is unspecified, with no switch.

## Import mapping

| Map section or key | Parsed form | Encoding and result | When absent |
|---|---|---|---|
| `[IsoMapPack5]`, all entry values | trimmed packed text in enumeration order | base-64, then distance-and-copy chunks, then terrain records | initial terrain fill remains; S-0009 |
| `[OverlayPack]`, all entry values | same | base-64, then literal-and-copy chunks, then overlay type array | no map overlays |
| `[OverlayDataPack]`, all entry values | same | base-64, then literal-and-copy chunks, then overlay data array | keep values produced by placement |
| `[PreviewPack]`, all entry values | same | base-64, then distance-and-copy chunks, then RGB image | no embedded image data |
| `[Preview] Size` | four signed decimal integers; S-0003 and S-0009's rectangle parsing | third and fourth numbers are pixel width and height; first two ignored | S-0009 claim 30's rectangle default, not a size inferred from the packed bytes |
| `[Basic] NewINIFormat` | signed integer, default 0 | overlay arrays read only when at least 2; S-0009 claim 11 | 0 |
| `[IsoMapPack]`, `[IsoMapPack2]`, `[IsoMapPack3]`, `[IsoMapPack4]` | legacy streams | byte encodings outside this spec; precedence remains S-0009 claim 10 | no legacy contribution |

Section and key parsing, including ignored empty-value lines, remain S-0001 and S-0002's. Do not sort entry names numerically to implement this mapping.

## Measurement scenarios

All game tests use the stock program and stock rules and art, director and mods off, a purpose-made skirmish map with fixed starts for one human and one bystander computer player, game speed index 2, and a fresh program launch per variant. Keep the observed area away from the bystander; apply the glossary's void rule. File-only checks operate on the user's installed map data without changing it. Such checks identify encoding structure but do not alone confirm what the original consumes; game-output comparisons below supply the behavioural discriminators. Any output mismatch in a valid equivalent encoding falsifies the corresponding rule; failures of intentionally unsafe variants are exploratory, not formula confirmations. Test maps change only the named map bytes and their necessary re-encoding, not rule values. No test requires a network game. The scenarios require observations of encoded bytes, selection, and the first battlefield view, together with controlled test inputs. Any means used to produce the test maps or make those observations is outside this spec.

### T1 — joining, limits and normal base-64 (claims 1 and 2)

- **Setup:** choose three stock maps carrying the four blocks. Record each entry's key, character count and file order, and the joined decoded bytes. Check that the numbered values join into complete four-character groups, using the stated alphabet; compare the packed byte length with the lengths implied by the chunks. Use one tiny test map with a distinctive terrain patch and preview colours.
- **Procedure:** keep encoded text unchanged but rename its keys to nonnumerical names with distinct name hashes, then reverse only their numerical labels, then divide the same text at one-, two- and three-character positions of a group into short entries. Also exchange actual entry order. For duplicates, supply two different fragment values under the same key and enough other entries for S-0002 to predict the winner. Compare with a unique-key control containing that winning fragment at both enumerated positions; construct both joined streams to encode a valid, distinctive preview. Add leading and trailing whitespace to retrieved values, keeping groups complete after trimming. For a separate pair, merge entries into a value longer than 127 characters and compare with a copy re-split into entries of at most 70 characters. Test exactly 127 and 128 characters too.
- **Observable:** packed bytes, terrain patch cells and preview colour positions on loading. One launch per variant.
- **Expected:** renaming and re-splitting keep identical content; exchanging the actual entry order changes the joined content. The long value loses everything after character 127 in the original, whereas its split copy does not. Ordinary terminal `=` groups emit one or two bytes, not a zero byte for the unused place. Numerical sorting, line-local decoding or preservation of character 128 falsifies the relevant claim. Duplicate-key output must match the control with the winning fragment repeated; taking each occurrence's own value instead falsifies the lookup rule. Outer whitespace must not alter the content; a changed image falsifies trimming.

### T2 — nonstandard text (claim 2)

- **Setup:** a tiny preview and terrain patch with valid framed chunks. Make complete-group variants with each collected-digit count from 0 through 4, using internal spaces, `!`, `-` or `_` as skipped characters and `=` at different positions. Make three further variants with one, two or three characters in the final incomplete group. All skipped spaces must be internal to an entry, so text trimming does not remove them.
- **Procedure:** derive the expected packed bytes using claim 2 and F1. An explicit two-digit test re-encodes every packed byte *b* as a separate four-character group: digit ⌊b / 4⌋, digit 16·(b mod 4), then `!!`. Compare with the normal encoding of exactly the same complete chunk. Repeat with `==` instead of `!!`, including groups before later groups: each group must still emit exactly one byte. For the zero-digit case, replace a group encoding one zero byte by `!!!!` or `=!!!`. For the one-digit case, choose a packed byte divisible by 4 and replace its group by its first digit followed by `!!!` or `=!!`. Use such bytes in a literal preview payload, keeping chunk lengths and end markers valid under the predicted interpretation. Include three-digit groups for pairs of bytes and four-digit groups for triples. Run each once; record success, image, terrain or failure. Do not rely on a crash as the only discriminator between two valid interpretations.
- **Observable:** the resulting pixel colours or cell patch, and whether loading fails. Test the engine separately for warnings and the stated prefix result.
- **Expected:** complete malformed groups behave locally, not as global character filtering, and a later complete group is still decoded after `=`. Zero-, one- and two-digit groups each emit exactly one byte. The bytewise two-digit variants must reproduce the normal preview exactly; an extra byte per group, changed pixels or failure to display this otherwise valid preview falsifies the one-byte rule. The zero- and one-digit substitutions must also preserve the control's preview. Different valid content falsifies the corresponding complete-group hypothesis. Incomplete final groups have no predicted original pixels; the engine must warn and keep only earlier complete groups. That last expectation tests the departure, not an asserted original output.

### C1 — stock chunk framing and expansion (claims 3 to 5)

- **Setup:** the three stock maps from T1. Inspect all four blocks.
- **Procedure:** decode text, read each pair of length values, and expand every chunk by tables D1 and L. Record each compressed and expanded length and the final packed position. Compare terrain and preview output against the map's cell and image content; compare overlay arrays against visible overlays. Do not infer byte order solely from a length that fits in one byte.
- **Observable:** consumed compressed byte counts, expanded lengths, complete end markers, terrain record counts, both array lengths and preview pixel count.
- **Expected:** every chunk expands to its advertised length; packed positions advance by exactly 4 + C; arrays are 262144 bytes each; preview is 3·w·h bytes. Terrain ends with four zero coordinate bytes after its records. A big-endian length, shared copy history, different compression choice or row-end gaps producing a better match falsifies the corresponding claim.

### C2 — distance-and-copy controls (claims 3 and 4)

- **Setup:** a preview with deliberately chosen channel bytes, large enough to provide at least 2049 preceding bytes for the post-literal copy tests, and a terrain stream with two distinctive legal-cell records and its end marker. Encode equal expanded results in different ways, including a record and an RGB triple split across two chunks.
- **Procedure:** create valid files exercising every D1 row that fits within 8192 expanded bytes per chunk, extended lengths with zero, one and two extension-zero bytes, all trailing literal counts, and overlapping distances 1 and 2. Provide copy history within the same chunk, never in an earlier chunk. A below-16 post-literal copy needs at least 2049 preceding bytes: use an ordinary literal run long enough to supply them. A below-16 trailing-literal copy can use a short distance after a preceding copy with trailing literals. Include initial runs of one through four literals followed by a below-16 control as separate intentionally invalid files: none supplies the required history for a post-literal copy, including the four-literal case. For initial runs of one through three literals, contrast the invalid stock interpretation with the different, short-distance interpretation a generic decoder might offer; our engine must reject the stock-invalid reference. Non-end controls 16 … 31 cannot have sufficient history within the supported chunk size; test their rejection only, not a supposed valid large-preview equivalent.
- **Observable:** the complete preview RGB sequence, terrain patch and outcome on invalid initial-short files. One launch per valid variant, and one fresh launch per intentionally invalid variant.
- **Expected:** equal expanded bytes give equal images and terrain, despite changed chunk boundaries and controls. Lengths follow D1, and overlaps repeat the relevant preceding bytes. A valid variant painting different bytes falsifies the affected control row. The initial-run invalid-reference variants and unsupported far copies are failure tests: safe rejection in our engine, no claimed original colour and no formula confirmation from an original crash or successful load. The original's rare control contexts remain medium confidence pending these tests; enlarging the whole preview without enlarging its chunks cannot make a far reference valid.

### C3 — literal-and-copy controls (claims 3 and 5)

- **Setup:** two legal cells with visibly different, valid stock overlay types and data values. Other array positions have no overlay. Provide enough prior bytes within a chunk for each test copy position.
- **Procedure:** encode identical arrays with every row of L, including the shortest and longest control-coded lengths, separately encoded lengths above 255, absolute positions above 255, overlapping copies and repeated-byte runs. Add zero-length repeated-byte and absolute-copy variants, including a zero-length copy at a position not yet produced; each must consume its operands without output or a source-byte requirement. Divide the arrays at a non-row boundary into complete chunks. Make consecutive chunks start with different bytes and use an absolute copy of position 0 in the second chunk after producing its own initial bytes, so copying from the block's beginning instead would change a visible overlay.
- **Observable:** overlays, their data-dependent appearance in the first battlefield view, and the expanded arrays. One run per variant.
- **Expected:** equal arrays give equal cells regardless of compression choice; long lengths and positions are little-endian; history resets at each chunk. An unequal result from a valid encoding falsifies the corresponding row. Output from a zero-length item falsifies its length rule; copying the first chunk's byte for the second chunk's position 0 falsifies independent copy positions. A copy referring to a prior chunk must be rejected by our engine and is not a valid equivalent variant.

### R1 — terrain layout (claim 6)

- **Setup:** the stock terrain blocks from C1, plus a test map with two legal cells, a tile with multiple valid sub-tiles and a known height difference.
- **Procedure:** record expanded lengths and successive decoded values from the 11-byte table. In copies change just one coordinate, tile, sub-tile, level or flag byte, then re-encode. Coordinates above 255 distinguish the high byte, using a sufficiently large valid map. A separate copy changes one of the high tile-number bytes to make the value exceed the theater's tile count. Use levels 1 and −1 for the signed-byte comparison.
- **Observable:** which cell changes, its picture, sub-tile and height lift. For the high-tile-byte test use a non-overlay cell: S-0010 claim 7 predicts that an out-of-range positive tile becomes the no-tile marker, unlike the unchanged distinctive valid tile if the high bytes were ignored. A surviving distinctive tile falsifies 32-bit consumption. S-0010 claim 24's unspecified stray-tile picture is not a numeric discriminator; record it separately only if a variant uses the overlay case. Inspect the ice byte in the file; its gameplay effect is outside this spec.
- **Expected:** exactly the value named by that byte position changes. The high tile bytes affect the signed 32-bit number rather than being ignored. Stock lengths equal 11 times their record count plus the four-byte coordinate end marker. A 10- or 12-byte record or reversed byte order matching the file better falsifies the layout. Negative levels use the height rule of S-0009 and S-0106, not unsigned height.

### R2 — terrain terminator and cut-off values (claims 6 and 9)

- **Setup:** a tiny map with initial fill, one complete record and a final legal-cell record whose cell and tile values are known. Put any truncation in the *expanded* bytes of a complete, valid chunk. Use separate variants with all of the final record's first 0 through 11 bytes. For the three-coordinate-byte variant choose legal x and y below 256. Also test a first record with complete coordinates selecting a legal cell and only two or three tile-number bytes, with no earlier complete record. Choose low bytes that translate to a distinctive valid tile, and compare with a complete-record control for that tile.
- **Procedure:** compare with a full record and with a four-zero-byte end marker followed by a third distinctive record. Include duplicate records and omitted cells. For a truncated duplicate, let an earlier complete record set a sub-tile and level different from the initial fill, then cut the later duplicate before either one-byte value. Keep the truncated duplicate's complete tile number equal to that earlier record's valid multi-sub-tile tile, with the earlier sub-tile visibly distinct from the fill's sub-tile. Cut after all four tile bytes but before the sub-tile, and after the sub-tile but before the height. This distinguishes retaining the cell's values in force from restoring the fill without hiding the sub-tile under a no-tile result.
- **Observable:** first-record cell, final-record cell and the cell after the end marker, at the first battlefield view. One run per variant. Tile artwork and height lift identify the final cell's values; where a tile is out of range, record the original failure and use a variant whose low bytes name a valid tile to discriminate the formula.
- **Expected:** in variants with an earlier complete record, that record always survives. In the first-record partial-tile variants, the selected cell must show the distinctive valid tile of its complete-record control; a no-tile result instead falsifies first-record application of F3. Cutting after zero, one or two coordinate bytes must leave the chosen small-coordinate final cell unchanged; three bytes must select it. With two or three tile bytes, choose their low bytes to name a distinctive valid tile and compare with a complete-record control for that tile. A no-tile result instead of that valid tile falsifies F3 for those lengths. The one-tile-byte variant is exploratory only: its unusable result cannot identify the exact number. End marker stops the later record. Three coordinate bytes can select the legal final cell and set its tile to 65535. Available tile bytes give F3 before translation, while absent one-byte values retain their values in force, including an earlier duplicate's sub-tile and level. A change to a later record after the end marker, or disagreement with the valid two-/three-tile-byte controls, falsifies the corresponding claim. A missing sub-tile or level restoring initial fill instead of the earlier duplicate's value falsifies the retention rule; missing ice retention remains unmeasured. Our engine also tests the complete-value alternate and warnings.

### R3 — arrays (claim 7)

- **Setup:** the stock arrays from C1 and a tiny test map with one overlay on a legal cell and another at the next x position; choose a non-vein type with visible data-dependent appearance.
- **Procedure:** inspect their lengths, change only the two relevant byte positions, then swap x and y only in the chosen position. Independently expand the stock arrays and compare occupied positions with the map's visible overlay cells. Supply a data byte at a no-overlay position as well. Use short-array variants under B1.
- **Observable:** cell of each overlay, its data-dependent picture, and the two array lengths.
- **Expected:** positions use 512·y + x independently of the map's size; changing a type to 255 removes it, and a data byte does not create an overlay. Any row-length based on map width or coordinate prefixes matching better falsifies the layout.

### R4 — RGB image (claim 8)

- **Setup:** independently expanded stock previews from C1, plus a 3 × 2 test preview with different red, green and blue pixels and no preview start markers over them.
- **Procedure:** compare the stock expanded length with three times its imported width times height, and compare its decoded colours and row positions with the displayed selection image. For the test preview, encode its six pixels, change one channel byte, then re-encode with a chunk boundary after a pixel's first byte. Compare with a copy swapping the two rows. Record the displayed selection image and loading view once per variant.
- **Observable:** channel and position of each colour, and equality across changed chunk boundaries.
- **Expected:** six RGB triples, top row first, no row-end bytes; the changed channel changes only that channel of that pixel. Swapped channels or a shifted second row falsifies the layout. A chunk boundary does not break a pixel.

### B1 — short, long and incomplete blocks (claims 7 to 9)

- **Setup:** a map with a valid first chunk producing identifiable cells or pixels. Variants end the next header after one, two or three bytes, or remove its last compressed byte. Other variants have a valid final chunk but short expanded arrays or preview bytes, and still others append extra expanded content after each consumer's limit. Compare absent data text with nonempty text whose first chunk is incomplete. For that comparison, pre-check a stock overlay arrangement whose placement without a data stream produces a visibly nonzero data value; a zero-valued baseline cannot discriminate preservation from zero filling.
- **Procedure:** launch each original variant once and record the first terrain patch, overlays and preview. For preview cut after one and two bytes of a pixel. Re-run each on our engine, recording importer warnings.
- **Observable:** preserved prefix, missing-position outcomes, preview complete pixels, unchanged content before the consumer limit.
- **Expected:** incomplete framing contributes no expanded bytes from that chunk. Short type arrays omit overlays, present short data arrays supply zero after their end, absent data preserves placement-produced values. The original retains complete preview pixels; no colour is predicted for remaining original pixels. Our engine makes those black. Extra content has no effect and is warned about by our importer. A missing chunk contributing its prefix, loss of an earlier complete preview pixel, or extra consumed array bytes falsifies the respective original claim. The last incomplete original pixel is exploratory, not a paint/no-paint discriminator, because its background is unspecified.

### B2 — invalid content and safety (claim 10 and Edge cases)

- **Setup:** the valid-prefix map of B1, with a next chunk containing separately: a missing end marker, a truncated control's operands, a copy before byte 0, a copy at the next output position, incorrect expanded length, zero lengths, expansion above 8192 bytes, and trailing compressed bytes after the end marker. Add preview dimension variants that are zero, negative and give a required byte count beyond signed 32-bit range. Also test a nonempty terrain packed stream whose expansion supplies zero through three bytes for its very first coordinate pair.
- **Procedure:** one fresh original launch per malformed file, recording success, failure, hang or crash without assuming any particular garbage output. Test our engine independently.
- **Observable:** original failure mode and any surviving prefix; our engine's warning, complete prior content and deterministic replacement of missing content.
- **Expected:** no deterministic original output is claimed for unsafe compressed content. Any consistent acceptance should be recorded for a later revision, not treated as license to read undefined bytes. Our engine retains only prior complete chunks, never hangs or crashes, and warns. Invalid preview dimensions suppress only the preview. An incomplete first terrain coordinate pair applies no record in our engine; no original coordinates are predicted. Failure of those engine safety outcomes falsifies conformance to the design requirement, not the original's byte layout.

## Open questions

- The four legacy terrain encodings remain outside this spec; S-0009 claim 10 retains their precedence only.
- Valid original-game measurements of both below-16 copy contexts would improve claim 4's medium confidence. Initial one-to-three-literal runs followed by a below-16 control have only exploratory failure tests, not a discriminator for their exact original copy formula. The inspected stock files establish ordinary literal runs, short and medium copies, and end markers, but do not exercise those rare contexts. Non-end controls 16 … 31 cannot form valid copies within our supported chunk size and have only failure-mode tests.
- No stock file supplies malformed base-64 groups or inconsistent chunk lengths. Their failure-mode measurements must use named test maps; unsafe original output is deliberately not specified as reproducible bytes.
