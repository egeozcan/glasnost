---
id: S-0015
title: String table file layout and lookup results
topic: formats
revision: 4
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-06
measurements: []
requests: [Q-2686]
supersedes: []
---

# S-0015: String table file layout and lookup results

## Summary

This spec names the file that the program reads as its string table and gives that file's byte
layout, how an entry's stored texts are decoded into text units, which text a lookup gives when
an entry has several texts or none, the marker word of the missing-label placeholder, and what
happens when the file cannot be decoded. It answers Q-2686.

S-0005 owns where the file is found and when it is read (claims 9 and 16). S-0004 claim 17 owns
how a label is matched, the placeholder's form and the space normalisation of a text. This spec
starts where S-0004 claim 17 says "the entry labels as the string table file stores them" and
"the decoded entry texts".

## Terms

- **Text unit:** as in S-0014, one unsigned 16-bit value of a text.
- **Stored unit:** one unsigned 16-bit value as the file holds it, before decoding (claim 4).
- **Entry:** one label of the string table together with its texts (S-0004 Terms,
  *String-table entry, entry label*). An entry stores a text count, and so may hold several texts
  or none (claim 6).
- **Extended text:** a text stored with an extra value, a run of single-byte values that follows
  it in the file (claims 3 and 7).
- **Damaged entry:** an entry that the reading of claim 9 cannot take whole (claim 9 lists the
  cases).

## Behaviour

1. **Which file (Q-2686 question 1).** The program reads one string table, the file `ra2md.csf`,
   resolved as S-0005 claim 9 gives while only the two language archives are mounted. In stock data
   it is a member of the expansion's language archive `langmd.mix`. The base game's own string table
   `ra2.csf` is never read, wherever it is, so no lookup gives a text that only it holds: when
   `ra2md.csf` cannot be resolved, the program refuses to start (S-0005 claim 16). So exactly one
   table is read, the expansion's.
   - Backing: `measurable` (S-0005 scenario M16 (d) for a loose `ra2md.csf` being the table read,
     M16 (e) (i) for the refusal to start without `ra2md.csf`, scenario C1 (i) for no lookup giving
     a text that only `ra2.csf` holds)
   - Confidence: high

2. **Header.** The file begins with a 24-byte header. Its first four bytes, in file order, are
   decimal 32, 70, 83, 67, spelling a space and `FSC`. The other five values are 32-bit integers
   in little-endian byte order, read as signed:

   | First byte, counted from 0 | Length in bytes | Meaning |
   |---|---|---|
   | 0 | 4 | identifying bytes above |
   | 4 | 4 | format version |
   | 8 | 4 | entry count |
   | 12 | 4 | text count, all entries together |
   | 16 | 4 | unused value |
   | 20 | 4 | language code |

   The format version, the unused value and the language code have no effect on what is read or
   on any text shown. The program refuses to start, as for a string table that cannot be resolved
   (S-0005 claim 16), when the file is shorter than 24 bytes, when its first four bytes differ
   from those above, or when the entry count or the text count is 0 or below.
   - Backing: `measurable` (scenario C2 parts (i) to (iv), (vii) and (viii) for the refusals, (v)
     and (vi) for the version, unused value and language code having no effect, C1 for the layout)
   - Confidence: high for the layout and the refusals for a short file, wrong identifying bytes
     and a count of 0. Medium for a count below 0, and for the version, unused value and language
     code having no effect, since only two values of each are run in C2.

3. **Entries.** The entries follow the header one after another, with no gap, to the end of the
   file. An entry is:
   - Four identifying bytes, decimal 32, 76, 66, 76 (a space and `LBL`).
   - The entry's text count *c*, a signed 32-bit little-endian integer.
   - The label length *L* in bytes, a signed 32-bit little-endian integer.
   - *L* label bytes, single-byte values stored as they are (no terminator, not complemented).
   - Then *c* texts, one after another. A text is four identifying bytes, either decimal 32, 82,
     84, 83 (a space and `RTS`: a plain text) or decimal 87, 82, 84, 83 (`WRTS`: an extended
     text); then its unit count *n*, a signed 32-bit little-endian integer; then *n* stored units,
     each an unsigned 16-bit little-endian value. An extended text then has its extra length *m*
     in bytes, a signed 32-bit little-endian integer, and *m* bytes of extra value.

   The reading of entries ends normally when fewer than four bytes remain where the next entry would
   start, so one to three trailing bytes are ignored. The entries need not be in any order. Their
   order in the file matters only for claim 6's entry with no texts, for two labels that match each
   other (claim 5) and for which entries are kept when the reading stops at a damaged entry or at
   the header's counts (claim 9).
   - Backing: `measurable` (scenario C1: every probe entry built with this layout is found, C3
     (ii) for an end of file between entries). `unmeasured` for one to three trailing bytes being
     ignored: the original keeps the entries before them whether it ends there normally or stops
     as at a damaged entry (claim 9), so no run can tell the two apart.
   - Confidence: high, except medium for the trailing bytes.

