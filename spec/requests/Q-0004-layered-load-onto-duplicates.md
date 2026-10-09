---
id: Q-0004
title: Which occurrence a lookup returns after a layered load into a table with a repeated name hash
status: open
related: [S-0002]
answered-by: []
---

## Need
S-0002 claims 6, 9 and 10 define the layered load (a second file read into a
document that already has sections). When a table that the load changes
already holds two occurrences with the same name hash (a key written twice,
two non-empty sections with one name, or two names with one hash), or ends
up holding them, S-0002 leaves open which occurrence later lookups return
(Open questions, first item; State and ordering). The importer must give a
deterministic result for every input. Code gap: `glasnost_ini::Document::layer`.

## Questions
1. After a layered load replaces one occurrence of a key that the section
   holds twice, which occurrence does a lookup of that key return: the new
   entry, or the occurrence that was not replaced? Does the answer depend on
   the other keys of the section, as the hash-order rule of a fresh load does?
2. Where does an entry that a layered load adds or replaces sit in the order
   that decides the winner, when that order is not the entry order (S-0002
   State and ordering)?
3. The same two questions for the table of section names: a layered load
   that adds a section to a document holding two non-empty sections with one
   name, or a layered load whose new key or section shares a name hash with a
   different existing name.
4. If the original's result cannot be stated as observable behaviour, which
   deterministic result should our engine use (a departure), and should it
   carry a quirk switch?

## Possible observation
S-0002 scenario D8 already records one outcome for the first question; a
variant with a third, unrelated briefing-section key chosen so that the
fresh-load winner changes would show whether the other keys matter.

## Current workaround
With `quirks.ini_duplicate_hash_order` on, `Document::layer` refuses such a
load (an error naming the section and line) and leaves the document
unchanged. With the switch off the result is defined by S-0002 (the last
occurrence wins). The original reads no stock file by a layered load
(S-0002 claim 6), and campaign missions, the only case that uses one, are
outside the current scope.
