---
id: Q-0393
title: Where does a long status message wrap, and in which font is it drawn?
status: open
related: [S-0213, S-0411]
answered-by: []
---

## Need

S-0213 claim 25 says long imported text in the shared status display wraps to further lines,
each line with its own lifetime and each counting against the six-line capacity. It does not say
the width at which a line wraps, where a line may break, or the font that measures it. The client
(`play::say`, `session::draw`) cannot split a message into lines without these.

## Questions

1. At what width (in pixels, or relative to the battlefield's width) does a status line wrap?
2. Where may a line break: only at spaces, also at other characters, or inside a word when a
   word is longer than the width? Are leading spaces of a continuation line kept?
3. Which font and character spacing measure and draw status lines?
4. Does an explicit line break in the imported text start a new line?

## Possible observation

S-0213 Scenario O21's long-text variant: test string table entries of known lengths, with and
without spaces and explicit breaks, at 640 × 480 and 800 × 600.

## Current workaround

Each message is one line; our own messages are short texts that are not wrapped.