4. **Decoding a text (Q-2686 question 2).** The text unit of a stored unit *s* is its bitwise
   complement, *u* = 65535 − *s* (exact integer arithmetic, unsigned 16-bit domain). The decoded
   text is the units decoded in order, ending after the *n*-th unit or just before the first
   stored unit equal to 0 or to 65535, whichever comes first. A stored 0 or 65535 therefore ends
   the text, and the units after it are never shown. The decoded text then goes through S-0004
   claim 17's space normalisation, once, when the table is read. A unit count of 0 gives an empty
   text.
   - Backing: `measurable` (scenario C1 (a) to (c) for the complement, C1 (d) and (e) for the
     early end, C1 (h) for a unit count of 0, S-0004 scenario L2 for the normalisation)
   - Confidence: high, except medium for the space normalisation, as S-0004 claim 17 rates it.

5. **Entry labels.** An entry's label is its *L* label bytes, cut just before the first zero
   byte among them if there is one. It is matched against a requested label as S-0004 claim 17
   gives. An entry label of 32 bytes or more after that cut matches no lookup: in the original it
   matches no requested label of 31 bytes or fewer, and in our engine it matches none, and the
   importer warns. Two entries whose labels are equal, or differ only in letter case, follow
   S-0004's Edge case for labels that differ only in letter case: which one a lookup finds is
   unspecified in the original, and our engine uses the one that comes first in the file and
   warns.
   - Backing: `measurable` (scenario C1 (f) for the cut at a zero byte). `unmeasured` for a label
     of 32 bytes or more: no requested label in the set-ups here is longer than 31 bytes (the
     display-name label is cut to 31, S-0004 claim 17), so no run tells such an entry apart from
     one with a different label. `unmeasured` for which of two equal labels is found, as S-0004
     gives for labels that differ only in case.
   - Confidence: medium

