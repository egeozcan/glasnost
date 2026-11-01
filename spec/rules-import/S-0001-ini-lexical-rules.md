---
id: S-0001
title: INI lexical rules
topic: rules-import
revision: 7
sources-class: [decomp, measurement, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-01
measurements: [M-0012]
requests: [Q-0478]
supersedes: []
---

# S-0001: INI lexical rules

## Summary

How the original turns the bytes of a rules, art or map document into sections and entries: line
splitting, line length, section headers, comments, whitespace trimming (including a trimming defect),
empty values, and case sensitivity. How several entries with the same name are resolved is
S-0002; how a value's text is turned into a number, flag or list is S-0003.
Implementation scope is skirmish; network games and campaigns are out of scope.

## Terms

- **Document.** One INI-format file after it has been read: an ordered set of sections.
- **Line.** The bytes between two line-feed bytes (byte value 10), or between the start of the
  file and the first line feed. The bytes after the last line feed, if any (or the whole file, if it
  has no line feed), form a final line with no line feed after it (claim 5).
- **Whitespace byte.** Any byte with a value from 1 to 32 inclusive: space, tab, other control
  characters. Bytes 128 to 255 are never whitespace.
- **Trim.** Removing whitespace bytes from both ends of a byte string. The original's trim has a
  defect, described in claim 10 and in Formulas.
- **Section.** A named group of entries, introduced by a header line `[name]`.
- **Entry.** One key/value pair inside a section, from a line `key=value`.
- **Key, value.** The trimmed text before and after the first `=` of an entry line.
- **Fresh load.** Reading a file into an empty document. This is how every stock rules, art, AI,
  sound, speech and map file is read. Layering a second file onto a non-empty document is S-0002.
- **Name hash.** The number, with 2³² possible values, that S-0002 derives from the bytes of a section name or key.
  Lookups identify names by this number alone, so two different names with the same name hash are
  one name (S-0002 claim 3, `quirks.ini_hash_identity`); that is the one exception to claim 13. This
  spec otherwise uses the hash only to choose collision-free names in the measurement scenarios.

Also used, with the meaning given in the shared glossary: *the original*, *YR* and *YR behaviour*,
*stock data*, *measurement setup*, *engine add-on*, *loose file*, *field*, *absent-key default*,
*type*, *override section*, *rules layer*, *reset on override*, *importer* and *quirk switch*. In short: *the
original* is the reference release of the game (version 1.001 as shipped, its program together with
its stock data), *YR behaviour* is what that release does, defects included, and *stock data* is that
release's unmodified data files.

## Behaviour

1. **Bytes, not characters.** A document is a byte string with no character encoding. Bytes 128 to
   255 are ordinary name and value bytes. A byte-order mark is not recognised. Its bytes are not
   whitespace, so when a file starts with one and its first line is a section header, that line is not
   a header (claim 6): the first section's header is lost, and its entries are ignored as if they came
   before any header (claim 7). If the first line is blank or a comment, the mark has no effect.
   - Backing: `measurable` (scenario L1)
   - Confidence: high

2. **Line splitting.** A line feed ends a line. A CR byte (value 13) is deleted wherever it
   appears, also in the middle of a line, so "a, CR, b" reads as "ab". Both LF and CR LF files
   therefore work. CR alone never ends a line: two lines separated only by a CR are read as one line.
   A file that uses CR alone as its line separator is one single line, cut by claim 3 and, having no
   line feed after it, ignored by claim 5.
   - Backing: `measurable` (scenario L2)
   - Confidence: high

3. **Line length limit.** Only the first 511 bytes of a line are kept. CR bytes, which claim 2
   deletes, do not count towards the limit. The rest of that line is discarded, and the next line is
   read normally.
   Consequences:
   - a value that runs past the limit is cut short,
   - an entry line whose `=` lies beyond the limit is not an entry line at all,
   - a section header whose `]` lies beyond the limit is not a header (claim 6). This counts any
     indentation before the `[`, so a header line is a header only if its first `]` is among its
     first 511 bytes, and a section name can be at most 509 bytes long.
   - Backing: `measurable` (scenario L3)
   - Confidence: high

4. **NUL bytes.** A byte of value 0 inside a line ends that line's content. The bytes after it, up
   to the next line feed, are ignored.
   - Backing: `measurable` (scenario L2)
   - Confidence: high

5. **Unterminated final line.** In a fresh load, the last line of a file is ignored if no line feed
   follows it. A file whose final entry has no trailing newline loses that entry. One stock data file
   ends this way, so its last entry is not seen by the original (see Import mapping).
   Quirk `quirks.ini_final_line_needs_newline`.
   - Backing: `measured` (M-0012) for one key whose line is the last line of the rules file read
     at a skirmish's start: without a line feed after it the game behaves as if the key were
     absent. `measurable` (scenario L4) for the same key read with a line feed after it, and for
     other files and keys.
   - Confidence: high

6. **Section headers.** A line is a section header when its first non-whitespace byte is `[` and the
   line contains a `]` somewhere after it. The section's name is every byte strictly between that `[`
   and the first following `]`, taken as it is:
   - spaces inside the brackets are part of the name (`[ Alpha ]` names the section " Alpha ", not
     "Alpha"),
   - `;` and `=` inside the brackets are part of the name,
   - the name may be empty (`[]`),
   - everything after the `]` is ignored, including comments and text such as `=1`.

   A line starting with `[` that has no `]` is not a header. It is handled as an ordinary line by
   claims 8 and 9: it is ignored unless it contains `=`, in which case it is an entry whose key starts
   with `[` (`[x=5` in a section gives key "[x", value "5").
   - Backing: `measurable` (scenario L5) for header recognition and its effect on the current
     section. The exact stored name of an empty or punctuation-bearing section and the exact key
     stored from a malformed header are `unmeasured`: L5 observes the cost lookup, not an enumeration
     of these names, and no stock consumer of those particular names is identified here.
   - Confidence: high

7. **Lines before the first header** belong to no section and are ignored.
   - Backing: `measurable` (scenario L5)
   - Confidence: high

8. **Comments.** In any line that is not a section header, the first `;` and everything after it form
   a comment. No byte of the comment can be part of a key or value, or can make the line an entry
   line. This holds wherever the `;` stands, also inside a value. A value can therefore never contain
   `;`, and there is no escape for it. A line whose first byte is `;` is a full comment line. A
   comment's presence also changes how the rest of the line is trimmed (claim 10).
   - Backing: `measurable` (scenario L6)
   - Confidence: high

9. **Entry lines.** Considering only the text before any comment (claim 8):
   - a line containing no `=` is ignored,
   - otherwise the line splits at its first `=`: the key is the part before it, the value the part
     after it, and later `=` bytes belong to the value (`a=b=c` gives key "a", value "b=c"),
   - a line whose text before the first `=` is empty or all whitespace (for example a line starting
     with `=`) has an empty key and creates no entry (claim 11); its text is not re-split at a later
     `=`,
   - key and value are each trimmed (claim 10); whitespace inside a key or value is kept (`my key`
     is a valid key, different from `mykey`).
   - Backing: `measurable` (scenario L11)
   - Confidence: high

10. **Trimming, and the trimming defect.** Trimming removes leading and trailing whitespace bytes,
    except in one case: when the text has more leading whitespace bytes than core bytes, but not more
    than core and trailing bytes together, some trailing whitespace survives (exact rule in Formulas).
    Effects:
    - **Keys:** a key written with a deep indent and a space before `=` keeps a trailing space, and is
      then never found under its intended name. An indent of 5 spaces, a 4-byte key, then a space
      before `=` leaves the key as the 4 bytes plus one space.
    - **Values:** a value can keep trailing whitespace in the same way. Text reads (S-0003) trim
      again without the defect, so they are unaffected. The integer reader recognises a hexadecimal
      suffix only when it is the value's final byte (S-0003), so for it the surviving space matters: 4
      spaces, then `1Fh`, then 2 spaces is read as decimal 1, not hexadecimal 31.
    - **Comments:** on an entry line that carries a comment, the key is always trimmed fully,
      whatever the indentation. The value can still keep trailing whitespace in rare layouts (a deep
      indent, a short key and value, spaces after the `=` and before the `;`); the exact rule is in
      Formulas. The entry from the key example, with a trailing `;` comment, is read correctly.

    Quirk `quirks.ini_trim_defect`.
    - Backing: `measurable` (scenario L7) for the key and integer-value effects. Text-value
      re-trimming inherits S-0003 claim 1's backing. That no stock input triggers the defect is
      `unmeasured`: it is a property of the input files, not an outcome isolated by L7.
    - Confidence: high

11. **Empty keys and empty values.** If the key or the value is empty after trimming, the line
    creates no entry. `key=`, `key=   ` and `key= ; note` all behave exactly as if the line were
    absent: the field gets whatever a missing key gives it (see *absent-key default*, *rules layer*
    and *reset on override*). They do not set an
    empty string or zero. This is a key-independent lexical rule: there are no exceptions for
    particular keys, including the country name-prefix key (Import mapping I5). A later field read
    may reset an omitted field when its section exists (S-0006 claim 5), but an empty written value
    still supplies no entry and cannot request a different result from omission. Stock rules data
    has about two hundred such lines.
    - Backing: `measurable` (scenario L8; L9d tests the name-prefix key's lexical treatment)
    - Confidence: high

12. **Empty sections do not exist.** A section that ends up with no entries (none written, or all
    dropped by claims 9 and 11) does not exist once the load is complete. Asking whether it exists gives
    "no", exactly as for a section never written. In particular, an empty section never competes with
    a non-empty section of the same name in the same file: lookups always reach the non-empty one,
    whichever of the two comes first (duplicate sections are S-0002). A country section containing
    only an empty name-prefix line is therefore absent, not a section that resets or clears a field.
    - Backing: `measurable` for section absence (scenario L9); `unmeasured` for the country-prefix
      field consequence, which follows S-0006 claims 2 and 3 but has no known play observable
      (S-0413 claim 4)
    - Confidence: high

13. **Case sensitivity.** Section names (as claim 6 gives them, untrimmed) and keys (as trimmed by
    claim 10) are matched by their exact bytes, except where S-0002's name-hash collision rule applies (S-0002 claim 3,
    `quirks.ini_hash_identity`): two different names with equal name hashes are one name, and a
    lookup of either can find the other's entry or section. No stock key or section name shares a
    name hash with a different name the original looks up, so in stock data matching is by exact
    bytes throughout. Upper and lower case are different (`[Alpha]` is not found as `ALPHA`, and
    `key` is not found as `Key`). Stock data contains keys whose case differs from the spelling the original reads, and
    those entries have no effect (Import mapping). Type names used as *values* are matched without
    regard to case; that is S-0003.
    - Backing: `measurable` (scenario L10). The hash exception inherits S-0002 claim 3's
      `measurable` backing (its scenario D4); no measurement record is claimed here. The stock-file
      collision and misspelling observations are `unmeasured`: they are properties of the input
      files and the names read, not outcomes isolated by L10.
    - Confidence: high

14. **Counts and file order.** There is no fixed maximum number of sections per document
    or entries per section; only available memory limits them. In particular, a document with
    5000 filler sections followed by the test section, and 5000 filler entries followed by the test
    entry in that section, accepts the test entry (5001 sections and 5001 entries respectively).
    Entry order (the order an enumeration of a section's keys returns, and the order in which
    S-0002's duplicate rule sees a section's entries) is file order. The same holds for the order of
    a document's non-empty sections.
    - Backing: `measurable` for the stated finite counts (L12a) and duplicate-resolution order
      (L12b, L12c and S-0002 claim 2's scenario D2). The absence of any fixed count maximum is
      `unmeasured`: no finite input can distinguish no limit from a larger untested limit.
      Enumeration order inherits S-0002 claim 4's backing (its scenario D3).
    - Confidence: high

## Formulas

**Trim with the original's defect.** Let a byte string consist of *L* leading whitespace bytes, a
core of *C* bytes (starting and ending with a non-whitespace byte), and *T* trailing whitespace bytes.
The trimmed result is the core followed by the first *K* of the trailing whitespace bytes, where

> *K* = *L* − *C*  if *C* < *L* ≤ *C* + *T*
>
> *K* = 0  otherwise

*L*, *C*, *T* and *K* are non-negative integer byte counts. The formula has no division, rounding or
floating point. All-whitespace strings trim to the empty string. With `quirks.ini_trim_defect` off,
*K* = 0 always.

Examples (· = space):

| Raw text | L | C | T | Result |
|---|---|---|---|---|
| `··k··` | 2 | 1 | 2 | `k·` |
| `·····abcd·` | 5 | 4 | 1 | `abcd·` |
| `····abcd·` | 4 | 4 | 1 | `abcd` |
| `·······k··` | 7 | 1 | 2 | `k` |
| `k···` | 0 | 1 | 3 | `k` |

**Entry lines that carry a comment.** Let the text before the comment consist of *L*₁ leading
whitespace bytes, a core of *C*₁ bytes and *T*₁ trailing whitespace bytes, and let *K*₁ be the *K* of
the formula above with *L* = *L*₁, *C* = *C*₁ and *T* = *T*₁. The key is trimmed with no surviving whitespace. The value's
raw text is the core's bytes after its first `=`, followed by the first *K*₁ of the *T*₁ trailing
whitespace bytes; the value is that raw text trimmed by the formula above. Domain as above.

Example: `··········k=···v····;` has *L*₁ = 10, *C*₁ = 6, *T*₁ = 4, so *K*₁ = 4. The key is `k`. The
value's raw text `···v····` has *L* = 3, *C* = 1, *T* = 4, so *K* = 2 and the value is `v··`.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Line length limit | 511 | bytes | Kept per line; CR bytes do not count (claim 3) |
| Longest section name | 509 | bytes | Follows from the line length limit and the two brackets |
| Highest whitespace byte | 32 | byte value | Bytes 1 to 32 are whitespace |
| Comment byte | 59 | byte value | `;` |
| Assignment byte | 61 | byte value | `=` |

## State and ordering

Line classification during a fresh load. "Current section" starts as *none*.

| Current section | Line, as claims 2 to 4 leave it | Result |
|---|---|---|
| any | final line with no line feed after it | Ignored (claim 5, with `quirks.ini_final_line_needs_newline` on) |
| any | first non-whitespace byte is `[` and a `]` follows | A new current section with the bracketed name begins, and the previous one (if any) ends. A section with no entries is absent from the finished document (claim 12) |
| none | anything else | Ignored |
| a section | text before any comment has no `=` | Ignored |
| a section | key (text before the first `=`) or value empty after trimming | Ignored |
| a section | otherwise | Entry appended to the current section, in file order |
| a section | end of file | The load is complete. Sections with no entries are absent (claim 12) |

## Edge cases and quirks

- `quirks.ini_final_line_needs_newline` (YR default: on). In a fresh load, the last line of a file is
  ignored unless a line feed follows it (claim 5). Off: the final line is read like any other.
- `quirks.ini_trim_defect` (YR default: on). Trimming keeps *K* trailing whitespace bytes as given in
  Formulas. Off: trimming removes all leading and trailing whitespace. Only text with deep
  indentation is affected; stock data never triggers it.
- Byte-order mark: not a quirk switch. With the mark directly before the first header on the
  same line, that header is lost (claim 1). No switch because stock data never carries the mark and
  the importer warns on every file that does (Importer warnings). That stock data never carries the
  mark is `unmeasured`: it is a property of the input files, not an outcome of the original that a
  measurement isolates. Whether a switch is wanted is a question for the project lead (see
  Questions for the project lead).
- CR-only line endings: not a quirk switch. The whole file is one line (cut at 511 bytes) with no line
  feed after it, so nothing loads (claims 2, 3 and 5). No switch for the same reason: stock data
  never uses CR-only endings and the importer warns. That stock data never uses CR-only endings is
  `unmeasured`: it is a property of the input files, not an outcome isolated by a measurement.
  Whether a switch is wanted is a question for the project lead (see Questions for the project lead).

These behaviours apply to every INI-format input, stock data, user maps and mods alike, and both
quirk switches stay on by default for all of them.

**Importer warnings.** The importer warns (see *importer warning* in the glossary) on each of the
following. A warning never changes the imported result, which stays the one the claims above give.
- A file that starts with a byte-order mark (the 3 bytes 239, 187, 191), whether or not the mark
  costs the file its first header (claim 1).
- A file that contains a CR byte not directly followed by a line feed. A file with CR bytes and no
  line feed at all is the CR-only case above; a stray CR elsewhere is deleted (claim 2) and joins its
  neighbours.
- A key whose bytes differ from a key the importer reads for that section only in the letter case of
  bytes A to Z and a to z, so the entry has no effect (claim 13; rows I2 and I3 of Import mapping).
- A key or value whose trimmed result keeps trailing whitespace under the trimming defect, that is,
  wherever *K* in Formulas is above zero for the key or for the value (claim 10).
- A final line with no line feed after it that would have been a section header or would have
  created an entry had a line feed followed it, because that header or entry is lost (claim 5).

Duplicate sections and duplicate keys also warn; that warning belongs to S-0002.

The warnings and switch-off results are design requirements, `unmeasured` in the original,
which supplies neither these warnings nor these alternatives. Confidence: high. Engine acceptance
checks use L1a/L1b for the mark warning, L2b/L2c for the stray-CR warning, L10a for the case warning,
L7a/L7d for the trim warning, and L4a for the final-line warning. The corresponding unchanged control
inputs must not produce those warnings. With the final-line switch off L4a must give *V*; with the
trim switch off L7a must give *V* and L7d/L7f must give 31 credits. Any other result fails the check.

## Import mapping

These rules apply to every INI-format input: the rules, art, AI, sound and speech files, game-mode
files, and the INI part of map files.

Stock data that the rules above affect:

| Row | Stock fact | Effect under YR rules |
|---|---|---|
| I1 | The art file ends without a newline after its final line, a numbered entry of the `[Movies]` section | That entry is ignored (claim 5) |
| I2 | Rules file: `JumpJetAccel` and `JumpJetTurnRate` (the original spells both with a lower-case "j" in "jet"), `Maxdebris` (the original reads `MaxDebris`) | These entries have no effect (claim 13) |
| I3 | Sound file: `Fshift`, `Vshift`, `volume` (the original reads `FShift`, `VShift`, `Volume`) | These entries have no effect (claim 13) |
| I4 | About 210 rules lines, 230 speech lines, 30 sound lines and 3 art lines have an empty value (`Key=`) | Each behaves as if absent (claim 11) |
| I5 | `Prefix` in a country's rules section, including `Prefix=`, `Prefix=   ` and `Prefix= ; note` | Empty after trimming: no entry, exactly as omission, with no key-specific exception. Alone it creates no section (claims 11 and 12). Nonempty text and the absent-key field default belong to S-0413 claim 4 |

Row I2 and I3 were found by comparing stock keys with the key spellings the original looks up. The
list may be incomplete for keys whose spelling the original builds at run time.

The measurement scenarios below write the infantry cost field under its rules key `Cost` (4 bytes),
called the *cost key* there. The filler names they use (`Filler1`, `Filler2` and so on) are not
keys the original reads.

The stock-file facts in I1 to I4, including the approximate counts, are `unmeasured` (confidence:
high): they describe the supplied files, not a gameplay result isolated by these scenarios. Their
lexical consequences inherit claims 5, 11 and 13's backing.

## Measurement scenarios

**Common setup.**
- *Install and data:* the measurement setup of the glossary, that is an unmodified install with no
  loose override files and no engine add-ons, using stock data only.
- *Map:* a tiny purpose-made two-player multiplayer map, played as a skirmish against one computer
  house, kept away from the player's base and test buildings under the glossary's bystander
  computer player void rule. No engine add-on and no mods (the named test files of a scenario and
  the purpose-made test maps excepted). Its INI part carries an override section
  for one buildable infantry type of the player's side, the *test type*. The section is named
  exactly as the test type is spelled in the rules file. Select a country and type whose sidebar
  price equals its cost field, confirmed by a normal test-line control. Any game speed.
- *Test line:* the cost key (Import mapping), `=`, and a value *V*: a positive whole number of credits in
  integer form (S-0003) that differs from the test type's stock cost. Unless a row says otherwise,
  the override section holds the test line and nothing else, and every line ends with a line feed.
  The numbers in the rows (123, 150, 1Fh and so on) are examples. Replace any that equals the test
  type's stock cost.
- *Procedure and tool:* start the named map variant as a skirmish, make the test type available
  through the normal build prerequisites, and hover its sidebar icon. Record the displayed cost
  manually or with a screen capture. No change to a harness is required.
- *Observable:* the test type's cost in credits, as the sidebar shows it once the player's house can
  build the test type. "Stock" in
  the table means the stock cost, so the test line was not seen. "*V*" means it was seen.
- *Runs:* reading a file is deterministic and uses no random draws, so each map variant needs
  **one run** and its own game start. A row with multiple variants needs one start per variant.
  Otherwise it needs one start.
- *Falsifier:* in every row, any cost other than the expected one falsifies the claim. The last column
  names the result that the most likely misreading would give.

Spaces are written as · where their number matters.

| Row | Claim | Map variant | Expected cost | Falsified if |
|---|---|---|---|---|
| L1a | 1 | The file starts with the 3-byte UTF-8 byte-order mark, directly followed by the override section's header (the override section is the first section of the file) | stock | *V* |
| L1b | 1 | As L1a, but a full comment line `; test` comes between the mark and the header | *V* | stock |
| L1c | 1 | Prefix the cost key with one byte of value 128 to 255; one variant for each byte | stock in all 128 runs (the extra byte is part of the key, not whitespace) | *V* in any run |
| L1d | 1 | Prefix the integer value *V* with one byte of value 128 to 255; one variant for each byte | 0 credits in all 128 runs (the retained byte prevents an initial decimal number, S-0003 claim 3) | *V* in any run |
| L1e | 1 | As L1a, but a line feed immediately follows the mark, leaving the header on the next line | *V* | stock |
| L2a | 2 | Every line of the file ends with CR LF | *V* | stock |
| L2b | 2 | The value is written as `12`, one CR byte, `3` | 123 | 12 or stock |
| L2c | 2 | The header and the test line are separated by a CR byte alone (no line feed between them) | stock (one header line; the text after `]` is ignored) | *V* |
| L2d | 4 | The value is written as `12`, one NUL byte, `34` | 12 | 1234 or stock |
| L3a | 3 | 520 spaces between the cost key and `=` | stock | *V* |
| L3b | 3 | The value *V* is followed by spaces up to a line length of 600 bytes | *V* (the cut falls inside the spaces) | stock |
| L3c | 3 | Directly before the test line, a filler line `Filler1=` followed by 600 digits | *V* (the next line is read normally) | stock |
| L3d | 3 | The header is indented by 509 − *N* spaces, where *N* is the length of the test type's name in bytes, so its `]` is byte 511 of the line | *V* | stock |
| L3e | 3 | As L3d with one more space of indentation, so the `]` is byte 512 | stock | *V* |
| L3f | 2, 3 | As L3d, with 20 CR bytes inserted among the indentation spaces | *V* (CR bytes do not count) | stock |
| L3g | 3 | The cost key, `=`, then enough leading zero digits that the following `1Fh` ends at byte 511 | 31 | 1 or stock |
| L3h | 3 | As L3g, with one extra leading zero so the suffix `h` is byte 512 | 1 (the retained value ends in `1F`, read as decimal) | 31 |
| L4a | 5 | The override section is the last section, and the test line is the last bytes of the file, with no line feed after it | stock | *V* |
| L4b | 5 | As L4a, with a line feed after the test line | *V* | stock |
| L5a | 6 | The header is written with a space after `[` and before `]` | stock | *V* |
| L5b | 6 | The header is followed by `···; note` on the same line | *V* | stock |
| L5c | 6 | The header is followed directly by `=1` on the same line | *V* | stock |
| L5d | 6 | The header has `;` just before its `]` | stock | *V* |
| L5e | 6 | A line `[]` between the header and the test line | stock (the test line now belongs to the section with the empty name) | *V* |
| L5f | 6 | Between the header and the test line, a line made of `[`, the cost key, `=` and 150, with no `]` | *V* | stock (the line ended the section) or 150 |
| L5g | 7 | The test line is moved above every section header of the file | stock | *V* |
| L6a | 8 | The value is written as `1Fh;123` | 31 | 1 (comment retained, so the hexadecimal suffix is no longer final) or stock |
| L6b | 8 | The test line starts with `;` | stock | *V* |
| L6c | 8, 11 | The value is written as `;1Fh` | stock (the value before the comment is empty) | 0 or 31 |
| L7a | 10 | The test line is indented by 5 spaces, with one space between the cost key and `=` (claim 10's key example) | stock | *V* |
| L7b | 10 | As L7a, with `··; note` appended | *V* | stock |
| L7c | 10 | As L7a, indented by 4 spaces instead | *V* | stock |
| L7d | 10 | The value text is `····1Fh··` | 1 | 31 |
| L7e | 10 | The value text is `1Fh` (control for L7d) | 31 | 1 or stock |
| L7f | 10 | 16 spaces, the cost key, `=`, `·····1Fh····` and then `;` | 1 (the comment case of Formulas keeps two spaces) | 31 |
| L8a | 11 | The cost key and `=`, with nothing after it | stock | 0 |
| L8b | 11 | As L8a, followed by `··; note` | stock | 0 |
| L8c | 11 | The cost key, `=` and three spaces | stock | 0 |
| L9a | 12 | Two sections named after the test type: first an empty copy, whose header is followed only by L8a's line, then the normal override section with the test line | *V* | stock |
| L9b | 12 | As L9a, with the two copies in the opposite order | *V* | stock |
| L9c | 12 | L9a and L9b again, with the empty copy holding no lines at all | *V* in both | stock in either |
| L9d | 11, 12 | L9a and L9b again, but the empty copy holds only the country name-prefix key (I5), `=` and, in separate variants, no text, three spaces, or a space and a comment | *V* in all six runs: lexical treatment does not depend on whether a key belongs to the section's type | stock in any run |
| L10a | 13 | The cost key written all in lower case | stock | *V* |
| L10b | 13 | The section name written all in upper case (pick a test type whose name contains a lower-case letter) | stock | *V* |
| L11a | 9 | The test line is the cost key, `=`, 150, `=`, 45 | 150 (the value `150=45` is read in integer form up to the `=`, S-0003) | stock (split at the last `=`) or 45 |
| L11b | 9 | The test line starts with an extra `=` before the cost key | stock (empty key, no re-split) | *V* |
| L11c | 9 | The cost key with a space between its second and third bytes | stock | *V* |
| L11d | 9 | The cost key, a space and *V*, with no `=` | stock | *V* |
| L12a | 14 | Before the override section, 5000 filler sections `[Filler1]` to `[Filler5000]`, each holding one line `Filler1=1`. Inside the override section, 5000 filler lines `Filler1=1` to `Filler5000=1` before the test line, which comes last | *V* | stock |
| L12b | 14 | The override section holds two test lines and nothing else: the cost key with value 150, then the cost key with value 160 | 150 | 160 |
| L12c | 14 | As L12b, with the two lines in the opposite order (160 first, then 150) | 160 | 150 |

Why some rows can falsify their claim:
- **L9.** If an empty copy did exist, S-0002 claim 2 would pick one of the two copies by name hash and
  position alone. The two copies have the same name, so swapping them swaps the copy that is found.
  The empty copy would then be found in exactly one of L9a and L9b, and that row would show the stock
  cost.
- **L10.** The lower-case cost key does not share a name hash with the cost key, so L10a tests case,
  not a collision. Before running L10b, check in the same way that the upper-case section name does
  not share a hash with the test type's name.
- **L12a.** The names `Filler1` to `Filler10000` were checked against S-0002's name hash: none shares
  a hash with the cost key, nor with another filler name. Before running, also check that none shares
  a hash with the test type's name. The original reads no filler key. There are 5001 sections before
  any other required map sections are counted, and 5001 entries in the test section. A limit that
  discards the test section or its last entry would show the stock cost; a refusal to load also
  falsifies the finite-count prediction. The run cannot prove that there is no limit at all. It
  bounds any limit from below.
- **L12b and L12c.** These are S-0002's fixture 1 and its mirror. The duplicate rule sees only name
  hashes and the entry order. Both lines have the same hash, so the winning position is the same in
  both rows, and the winning value follows the order in the file. The same value winning in both rows
  would mean that entry order is not file order. File order of sections is used by S-0002 scenario
  D2's prediction, which tests it.

## Cross-spec hand-off

Q-0478's lexical questions were already answered by claims 11 and 12: an empty value creates no
entry for any key, and an entryless section is absent. This revision makes the country-prefix
case explicit. S-0006 claims 2, 3 and 5 already distinguish an absent section from an omitted
field in an existing section. S-0413 claims 1 and 4 and its Import mapping must remove their
contradictory empty-prefix exception and require the same result as omission: keep the held
prefix. No other key needs a lexical exception. Prefix field parsing and its default remain
owned by S-0413 claim 4.

## Open questions

- Whether any stock loader reads a file that relies on the final-line quirk, other than the art file.
  Maps edited by hand often lack a final newline. This doesn't block the spec.

## Questions for the project lead

- Should the byte-order mark (claim 1) and CR-only line endings get a quirk switch? The draft proposes
  none. The reasons are given in Edge cases and quirks.
