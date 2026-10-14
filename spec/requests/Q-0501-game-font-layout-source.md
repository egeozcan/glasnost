---
id: Q-0501
title: Which public layout does the game bitmap font follow, and how are text units, the fallback glyph and colours mapped?
status: answered
related: [S-0005, S-0213]
answered-by: [S-0014]
---

## Need

S-0005 claim 18a (revision 6, answering Q-0451) says `game.fnt` "follows the public bitmap-font
file format" and adds no byte-layout contract. S-0213 claim 25 needs from that file each glyph's
width, the fallback glyph (an inverted capital X) and the glyph shapes for drawing status lines.

The only public, GPL-compatible format source an implementer may read is XCC's `misc/fnt_file.h`
with `t_fnt_header` in `misc/cc_structures.h` (GPL-3.0-or-later). That reader:

- accepts only two header variants, which it names as the first two games (`game_td`, `game_ts`),
  and names neither this game nor its expansion;
- counts characters in an 8-bit field (`c_chars + 1`, at most 256 entries), which cannot index
  S-0213's 16-bit text units directly;
- says nothing about a fallback entry or about drawing colours.

Neither XCC nor OpenRA has another font reader. So we cannot tell which document "the public
bitmap-font file format" means, or whether this game's file passes XCC's validity test at all.
We will not derive the layout from the real file or from memory.

## Questions

1. Which public document or reader gives the layout of this game's `game.fnt`? If it is XCC's
   `fnt_file.h`, which header variant (`id1`, `id4`) does the file carry? If it is another layout,
   please give its header, the covered range and the per-glyph width, height, offset and pixel
   storage (bit depth, row order, byte order).
2. How does a status text's 16-bit text unit select a glyph entry? When is a character
   *missing*: outside the covered range, an empty entry, a zero width, or something else?
3. Which entry is S-0213 claim 25's fallback glyph (the inverted capital X), and is its width that
   entry's stored width?
4. Which colours or palette indices do a glyph's pixel values select when a status line is
   drawn (S-0213 claim 25), and is any value transparent?
5. Does a carriage return or line feed ever take an advance (its glyph width plus 1), or does
   S-0213 claim 25's "ends a line after that character" mean it has none? (Q-0451 question 4,
   not addressed by S-0005 revision 6.)

## Possible observation

S-0005 Scenario M26 with a test copy whose header, widths and a fallback glyph are altered one at
a time; S-0213 Scenario O21 part b for widths and fallback use; a status notice containing a
character beyond the covered range.

## Current workaround

As under Q-0451: status lines are split and drawn with our own interface font's glyph widths
(`glasnost_ui::status::our_glyph_width`: the mask width, a space 2 pixels, `?` as the fallback).
A carriage return or line feed has no advance. No font file is read and the missing-font
diagnostic is not raised, since it would drop every status line.