6. **Several texts or none (Q-2686 question 3).** A lookup whose label matches an entry gives
   that entry's first text. Its later texts are read and kept, and no lookup ever gives them. A
   lookup whose label matches an entry with no texts gives empty text in our engine, not the
   missing-label placeholder, since the label is present. The original has a display bug there:
   it gives the first text of the next entry in the file that has a text, and an undefined result
   when no later entry has one. This is a drawing-only defect (lead decision 2026-10-14) and has
   no quirk switch.
   - Backing: `measurable` (scenario C1 (a) for the first text, C1 (b) for the original's next
     entry's text). Empty text for an entry with no texts is our engine's design rule, not an
     oracle prediction.
   - Confidence: high for the first text and for the original's next entry's text. Medium for the
     original's undefined result when no later entry has a text, which is a reading of the
     original that no run here checks.

7. **The extra value (Q-2686 question 2).** An extended text carries an extra value (claim 3).
   No lookup in the program asks for it, so it is never shown and has no effect: an extended text
   shows exactly as the same text stored plainly. Our engine reads the extra value and discards
   it.
   - Backing: `measurable` (scenario C1 (c))
   - Confidence: high

8. **The missing-label placeholder (Q-2686 question 4).** When no entry matches (S-0004 claim
   17, *Missing entry*), the text is: the marker word (Constants: seven capital English letters
   meaning "not present"), a colon, an apostrophe (text unit 39), the requested label with each of
   its bytes as the text unit of the same value, and an apostrophe. The placeholder does not go
   through the space normalisation of claim 4, so every space of the requested label is kept.
   - Backing: `measurable` (S-0004 scenario L1 (h) for the form, L1 (m) for a byte above 127
     showing as the unit of the same value, scenario C1 (g) for the marker word and the kept
     spaces)
   - Confidence: high for the form and the marker word. Medium for a byte above 127 and for the
     kept spaces.

9. **A table that cannot be decoded (Q-2686 question 5).** In our engine the header's checks (claim
   2) are the only ones that stop the program from starting. In the original that holds for case (i)
   below and for a file that ends between entries (scenario C3), and its result in cases (ii) to
   (iv) is undefined, as given below. With a valid header, the entries are read in file order and
   the reading stops at the first damaged entry. The entries read whole before it are kept, the
   program starts, and a lookup of a label that only the entries not read hold gives the
   placeholder. A later entry not read whose label repeats a kept entry's label changes nothing: a
   lookup of that label gives the kept entry's text (claim 5). Our engine also gives the importer a
   warning naming the place where the reading stopped. A damaged entry is one that:
   - (i) does not start with the four identifying bytes of claim 3.
   - (ii) holds a text whose four identifying bytes are neither of claim 3's.
   - (iii) has a text count, label length, unit count or extra length below 0.
   - (iv) runs past the end of the file.

   The original stops in the same way, with the same result, in case (i). In cases (ii) to (iv) its
   result is undefined, and our engine's result above is a departure for bad data (glossary). Two
   further cases of data that no stock file holds:
   - Entries or texts beyond the header's entry count or text count give an undefined result in the
     original. Our engine reads entries only up to the header's entry count, and stops before an
     entry whose texts would take the texts read past the header's text count. It keeps the entries
     read whole before that point and gives the importer a warning, as at a damaged entry. This is a
     departure for bad data (glossary).
   - A label, extra value or text far longer than any in stock data can give an undefined result in
     the original. Our engine reads labels, extra values and texts of any length the file holds,
     with no length limit, and a long label then matches as claim 5 gives. This is our engine's
     design rule for data the original was not built for.

   When the file holds fewer entries or texts than the header's counts, the entries read whole are
   used as above, in the original and in our engine, and our engine gives the importer a warning. In
   the original a lookup of an empty label then has an undefined result. No lookup of an empty label
   is known (a type without a label gets empty text with no lookup, S-0004 claim 17), and in our
   engine a lookup of an empty label is matched as claim 5 gives, like any other: it gives the first
   text of the first kept entry whose label is empty, empty text if that entry has no texts (claim
   6), and the placeholder if no kept entry has an empty label.
   - Backing: `measurable` (scenario C3 (i) for case (i) and for a valid header with a damaged entry
     not stopping the program from starting, C3 (ii) for fewer entries than the header's count).
     `unmeasured` for cases (ii) to (iv), for entries or texts beyond the header's counts, for the
     long values and for the empty label: the original's result is undefined there, or no known
     lookup reaches it, so no run can confirm a result. Our engine's results there are design rules.
   - Confidence: high for case (i), for the program starting and for fewer entries being used.
     Medium for the original's results being undefined in the other cases, for entries or texts
     beyond the header's counts, for far longer values and for the empty label.

## Formulas

All positions are byte positions counted from file byte 0, all lengths are in bytes unless
stated, and all arithmetic is exact integer arithmetic.

- The first entry begins at byte 24. An entry that begins at byte *e*, with label length *L*,
  ends just before byte *e* + 12 + *L* + Σ *t*, the sum running over the lengths *t* of its texts,
  and the next entry begins there.
- A plain text takes *t* = 8 + 2*n* bytes. An extended text takes *t* = 12 + 2*n* + *m* bytes.
- Within a text that begins at byte *p*, stored unit *i* (0 ≤ *i* < *n*) is the 16-bit
  little-endian value at bytes *p* + 8 + 2*i* and *p* + 9 + 2*i*, and its text unit is
  65535 − that value.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Header length | 24 | bytes | Claim 2 |
| File identifying bytes | 32, 70, 83, 67 | byte values | Claim 2 |
| Entry identifying bytes | 32, 76, 66, 76 | byte values | Claim 3 |
| Plain text identifying bytes | 32, 82, 84, 83 | byte values | Claim 3 |
| Extended text identifying bytes | 87, 82, 84, 83 | byte values | Claim 3 |
| Decoding | 65535 − stored unit | text unit | Claim 4 |
| Text-ending stored units | 0 and 65535 | stored unit values | Claim 4 |
| Longest label that a lookup can match | 31 | bytes | Claim 5 |
| Missing-label marker word | `MISSING` | fixed identifier, not quoted game text | Claim 8: seven capital English letters, then a colon |

## State and ordering

The table is read once, at start-up (S-0005 claim 9), and the space normalisation of each text is
applied then (claim 4). Nothing reads it again during a game or between games. No simulation state
or tick ordering is involved.

## Edge cases and quirks

- An entry with no texts: the original shows the next entry's text, a drawing-only defect with
  no switch. Our engine gives empty text (claim 6).
- A damaged entry in cases (ii) to (iv) and more entries or texts than the header gives: departures
  for bad data with an importer warning (claim 9). An entry label of 32 bytes or more (claim 5), and
  fewer entries or texts than the header gives (claim 9): read as those claims give, with an
  importer warning. A far longer label, extra value or text: read whole, our engine's design rule
  (claim 9).
- Two entries with equal labels: as S-0004's Edge case for labels that differ only in case
  (claim 5).

## Import mapping

| Input | Imported concept | Form |
|---|---|---|
| `ra2md.csf`, resolved by S-0005 claim 9 | the string table | layout of claims 2 and 3 |
| a header that fails claim 2's checks | as a table that cannot be resolved | S-0005 claim 16 |
| a damaged entry and those after it | none | importer warning (claim 9) |
| entries or texts beyond or short of the header's counts | as in claim 9 | importer warning (claim 9) |
| `ra2.csf` | none | never read (claim 1) |
| header format version, unused value, language code | none | read and ignored (claim 2) |
| entry label bytes | entry label | single-byte values, cut before a zero byte (claim 5) |
| stored units of an entry's first text | that entry's text | claim 4, then S-0004 claim 17 |
| an entry's later texts and every extra value | none | read and ignored (claims 6 and 7) |

No rules-file key controls this file.

## Measurement scenarios

All three scenarios use the set-up of S-0004 scenario L: its common set-up of K (a tiny
purpose-made skirmish map, game speed 2, crates off, stock data, no engine add-on and no mods,
the named test files of a scenario and the purpose-made test maps excepted), buildable copies of
one stock vehicle that differ only in their display-name field, and the observable of each copy's
display name in its sidebar tooltip and its name tip (S-0111 claim 20). The named test file is a
loose probe `ra2md.csf` in the game directory (S-0005 claim 9: a loose file outranks the language
archives), built with the layout of claims 2 and 3. Unless a part says otherwise it holds every
stock entry unchanged, adds the probe entries named below with distinct short probe texts of
plain letters, and its header's entry count and text count equal the entries and texts in the
file. Remove it after each run. One run per part, in a fresh session.

**C1, decoding and choice of text (claims 1 and 3 to 8).** Probe entries:
- (a) label *Q1* with two texts, *t1* then *t2*.
- (b) label *Q2* with no texts, followed directly in the file by label *Q12* with no texts and then
  by label *Q3* with one text *t3*.
- (c) label *Q4* whose one text *t4* is an extended text with an extra value of four letters,
  and label *Q5* with the same *t4* as a plain text.
- (d) label *Q6* whose one text is the stored units of a word *w*, then a stored 0, then the
  stored units of another word, and label *Q7* whose one text is *w* alone.
- (e) label *Q8*, as *Q6* with a stored 65535 in place of the stored 0.
- (f) label *Q9*, stored as the bytes of *Q9*, a zero byte and two letters, with the label
  length counting all of them, and one text *t9*.
- (g) No entry: two copies whose display-name labels are absent from the table, one holding two
  adjacent spaces between two words and the other the same words with one space.
- (h) label *Q10* with one plain text whose unit count is 0, placed in the file after an entry
  with a text and before another entry with a text.
- (i) No probe entry: a second loose file `ra2.csf` in the game directory, built with the layout of
  claims 2 and 3, holding one entry with label *Q11*, which neither the probe `ra2md.csf` nor any
  stock table holds, and one text *t11*. Remove it after the run.

One copy names each label (*Q2*, *Q12* and *Q3* each have their own copy). Expected: (a) *t1*. (b)
In the original, the copies of *Q2* and *Q12* both show *t3*, and in our engine they show no name
(S-0111 claim 20). (c) The copies of *Q4* and *Q5* show the same text with the same box width, and
no letter of the extra value shows. (d) and (e) The copies of *Q6* and *Q8* each show *w* only, with
a box as wide as *Q7*'s copy's. (f) *t9*. (g) Both show the marker word (Constants), a colon and the
label between apostrophes, and the two-space copy's box is wider than the one-space copy's. (h)
*Q10*'s copy shows no name (S-0111 claim 20). (i) *Q11*'s copy shows the placeholder of claim 8 with
the label *Q11*. Falsified by *t2* in (a), by any text other than *t3* for *Q2* or *Q12* in the
original in (b), by any difference between the two copies in (c), by anything after *w* in (d) or
(e), by the placeholder in (f), by another marker word or equal box widths in (g), by any text or
the placeholder for *Q10* in (h), and by *t11* in (i). A loose file outranks both language archives
(S-0005 claim 9), so a program that opened `ra2.csf` would read the probe's.

**C2, header checks (claim 2).** Variants of the probe table, without probe entries: (i) its first
byte changed to another letter; (ii) the file cut to its first 20 bytes; (iii) the entry count set
to 0; (iv) the text count set to 0; (v) the unused value and the language code each set to 9; (vi)
the format version set to 1; (vii) the entry count set to −1; (viii) the text count set to −1.
Procedure: start the program, one fresh start per variant. Observable: whether the main menu
appears, and if it does, the labels of its buttons. Expected: in (i) to (iv), (vii) and (viii) the
main menu never appears, as in S-0005 scenario M16 (e) (i). In (v) and (vi) the main menu appears
with the stock labels. Falsified by a main menu in (i) to (iv), (vii) or (viii), or by none, or
changed labels, in (v) or (vi).

**C3, reading stopped by damage (claims 3 and 9).** The probe table holds probe entry *Q1* with one
text *t1* first in the file, then the stock entries, and its header's entry count and text count are
those of the stock table plus *Q1* and its one text, so they count every entry and text of the full
file. (i) The first stock entry after *Q1* has its first identifying byte changed to another letter.
(ii) The file ends right after *Q1*'s entry, with the header unchanged. Procedure: start the
program, read the main menu, then start the skirmish and build *Q1*'s copy. Observable: whether the
main menu appears, its buttons' labels, and *Q1*'s copy's display name. Expected: in both variants
the main menu appears, each of its buttons shows the placeholder of claim 8 with its own label (or
as much of it as the button shows) in place of its stock label, and *Q1*'s copy shows *t1*. Our
engine also gives an importer warning. Falsified by a refusal to start, by a stock button label, or
by *Q1*'s copy showing the placeholder.

## Changes for other specs

- **S-0004, Open question 8 and claim 17.** Open question 8 is answered by this spec: claims 2
  and 3 give the file's layout and claim 4 the decoding of its texts. Claim 17's *Missing entry*
  bullet can name the marker word by pointing to claim 8 here, and its *Matching* bullet can
  point to claim 5 here for the entry label as stored (cut before a zero byte). S-0004's Edge
  case for labels that differ only in letter case can also cover two equal labels (claim 5).
  Confidence: medium, as claim 5 rates the equal labels. Backing: `measurable` for the layout and
  decoding (scenario C1), `unmeasured` for the equal labels (claim 5).
- **S-0005, Terms, claim 15 and Import mapping.** The *String table* term can name the file
  `ra2md.csf` and point to this spec for its layout. The Import mapping can add a row for
  `ra2md.csf` (string table, claims 9 and 16, layout in S-0015; stock source `langmd.mix`). Claim 15
  can add that a base game string table `ra2.csf`, wherever it is, is never read (claim 1, scenario
  C1 (i)). Claim 16's refusal for a string table that cannot be resolved also covers a file that
  fails claim 2's header checks here. Confidence: high. Backing: `measurable` (S-0005 scenario M16
  (d) and (e), C1 (i), C2).

## Open questions

- Whether any lookup asks for a label of 32 bytes or more (claim 5). Every fixed label that the
  program itself looks up has 31 bytes or fewer. Labels that come from data, such as the mode
  list's (S-0006 claim 13), have not all been checked for their text limits. If one can be longer,
  a measurement would decide whether an entry label of 32 bytes or more can match it in the
  original.
