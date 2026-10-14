---
id: Q-0451
title: What is the layout of the game's bitmap font file?
status: answered
related: [S-0213, S-0005]
answered-by: [S-0005]
---

## Need

S-0213 claim 25 (Line splitting and font) measures and draws status lines
with the imported game bitmap font `GAME.FNT` (Import mapping): each glyph's
own width plus 1 pixel, a fallback glyph (an inverted capital X) for a
missing character, and the glyph shapes for drawing. No formats spec gives
the file's layout, so our engine has no reader for it. The splitter
(`glasnost_ui::status`) takes glyph widths from a provider; the game client
(`play::say`, `session::draw`) can only give it our own font's widths.

## Questions

1. How is the font file laid out: its header, the character range it
   covers, how a character code maps to a glyph, and how each glyph's width,
   height and pixel data are stored (bit depth, row order, byte order)?
2. How is the character code of a status text (a 16-bit text unit) turned
   into a glyph entry? Which entry is the fallback glyph, and when is a
   character "missing" (outside the covered range, an empty entry, a zero
   width)?
3. Which colours or palette indices does a glyph's pixel value select when a
   status line is drawn?
4. Is a tab's or a line break's own glyph width ever used, or do they always
   take the advances of claim 25 and F (64-pixel tab stops, no advance for
   carriage return and line feed)?

## Possible observation

A formats spec, like those for the sprite and palette files, written from
the file and its use; a named test copy with changed glyph widths (S-0213
Scenario O21 b) confirms the widths' role.

## Current workaround

Status lines are split by claim 25's rules with our own interface font's
glyph widths (`glasnost_ui::font`: the mask width of a glyph, a space 2
pixels, `?` as the fallback) and drawn in that font. The missing-font
diagnostic of the Import mapping is not raised, since no font file is read.
