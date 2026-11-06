---
id: Q-2686
title: Which file is the string table, how its texts decode, and the missing-label marker word
status: open
related: [S-0004, S-0005, S-0111, S-0213]
answered-by: []
---

## Need

S-0004 claim 17 (the answer to Q-2674) gives how a label is matched, normalised and resolved, and
S-0111 claims 21 to 23 show the results in the name tip. To use them our engine must read the
string table, and three things are not given:

- S-0005 Terms and claim 9 say the string table is one file, read once at startup while only the
  two language archives are mounted, but no spec names the file (its name, and whether the
  expansion's and the base game's tables are separate files of which one is read). S-0004 Open
  questions 8 says no formats spec gives the file's layout or the decoding of its texts.
- S-0004 claim 17 says a missing entry shows "a fixed marker word, a colon, and the requested label
  between single quotes", but not the marker word.
- How an entry's text is chosen when the file stores more than one text for a label, or none.

`crates/app/src/game/strings.rs` holds the claim 17 lookup on a table of (label, text) entries.
Our engine reads no string table yet, so every label shows our stand-in.

## Questions

1. Which file is the string table: its file name, and which language archive normally holds it?
   Do the expansion and the base game each have one, and if so is exactly one of them read?
2. What is the file's layout: its header, how entries are counted, how an entry label is stored,
   and how an entry's text is stored and decoded into text units (S-0014 Terms)? Is there a
   second, extra value stored with some texts, and is it ever shown?
3. An entry can store several texts or none. Which text does a lookup use when there are several,
   and what does it give when there are none: empty text, or the missing-entry placeholder?
4. What is the placeholder's marker word? Please describe it by its form (for example "one
   English word in capitals"), or give it as a fixed identifier the way S-0111's Import mapping
   gives `TXT_SHADOW`, not as quoted game text.
5. Does a string table that cannot be decoded behave as a missing one (S-0005 claim 9: the program
   refuses to start), or is it read as far as it can be?

## Possible observation

A loose probe string table (as S-0004's scenario L1 already uses) built with the layout the
answer gives: one label with two texts, one with none, and one probe type whose `UIName` names a
label the table lacks. Record the sidebar tooltip or the name tip of each probe type.

## Current workaround

No string table is read (`Session::strings` is `None`, cited `design: Q-2686`). A display name
whose label the table lacks, or any label while no table is read, shows our stand-in: the type's
section name for a type's display name and the label itself for a fixed entry identifier
(`strings::resolve`). `StringTable::from_csf` takes each label's first text, as XCC's reader does,
and an empty text for a label with none; it is used only by tests until this is answered.
