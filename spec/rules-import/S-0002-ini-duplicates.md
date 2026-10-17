---
id: S-0002
title: INI duplicates and name resolution
topic: rules-import
revision: 8
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-17
measurements: []
requests: [Q-0004, Q-0183]
supersedes: []
---

# S-0002: INI duplicates and name resolution

## Summary

What the original does when a document names the same key twice in a section, or the same section
twice; how two different names can collide; how type lists behave with repeated keys or
values; and what happens when a second file is layered onto an already-loaded document, including
which occurrence of a repeated name wins afterwards. The
lexical rules (what counts as a section, key and value) are S-0001.

Implementation scope is skirmish. Network games and campaigns are out of scope. Existing
campaign material below identifies a diagnostic route for the general document rules, not a
requirement to implement campaigns.

## Terms

Also used, with the meaning given in the shared glossary: *section*, *entry, key, value*, *the
original*, *YR behaviour*, *stock data*, *measurement setup*, *loose file*, *field*, *type*, *type
kind*, *registry*, *registration index*, *type list*, *type section*, *override section*, *rules
layer*, *map layer*, *base rules*, *importer* and *quirk switch*. In addition (the first two as in
S-0001):

- **Document.** One INI-format file after it has been read: an ordered set of sections.
- **Fresh load.** Reading a file into an empty document. Every stock rules, art, AI, sound, speech,
  game-mode, mission information and map file is read this way.
- **Enumeration.** Listing a section's entries by position, in the section's entry order, instead
  of looking a key up by name. Type lists are read by enumeration.
- **Name.** A section name or a key, as a byte string after S-0001's trimming.
- **Name hash.** A 32-bit number computed from a name (Formulas). The original identifies names by
  this number alone.
- **Occurrence.** One appearance of a name in the file. A key written twice in a section has two
  occurrences.
- **Name table.** The set of names that one lookup searches. Each section's keys form one table.
  The section names of a document form another.
- **Winning occurrence.** The occurrence that lookups of a duplicated name return.
- **Layered load.** Reading a second file into a document that already has sections. This is not
  the same as applying a rules layer (claim 7): a layered load changes one document, while rules
  layers are separate documents read one after the other.
- **Mission information file.** The data file that gives each campaign mission its display name,
  briefing text and loading-screen settings, in one section per mission named after the mission's
  map file.
- **String-table label.** A value that names a piece of display text instead of holding the text
  itself; the game shows the text the label stands for. A *display-name label* is the label of a
  type's or a mission's display name.
- **Copy (of a section).** One of the sections of one file that share a section name.
- **Hash-order rule.** The rule that picks the winning occurrence of a duplicated name, defined in
  Formulas together with the *reference sort* and the *bisection* it uses.
- **Fixture.** One of the purpose-made override sections of rows D-c to D-e (Import mapping).
- **Test map, test type.** The purpose-made map and the infantry type that the measurement scenarios
  use (Measurement scenarios, common setup).
- **Working order.** The order in which a name table's occurrences stand for the bisection. After a
  fresh load it is file order. Lookups and layered loads change it (State and ordering).
- **Ordered table.** A name table whose working order was put in reference-sort order and has gained
  no occurrence since.
- **Remembered occurrence.** The occurrence that a name table's latest successful lookup returned,
  for as long as the table keeps it (State and ordering). A table remembers at most one occurrence.
- **Peak count.** The largest number of occurrences a name table has held so far. Removing an
  occurrence never lowers it. Together with the forgetting peaks (Constants) it decides only when the
  table forgets its remembered occurrence.

## Behaviour

1. **Duplicate keys are kept, but only one is visible to lookups.** When a key occurs more than
   once in one section of one file, every occurrence stays in the section's entry order (S-0001
   claim 14), and enumeration lists each of them. A lookup by name always returns the value of one
   particular occurrence, the winning occurrence, chosen by the hash-order rule (Formulas). The
   winner is neither reliably the first nor reliably the last occurrence. It depends on the hashes
   of *all* other keys in that section, so adding or removing an unrelated key can change which
   duplicate wins. Quirk `quirks.ini_duplicate_hash_order`.
   - Backing: `measurable` (scenario D1)
   - Confidence: high

2. **Duplicate sections are not merged.** When a section name occurs more than once in one file,
   each occurrence is a separate section. Lookups of that name find exactly one of them, chosen by
   the same hash-order rule applied to the document's table of section names. Every key lookup and
   every enumeration then uses that one copy only: keys that appear only in another copy are
   invisible, as if never written. A copy that is empty (S-0001 claim 12) is not part of the table of
   section names, so it never competes. Quirk `quirks.ini_duplicate_hash_order`.
   - Backing: `measurable` (scenario D2)
   - Confidence: high

3. **Names are identified by hash alone.** Two different names with the same name hash are the same
   name to the original. A lookup of either can return the entry (or section) of the other, decided
   by the hash-order rule as if they were duplicates. With 32-bit hashes this is rare (no name table
   of any stock data file, maps included, holds a collision: claim 8), but it is possible. Quirk
   `quirks.ini_hash_identity`.
   - Backing: `measurable` (scenario D4)
   - Confidence: high

4. **Repeated keys in a type list.** A type list has one position for each entry that enumeration
   lists, in entry order, and each position holds the value that a lookup of that entry's key *by
   name* returns, which for a duplicated key need not be the value written on that entry's own
   line. A duplicated key therefore yields
   the winning occurrence's value once for every occurrence, and the losing occurrences' values are
   never seen. For example, if key 3 is
   written twice, first with value Alpha and then with value Beta, and the first occurrence wins, the
   list contains Alpha at both positions and Beta not at all. Quirk
   `quirks.ini_duplicate_hash_order`: with it off, the winning occurrence is the last one
   (Formulas), so every position of a duplicated key holds the last occurrence's value.
   - Backing: `measurable` (scenario D3)
   - Confidence: high

5. **Repeated values in a type list.** When a type list names the same type twice (under different
   keys), the second mention finds the existing type and does not create another one. The type is
   registered at its first mention, and later mentions leave its registration index unchanged. Type
   names match case-insensitively (S-0003). Type registration as a whole is specified in S-0004.
   - Backing: `measurable` (scenario D3)
   - Confidence: high

6. **Layered loads: the later file wins.** When a second file is read into a document that already
   has sections, a lookup of a section and key that both files give returns the second file's value.

   Stock data never causes a layered load. The one layered load found in the original happens when a
   campaign mission starts: if a file named after the mission's map file (the same name with the
   extension `.INI`) exists and has at least one section with an entry, the mission information
   file is layered onto it, so the mission-named file is the earlier file and the mission
   information file the later one. The release ships no file with such a name for any of its
   missions, in any archive or as a loose file, so in stock play the mission information file is
   always read by a fresh load. Mods and test files can trigger the layered load.
   - Backing, per item:
     - the second file's value wins for a section and key that both files give: `measurable`
       (scenario D5),
     - the mission-named file is read and takes part in the layered load: `measurable` (scenario D7
       variant (a), which shows that the earlier file was read),
     - a mission-named file with no section that has an entry (for example one that holds only a
       section header; whether a line with an empty value counts as an entry is S-0001's rule,
       claim 11) gives a fresh load, not a layered one: `measurable` (scenario D7 variants (e) and
       (f)). The two kinds of load answer lookups differently when the mission information file
       ends without a line feed (claim 9, fourth item) and when it repeats a key (claim 9, second
       item), and those two variants use exactly those differences,
     - the release ships no mission-named file: `unmeasured` (a property of the stock files,
       checked by listing every archive and the game directory, as in claim 8),
     - this is the only layered load in the original: `unmeasured` (a claim that the original reads
       a file into a non-empty document in no other situation cannot be observed in a run; it was
       checked against the original's program).
   - Confidence: high for the rule and for the trigger. Medium that this is the only layered load in
     the original: other ways of reading a file were not all checked (Open questions).

7. **Overrides between documents are per field.** Where the original applies one document's rules on
   top of another's (for example a map's override sections on top of the base rules), it does not
   combine the documents. Each field is read from the overriding document with the field's current
   value as the fallback. A key that is absent from the overriding document, or has an empty value
   there, keeps its earlier value. Duplicate resolution (claims 1 and 2) applies within each document
   separately. The order of the layers and the exceptions to this rule are specified in S-0006 (rules
   layering).
   - Backing: `measurable` (scenario D6)
   - Confidence: medium

8. **Duplicates in stock data: two maps only.** Every file of the stock data that the original
   reads as an INI-format document was checked: the base rules file (stock data has no language
   rules file, S-0006), the art, AI, sound, speech, mission information and game-mode files, and every map
   the original can load,
   campaign and multiplayer, in the archives and beside them.
   - No rules, art, AI, sound, speech, mission information or game-mode file has a key twice in a
     section or two non-empty sections with the same name. One section name appears twice in the
     art file, but the first copy is empty, so it is not part of the table of section names
     (claim 2) and changes nothing.
   - Two maps do contain duplicates. The map of the third Soviet mission of the campaign holds one
     override section twice, and the two copies set different keys. One multiplayer map holds two
     keys twice each in one override section; one of them has the same value both times, the other
     two different values.
   - In both maps the hash-order rule picks the later occurrence: the second copy of the section,
     and the second occurrence of the key with two values. So claims 1 and 2 do change stock
     behaviour there: the keys that only the first copy of the section sets, and the first value of
     that key, are never seen. Because the later occurrence wins in both, turning
     `quirks.ini_duplicate_hash_order` off gives the same stock results.
   - No name table of any of these files, maps included, holds two different names with the same
     name hash, and no type list repeats a key. So claims 3 and 4 never change stock behaviour.
     They matter for user maps and mods.
   - Backing: `unmeasured` (a property of the stock files themselves, checked directly by reading
     every one of them and applying the hash-order rule to the duplicates found, which a game run
     cannot add to)
   - Confidence: high

9. **Layered loads: kept entries, repeated keys, empty values and the final line.** In a layered
   load (claim 6):
   - each entry of the second file replaces the existing entry with the same section and key name,
     or is added if there is none; entries of the first file that the second file does not mention
     stay as they were,
   - within the second file, a key written twice ends with its last value, whatever the hash-order
     rule would pick for a fresh load of that file,
   - an empty value (S-0001 claim 11) does not clear or replace an existing entry,
   - unlike a fresh load, the second file's final line is read even without a trailing line feed
     (with S-0001's `quirks.ini_final_line_needs_newline` off, a fresh load reads it too, and the two
     kinds of load agree).
   - Backing: `measurable` (scenario D5 for a replaced entry, D7 for the rest: one variant per item)
   - Confidence: high

10. **Layered loads: entry order, empty headers and existing duplicates.** In a layered load
    (claim 6):
    - a replaced entry moves to the *end* of its section's entry order, which changes its position
      in an enumeration (and so in a type list),
    - an empty section header in the second file creates nothing,
    - if the document already holds a key more than once, an entry of the second file with that
      key replaces exactly one of those occurrences: the one that a lookup of the key returns at
      that moment (claim 11). The other occurrences stay.
    - Backing: first item `unmeasured`; second item `measurable` (scenario D10 variant (d)), and
      third item `measurable` (scenario D8). The first item cannot show in the original: the only
      layered document is the mission information document, which is only ever looked up by name
      and never enumerated, so an entry's position is invisible. The second item follows from
      S-0001 claim 12: an empty section does not exist, so it never joins a table of names, and
      the second file is read under the same lexical rules as a fresh load. It is not invisible
      to lookups, because a section that was created would count towards the table's peak count
      (State and ordering), and D10 variant (d) tests exactly that. For the third item, D8 checks which
      occurrence is replaced and which one wins afterwards.
    - Confidence: high

11. **Layered loads: which occurrence wins afterwards.** During and after a layered load, lookups
    follow the name-table model in State and ordering (*Name tables during and after a layered
    load*). The result depends only on the two files:
    - Each line of the second file that has a non-empty key and value counts as two lookups: one of
      its section name in the table of section names, and one of its key in the key table of the
      section that this section lookup finds. A section that is not found is added, and the key
      lookup is made in the added section. In each table the lookups follow the order of the lines; how the lookups of the two tables interleave affects no result. An entry that is found is
      replaced (claims 9 and 10). Section headers, and lines that S-0001 drops, make no lookup.
    - After a replacement in a table that holds a repeated name hash, the winner can be the new
      entry or an older occurrence with the same name hash. Which one depends on the hashes of all
      names in the table, on the working order that earlier lookups left, and on the later lines
      of the second file in that section. As in a fresh load, a key that has nothing to do with the
      repeated one can change the outcome. Scenario D8 shows all three outcomes for one pair of
      repeated keys.
    - The table of section names behaves the same way. A line whose section is already in the
      document writes into the copy that its lookup returns, and a layered load never adds a second
      copy of a section that it finds. Which copy later lookups return can still change, because
      sections that the load adds change the working order (scenario D9).
    - Names are identified by hash (claim 3). A line whose key shares its name hash with an existing
      entry of a different name replaces that entry, and from then on the section holds the key
      under the second file's spelling. A line whose section name shares its name hash with an
      existing section of a different name writes into that section, which keeps its first
      spelling.
    - After the load, a lookup's result can depend on the lookups made before it, because a table
      may still remember an occurrence that re-sorting would no longer return (scenario D10). From
      the layered document the original reads only the current mission's section, and it takes
      every key it reads there (row D-g) from the one copy that a single lookup of the section's
      name returns. The values those keys give do not depend on the order in which they are read.
      Our engine gives the same values.
    - Backing: `measurable` (scenarios D8, D9 and D10)
    - Confidence: high

12. **Layered loads: the state at the start (departure).** Between the two loads, the original also
    applies the earlier file as a rules layer (S-0006, Edge cases). Those reads can leave a table of
    the earlier file remembering an occurrence. Our engine instead lets every table start the
    layered load with no remembered occurrence. Otherwise its tables are in the state that the fresh
    load and any lookups since have left. Nothing is added between the two loads, so a table that
    was ordered by those reads has the same working order as one ordered at its first lookup during
    the layered load.
    - The results differ from the original only when that rules reading found a name in a table that
      holds a repeated name hash, and the second file later changes that same table. In those cases
      the original can also go on using an entry that a replacement has discarded, so a later lookup
      can read discarded data or crash the game. From our engine's start state that cannot happen.
    - The importer warns (Edge cases). There is no quirk switch, because the original's result in
      these cases depends on the order of its own internal reads and is not a stable behaviour to
      reproduce.
    - Backing: `unmeasured` (a design choice of our engine. The cases it changes have no stable
      outcome in the original to measure. Outside them our engine agrees with the original, which
      D8 to D10 test, since their test files hold no rules sections.)
    - Confidence: high

13. **A missed lookup forgets an earlier winner only when it orders the table.** With the
    default quirk settings, a lookup in a non-empty, not-ordered name table that finds nothing
    leaves the table ordered and explicitly discards any previously remembered occurrence.
    It does not merely leave the earlier occurrence remembered without remembering a new one.
    A later lookup of that earlier name uses the bisection, so a duplicate can now resolve to a
    different occurrence. The same rule applies to keys and section names.
    - A miss in an already ordered table keeps its remembered occurrence, if any.
    - A lookup whose name hash equals the remembered occurrence's hash is a hit, even if its
      spelling differs. It returns that occurrence without ordering the table or forgetting it.
    - A lookup in an empty table changes nothing. A successful lookup that orders a table
      remembers the occurrence it finds instead of the previous one.
    - These are clarifications of the existing state table, not an additional quirk or a
      departure. The non-default name-comparison rule remains the one in Edge cases and quirks.
    - Backing: `measurable` (D11 for forgetting after a miss; D10 for the remembered-hit
      shortcut and forgetting peaks; D8 for successful lookups after ordering). The ordered-miss and empty-table distinctions are
      `unmeasured`: keeping or dropping a remembered occurrence in an unchanged ordered table
      gives the same winner, and an empty table has no occurrence to return
    - Confidence: high

## Formulas

### Name hash

The name hash *H* of a name *n* of *len* bytes is the standard 32-bit CRC, computed over a padded
copy *p* of *n*. The CRC's parameters: generator polynomial
*x*³² + *x*²⁶ + *x*²³ + *x*²² + *x*¹⁶ + *x*¹² + *x*¹¹ + *x*¹⁰ + *x*⁸ + *x*⁷ + *x*⁵ + *x*⁴ + *x*² + *x* + 1,
input bytes and output reflected (least significant bit first), register initialised to all ones,
final value complemented. Its value for the nine ASCII bytes "123456789" is 3421780262 (unsigned).
The padded copy is:

- if *len* is a multiple of 4 (including 0), *p* = *n*,
- otherwise, with *r* = *len* mod 4, *p* is *n* followed by one byte of value *r*, then 3 − *r* copies
  of the byte *n*[*len* − *r*] (the first byte of the last, incomplete group of four).

Examples: a 7-byte name gets one extra byte of value 3, and the name "A" is hashed as the bytes "A",
1, "A", "A". The empty name has hash 0.

For ordering, *H* is read as a signed 32-bit two's-complement integer *h* in the range −2³¹ to
2³¹ − 1, and names are compared by *h* only. All quantities in these formulas are integers; no
floating point is involved.

### Hash-order rule (which occurrence wins)

For one name table, let *A* = (*a*[0], …, *a*[*N* − 1]), *N* = |*A*|, be its occurrences in file
order (keys of one section in entry order, or the document's non-empty sections in file order), and
write *h*(*a*) for the signed hash of an occurrence's name. The winning occurrence of a lookup for a
name with hash *t* is

> *W*(*t*) = *F*(0, *N*), evaluated over *x* = Sort(*A*, 0, *N* − 1),

where *F* is the bisection and Sort the reference sort defined below. Both look at hashes only.
Occurrences with equal *h* end in whatever relative order the reference sort's definition gives
them; it is not stable.

For a document built by one fresh load, the input to the sort is file order, and the winner does
not depend on when lookups happen or in what order (State and ordering). Lookups behave as if each
table were put in reference-sort order at its first lookup and kept in that order from then on. A
table that is not changed afterwards gives the same winners either way. A layered load changes
tables after lookups may have begun. The winner is then decided by the name-table model in State
and ordering, which applies the same sort and bisection to the table's working order instead of
to file order (claim 11).

**Bisection.** For a window of *m* consecutive positions of *x* starting at position *b*, let
*q* = *b* + ⌊*m*/2⌋ and define

> *F*(*b*, 0) = not found
>
> *F*(*b*, *m*) = *x*[*q*] if *h*(*x*[*q*]) = *t*
>
> *F*(*b*, *m*) = *F*(*q* + 1, *m* − ⌊*m*/2⌋ − 1) if *h*(*x*[*q*]) < *t*
>
> *F*(*b*, *m*) = *F*(*b*, ⌊*m*/2⌋) if *h*(*x*[*q*]) > *t*

(for *m* ≥ 1). The winner is the occurrence at the first probed position *q* whose hash equals *t*.

**Reference sort.** Sort(*X*, *lo*, *hi*) maps a sequence *X* to a permutation of it that differs
from *X* only at positions *lo* to *hi*. Let *s* = *hi* − *lo* + 1. Elements are compared by *h*
only.

- *s* ≤ 1: Sort(*X*, *lo*, *hi*) = *X*.
- 2 ≤ *s* ≤ 8 (selection from the top): let *j* be the lowest position in [*lo*, *hi*] whose
  element has the largest *h* in that range, and let *X*′ be *X* with positions *j* and *hi*
  exchanged (unchanged if *j* = *hi*). Then Sort(*X*, *lo*, *hi*) = Sort(*X*′, *lo*, *hi* − 1).
- *s* > 8 (partition):
  - Let *X*⁰ be *X* with positions *lo* + ⌊*s*/2⌋ and *lo* exchanged, and let the pivot hash be
    *v* = *h*(*X*⁰[*lo*]).
  - Define cursor pairs (*u*ₖ, *d*ₖ), starting from *u*₀ = *lo* and *d*₀ = *hi* + 1. For *k* ≥ 0:

    - *u*ₖ₊₁ is the smallest position *i* with *u*ₖ < *i* ≤ *hi* and *h*(*X*ᵏ[*i*]) > *v*, or
      *hi* + 1 if there is none,
    - *d*ₖ₊₁ is the largest position *i* with *lo* < *i* < *d*ₖ and *h*(*X*ᵏ[*i*]) < *v*, or *lo*
      if there is none.

    If *d*ₖ₊₁ ≥ *u*ₖ₊₁, then *X*ᵏ⁺¹ is *X*ᵏ with positions *u*ₖ₊₁ and *d*ₖ₊₁ exchanged, and the
    next pair is formed from *X*ᵏ⁺¹.
  - Let (*u*, *d*) = (*u*ₖ₊₁, *d*ₖ₊₁) be the first cursor pair with *d* < *u*, and let *Z* be
    that *X*ᵏ with positions *lo* and *d* exchanged.
  - Then Sort(*X*, *lo*, *hi*) = Sort(Sort(*Z*, *lo*, *d* − 1), *u*, *hi*). Each inner range is
    classified by its own size *s*. Positions *d* to *u* − 1 lie between the two ranges, hold
    elements whose hash equals *v*, and are not moved again. The two ranges are disjoint, so the
    order in which they are sorted does not matter.

Worked example (fixture 1, row D-c): a section with two occurrences *K*₁, *K*₂ of the same key, in
that file order. Both have the same *h*. In the selection step *K*₁ counts as the largest, being the
lowest position among equals, so it is exchanged with position 1, giving (*K*₂, *K*₁). The
bisection probes position ⌊2/2⌋ = 1, which holds *K*₁: **the first occurrence wins**.

Fixture 2 (row D-d) adds a third key *G* after them, with a smaller *h* than *K*. The file order is
(*K*₁, *K*₂, *G*). The first selection exchanges *K*₁ with position 2, giving (*G*, *K*₂, *K*₁).
The second selection, over positions 0 and 1, finds *K*₂ already at position 1. The bisection probes
position ⌊3/2⌋ = 1: **the second occurrence wins**.

These two documents falsify both "first wins" and "last wins".

An implementation does not need a hash-sorted table. It only has to return the same winner. For
example, it can keep every occurrence in a map from name hash to occurrence list, and apply the
reference sort and the bisection only to tables in which some hash occurs more than once. Tables
without repeated hashes give the obvious answer whatever method is used.

With `quirks.ini_duplicate_hash_order` off, a duplicated name resolves to its last occurrence in the
table's order: for keys, the last in the section's entry order; for sections, the last copy in file
order. After a layered load this is still well defined, because the layered load places every entry
it adds or replaces at the end of the entry order (State and ordering) and never adds a copy of a
section. So a key that the second file gives resolves to the second file's last value for it, as
claim 9 says. How the two quirk switches combine is in Edge cases and
quirks.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Name hash width | 32 | bits | Compared as signed |
| Selection-step limit | 8 | elements | Ranges of at most this size use the selection step |
| Hash group size | 4 | bytes | Names are padded to a multiple of this before hashing |
| Forgetting peaks | 0, 10, 20, 40, 80, … | occurrences | 0, then 10, then each value twice the one before. When an occurrence is added to a table that holds exactly its peak count, and that count is one of these values, the table forgets its remembered occurrence (State and ordering). They are counts of occurrences only |

## State and ordering

Which occurrence wins, stated as an observable consequence: in a document built by a fresh load, the
winner depends only on the complete
set of names in the table and their file order, never on the order of later lookups. Every lookup
of a name returns the same occurrence for the lifetime of the document, unless a layered load
changes the table (claims 6, 9, 10 and 11). After that, a result can also depend on the lookups made
before it (claim 11).

Layered load, per line of the second file (claims 9 and 10):

| Existing entry with same section and key | New line | Result |
|---|---|---|
| none | non-empty key and value | Entry added at the end of the section's entry order (section created if needed) |
| present | non-empty key and value | Existing entry removed; new entry added at the end of the section's entry order |
| any | empty key or value | No change |
| (section header with no entries following) | | No section created |

The table above fixes the entry order that enumeration uses. Which occurrence a lookup returns is
fixed by the model below. When the existing document already holds duplicates of the replaced key,
only the occurrence that the line's lookup returned is replaced, and the other occurrences stay.

### Name tables during and after a layered load

Each name table behaves as if it kept, besides its occurrences, a working order of them, whether it
is ordered, at most one remembered occurrence, and a peak count (Terms). A fresh load leaves every
table with file order as its working order, not ordered, with nothing remembered, and with its
number of occurrences as its peak count. A lookup in a table
with no occurrences finds nothing and changes nothing. Otherwise, for one name table, with *t* the
name hash of the name looked up (Formulas):

| State of the table | Event | Result |
|---|---|---|
| Its remembered occurrence has hash *t* (ordered or not) | Lookup | Returns the remembered occurrence. Nothing changes |
| Ordered, and no remembered occurrence with hash *t* | Lookup, where the bisection over the working order *x* finds an occurrence | Returns that occurrence, which becomes the remembered one |
| Ordered, and no remembered occurrence with hash *t* | Lookup, where the bisection over *x* finds nothing | Finds nothing. Nothing changes: a remembered occurrence, if any, stays |
| Not ordered, and no remembered occurrence with hash *t* | Lookup, where the bisection over Sort(*x*, 0, *N* − 1) finds an occurrence | The working order becomes Sort(*x*, 0, *N* − 1) and the table is ordered. Returns that occurrence, which becomes the remembered one |
| Not ordered, and no remembered occurrence with hash *t* | Lookup, where the bisection over Sort(*x*, 0, *N* − 1) finds nothing | The working order becomes Sort(*x*, 0, *N* − 1) and the table is ordered. Finds nothing and forgets any previously remembered occurrence (claim 13) |
| Holds exactly its peak count, and that count is a forgetting peak (Constants) | An occurrence is added (by a fresh load, or as a layered load's new entry or new section) | The table forgets its remembered occurrence, and its new number of occurrences becomes its peak count. The new occurrence goes at the end of the working order, and the table is not ordered |
| Any other state | An occurrence is added (as above) | The new occurrence goes at the end of the working order, and the table is not ordered. If the table now holds more occurrences than its peak count, that number becomes its peak count. The remembered occurrence, if any, stays |
| Any | A layered load replaces an occurrence | The occurrence that the line's lookup returned leaves the working order. The others keep their relative order. The table stays ordered if it was, and forgets its remembered occurrence. The new entry is then added as in the two rows above |

The lookups that count are the two that each line of a layered load makes (claim 11), the reads made
from the document afterwards, and the reads of the earlier file as a rules layer between the two
loads. The last can only put tables in order, since claim 12 has every table start the layered load
with nothing remembered.
A remembered occurrence can outlast an addition. A later lookup of the same name hash then returns
it without re-sorting, even though re-sorting might now return a different occurrence. With only
fresh loads this cannot change any result, because a fresh load ends before the first lookup.

Two consequences, stated as results:

- In a table without a repeated name hash, every lookup returns the one occurrence of its name,
  whatever the model's state.
- From the start state of claim 12, the replaced occurrence is always the one the line's lookup
  returned, and no lookup ever returns an entry that a replacement discarded.

## Edge cases and quirks

- `quirks.ini_duplicate_hash_order` (YR default: on). The winner is chosen by the hash-order rule.
  Off: the last occurrence in the table's order wins (Formulas). The switch is on by default for
  every input: stock data, user maps and mods alike.
- `quirks.ini_hash_identity` (YR default: on). Names are equal when their hashes are equal. Off: names
  are equal only when their bytes are equal. On by default for every input, like the switch above.
- Importer warnings. Whatever the two switches are set to, the importer warns about the inputs
  below (see *importer warning* in the glossary). A warning never changes the imported result.
  - every key that occurs more than once in one section, in any file, a layered load's second file
    included,
  - every section name that more than one non-empty section of one file uses (an empty copy is not
    part of the table of section names, claim 2, so it can never change a result and does not warn),
  - every pair of different names in one name table that share a name hash (claim 3),
  - every line of a layered load's second file that replaces an entry of a different name, or writes
    into a section of a different name, because the names share a name hash (claim 11),
  - every layered load whose earlier file holds a repeated name hash in a table that the rules
    reading of that file looks up (its table of section names, or the key table of a section that
    reading reads) and that the second file then changes. Our engine's result may differ from the
    original's there (claim 12).

  The warnings exist because the winner by the hash-order rule is hard to predict, and data written
  for other programs often assumes that the first or the last occurrence wins.
- The two switches combine as follows, for a lookup of name *n* in one name table. "Same-hash
  occurrences" are the occurrences whose name hash equals *H*(*n*); "same-name occurrences" are
  those whose bytes equal *n*.

  | hash order | hash identity | Occurrences that count as *n* | Winner |
  |---|---|---|---|
  | on | on (YR) | all same-hash occurrences | the hash-order rule over the whole table |
  | on | off | same-name occurrences only | the hash-order rule over the whole table, and in a layered load the name-table model (State and ordering), with every comparison of hashes made on the pair (*h*, name) instead of *h* alone |
  | off | on | all same-hash occurrences | the last of them in the table's order (Formulas) |
  | off | off | same-name occurrences only | the last of them in the table's order (Formulas) |

  In the second row, pairs are ordered by *h* first. Pairs with equal *h* are ordered by their names
  as byte strings, byte by byte as unsigned numbers, with a name that is the beginning of a longer
  one coming first. Two occurrences tie only when they have the same name. The remembered occurrence
  is returned only to a lookup of its own name, and a layered-load line replaces only an entry of
  its own name, or writes only into a section of its own name. In a table in which no two different
  names share a hash, every comparison comes out as in YR, so every lookup gives the same winner as
  in YR; only lookups of colliding names change. This row is our engine's design. Up to revision 4
  of this spec it removed the other names' occurrences before sorting instead; the pair comparison
  replaces that because it extends to layered loads. A lookup that finds no counting occurrence is
  "not found" in every row.
- Enumeration count: with duplicates, the number of keys an enumeration reports is the number of
  occurrences, so a type list with a duplicated key reports more positions than distinct keys.

## Import mapping

| Row | Data | Note |
|---|---|---|
| D-a | `[InfantryTypes]`, `[VehicleTypes]`, `[AircraftTypes]`, `[BuildingTypes]` and the other type list sections of the rules file, such as `[Warheads]`, `[Animations]`, `[Countries]` | Type lists: claims 4 and 5 apply. Values are read as text of at most 31 bytes (S-0003) |
| D-b | A map's override section for an infantry type, key `Cost` | Observable used by the scenarios: the cost shown on the sidebar (integer form, S-0003) |
| D-c | Fixture 1: an override section whose only lines are `Cost=` value A, then `Cost=` value B | Hash-order rule predicts A |
| D-d | Fixture 2: an override section whose only lines are `Cost=` value A, `Cost=` value B, then `Filler0=1` | Hash-order rule predicts B, because `Filler0` has a smaller *h* than `Cost` (worked example in Formulas): *h*(`Cost`) = −1206633534, *h*(`Filler0`) = −1429695458. `Filler0` is not a key the original reads |
| D-e | Fixture 3: an override section whose only lines are `ZdxeLZgd=` value B, then `Cost=` value A | `ZdxeLZgd` has the same name hash as `Cost` (both −1206633534). The hash-order rule predicts B for the cost |
| D-f | A type section's `UIName` key | String-table label of the type's display name, shown in the sidebar tooltip together with the cost. Used by D2 as the extra key |
| D-g | The mission information file: one section per campaign mission, named after the mission's map file name, with keys `Briefing` (string-table label of the mission briefing) and `UIName` (string-table label of the mission's display name, from which the default description of a saved game is also derived) | Read for the current mission only, by name. These are the observables of D5 and D7 |
| D-h | Test file for D5 and D7: a loose file named like the mission's map file but with the extension `.INI` | The earlier file of the layered load; the mission information file is the later file (claim 6). The release ships no such file |
| D-i | A type section's `Strength` and `Speed` keys | Used by D6: the strength key gets an empty value in run 2; neither key is otherwise written by the test map |
| D-j | Loose copy for D7: a loose file with the same name as the shipped mission information file, holding the shipped file's contents with one edit to the first Allied mission's section (row D-g) | Found before the shipped file (S-0005), so it is the mission information file of the run. Only the edited section differs from stock |
| D-k | D8 variants: the section of D8's test file with a third line, `Filler0=1` (variant b) or `Filler9=1` (variant c) | *h*(`Briefing`) = 2057803766. *h*(`Filler0`) = −1429695458 is smaller and *h*(`Filler9`) = 2064265047 is larger. Neither filler key is read by the original |
| D-l | Test file for D9 and D10: a loose file named as in row D-h, holding two sections named like the first Allied mission's section, the first with the single line `Briefing=` X, the second with the single line `Briefing=` Y. Variant (b) of D9 adds a section `[Filler8]` with the single line `Filler=1` after the two, variant (c) adds it before them | *h*(`Filler8`) = 1645288982. No section of the file is one the rules reading looks up, so D9 and D10 test the original's rule, not claim 12's departure |
| D-m | Loose copy for D10: the shipped mission information file, with the first Allied mission's section moved unchanged to the end, followed by a new last section `[Filler999]` with the single line `Filler=1`. Variant (d) of D10 adds an empty section header `[Filler998]`, with no line under it, immediately before the moved section | Found before the shipped file (S-0005), as row D-j. *h*(`Filler999`) = 1330041197 |
| D-n | Filler sections for D10: `[Filler0]`, `[Filler1]`, … in that order, each with the single line `Filler=1`, placed in row D-l's test file before its two sections. Variant (b) uses as many as make the table of section names hold 39 sections just before the loose copy's last section is added; variants (c) and (d) use one more, so that the table holds 40 at that point | The number of filler sections follows from the number of sections of the mission information file. No section is ever removed, so the table's peak count is its number of sections. In (c) the table holds 40, a forgetting peak (Constants), so the addition makes the table forget its remembered occurrence. In (b) it holds 39, which is not one |
| D-o | D11 loose mission information file: only the first Allied mission's section with `Briefing=` Z, followed in variant (b) by `[Filler0]` with `Filler=1`, and in variant (c) by that section and then `[Filler1]` with `Filler=1` | All values are text, with a line feed after every line. X and Y in the earlier file are D8's briefing labels; Z is the first mission's own briefing label. Missing briefing uses no label. The filler sections are initially absent, create no game setting, and have signed name hashes −1429695458 and −1277975201. This is named diagnostic data, not a new campaign import requirement |

## Measurement scenarios

Common setup: the measurement setup (an unmodified install with no loose override files and no
engine add-ons), plus the test data each scenario names, which is removed afterwards. D7's loose
copy of the mission information file (row D-j) is such named test data: a loose file that overrides
shipped data on purpose, which the measurement setup allows for the scenario that names it. D1 to D4 and
D6 use a tiny purpose-made multiplayer map in skirmish against one computer opponent, any game
speed. The test type is one buildable infantry type of the player's side. Values A and B are two
different costs, both different from the type's base-rules cost. Every scenario is deterministic:
one run per variant is enough, and a second run only guards against operator error.

- **D1 (duplicate keys; claim 1).**
  - Setup: the test map, once with fixture 1 (row D-c) and once with fixture 2 (row D-d) as the test
    type's override section.
  - Observable: the cost shown on the sidebar for the test type, in each run.
  - Expected: A with fixture 1, B with fixture 2.
  - Falsified by: any other pair of results. In particular, the same occurrence winning in both
    runs would mean first-wins (A, A) or last-wins (B, B).
- **D2 (duplicate sections; claim 2).**
  - Setup: the test map holds the test type's override section twice. The first copy has cost A
    only. The second has cost B and the display-name key (row D-f) set to the label that another
    infantry type of the same side uses for its display name, copied from that type's own section.
  - Procedure: before running, compute the predicted winning copy with the hash-order rule from the
    map's complete list of non-empty section names, in file order.
  - Observable: the sidebar tooltip of the test type: its cost and its display name.
  - Expected: if the first copy is predicted, cost A and the test type's usual name; if the second,
    cost B and the other type's name.
  - Falsified by: cost A with the other type's name, or cost B with the usual name (the copies were
    mixed), or the result of the copy that was not predicted.
- **D3 (type lists; claims 4 and 5).**
  - Setup: the test map's map layer carries its own infantry type list (row D-a), which the original
    reads on top of the base rules' list. The map also defines two new infantry types, each a copy of
    a stock infantry type's fields under a new name, buildable by the player. Variant (a): one key,
    not used by the base rules' list, written twice, first naming the first new type, then the
    second. Variant (b): the first new type named under two different unused keys.
  - Procedure: in variant (a), predict the winner with the hash-order rule from the keys of that section.
  - Observable: the sidebar icons of the new types.
  - Expected: (a) only the predicted new type has an icon; (b) exactly one icon for the new type.
  - Falsified by: (a) both new types having icons, or only the other one; (b) two icons for one type.
- **D4 (hash collision; claim 3).**
  - Setup: the test map with fixture 3 (row D-e) as the test type's override section.
  - Observable: the cost shown on the sidebar.
  - Expected: B (the colliding key's value), although the cost key itself says A.
  - Falsified by: cost A.
- **D5 (layered load; claim 6).**
  - Setup: the test file of row D-h for the first mission of the Allied campaign, placed in the game
    directory. It holds no rules sections, only one section with the same name as that mission's
    section in the mission information file (row D-g). That section has two entries: the briefing
    key and the display-name key, each set to the label that the mission information file gives the
    *second* Allied mission for the same key. A control run without the test file. Any difficulty,
    any game speed.
  - Procedure: start the first Allied mission from the campaign menu. Once it is in play, open the
    mission briefing from the in-game menu, then open the save-game dialog. Remove the test file
    afterwards.
  - Observable: the briefing text shown, and the description the save-game dialog proposes, each
    compared with the control run.
  - Expected: both identical to the control run (the first mission's own briefing and name), because
    the mission information file is the later file and its values win.
  - Falsified by: the second mission's briefing or name in either place. That would mean the earlier
    file wins, or that the two sections compete by the hash-order rule as they would after a fresh
    load of both files into one document. The mission is chosen so that this test can tell the two
    apart: for the first Allied mission, the hash-order rule over the section names of the test file
    followed by those of the mission information file picks the test file's copy. (For the first
    Soviet mission it would pick the stock copy, so that mission could not separate them.)
  - A pass alone also looks like a run in which the test file was not read at all, because nothing
    else read from that document shows whether the file was found. D7 variant (a) uses a test file
    of the same name and shows that it is read, so the two scenarios together confirm claim 6.
- **D6 (per-field override; claim 7).**
  - Setup: the test map with an override section for the test type that sets the cost only (run 1),
    then the same section with an extra line giving the type's strength key (row D-i) an empty value
    (run 2). A control run without any override section.
  - Observable: the sidebar cost; the time the type needs to walk a fixed straight path of 10 cells
    on clear terrain, in ticks; and the number of identical hits from one fixed attacker needed to
    kill it.
  - Expected: run 1: the cost changes to the override's value, and walking time and hits to kill
    equal the control run. Run 2: as run 1.
  - Falsified by: the cost not changing, or walking time or hits to kill differing from the control
    run, in either run.
- **D7 (layered load in detail; claims 6 and 9).**
  - Setup: D5's test file (row D-h), plus a loose copy of the mission information file (row D-j).
    Variant (a) uses a test file of the same name whose section holds only the briefing line of
    D5's test file. Each variant edits only the first Allied mission's section of the copy, as
    follows:
    - (a) kept and added entries: the briefing line is removed,
    - (b) last value: the briefing line is written twice in a row, in place of the original line,
      first with the label that the file gives the second Allied mission for the briefing, then with
      the section's own briefing label,
    - (c) empty value: the briefing key is kept, but its value is left empty,
    - (d) final line: the section is moved to the end of the copy and its briefing line to the end
      of the section, unchanged, so that it is the last bytes of the file, with no line feed (and no
      CR byte) after it.
    - (e) and (f) final line and last value, with a header-only test file: these two use a test
      file (row D-h) that holds only a section header named like the first Allied mission's
      section, with no line under it, so that it has no section with an entry and the load of the
      mission information file is a fresh one (claim 6). Variant (e) uses the loose copy of variant
      (d), variant (f) the loose copy of variant (b).
    - Any difficulty, any game speed.
  - Procedure: for each variant, one run with the test file and one control run without it, so that
    the copy is read by a fresh load. In each run, start the first Allied mission from the campaign
    menu; once it is in play, open the mission briefing from the in-game menu, then open the
    save-game dialog, as in D5. In variant (b), first confirm with the hash-order rule, over the
    edited section's keys in file order, that a fresh load picks the first of the two briefing
    lines. It does for this section, which depends only on its key names.
  - Observable: the briefing text shown and, in variant (a), the description the save-game dialog
    proposes.
  - Expected, with the test file:
    - (a) the second Allied mission's briefing (an entry kept from the earlier file), and the first
      mission's own name in the save description (an entry the later file added),
    - (b) the first mission's own briefing,
    - (c) the second Allied mission's briefing,
    - (d) the first mission's own briefing,
    - (e) no briefing text: a fresh load drops the final line (S-0001 claim 5, with
      `quirks.ini_final_line_needs_newline` on), as in the control run of (d),
    - (f) the second Allied mission's briefing: a fresh load lets the hash-order rule decide, as
      in the control run of (b).
  - Expected, control runs:
    - (a) and (c) no briefing text (an empty briefing, or none offered),
    - (b) and (f) the second Allied mission's briefing (claim 1),
    - (d) and (e) no briefing text (S-0001 claim 5).
  - Falsified by, with the test file:
    - (a) no briefing text (the test file was not read, or its entries were dropped), or an empty
      or other save description (the later file's entry was not added),
    - (b) the second Allied mission's briefing (the first value won, or the hash-order rule
      decided),
    - (c) no briefing text (the empty value cleared the entry),
    - (d) the second Allied mission's briefing (the final line was ignored),
    - (e) the first mission's own briefing (the load was layered although the test file has no
      section with an entry, so the final line was read),
    - (f) the first mission's own briefing (the load was layered, so the last value won).
    Variants (e) and (f) rely on variant (a) having shown that a test file of that name is read,
    so that a header-only file is known to be found by name.
  - Every run is deterministic, so one run per variant and mode is enough.
- **D8 (layered load onto an existing duplicate; claim 10, third item, and claim 11).**
  - Setup: a test file named as in row D-h for the first Allied mission, with no rules sections and
    one section named like that mission's section in the mission information file. In variant (a)
    the section holds exactly two lines, both with the briefing key: first the label that the
    mission information file gives the *second* Allied mission for the briefing (value X), then the
    label it gives the *third* Allied mission (value Y). Variants (b) and (c) add a third line with
    an unrelated key (row D-k). The mission information file is the shipped one, not a loose copy.
    Any difficulty, any game speed.
  - Procedure: before running, confirm with the hash-order rule which briefing line a fresh load of
    the test file picks: X in (a) (the pattern of fixture 1), Y in (b) (the pattern of fixture 2,
    since the third key has the smaller hash), X in (c) (the third key has the larger hash and is
    sorted out of the way first). Start the first Allied mission from the campaign menu; once it is
    in play, open the mission briefing from the in-game menu. Remove the test file afterwards.
  - Observable: the briefing text shown.
  - Expected, from the name-table model (claim 11):
    - (a) the third Allied mission's briefing (Y): the load replaced X, and Y wins afterwards,
    - (b) the second Allied mission's briefing (X): the load replaced Y, and X wins afterwards,
    - (c) the first mission's own briefing: the load replaced X, and the new entry wins afterwards.
  - Falsified by: any other result in any variant. In particular, the briefing that a fresh load of
    the test file picks (X in (a) and (c), Y in (b)) means that the replaced occurrence is still
    seen, and the same result in all three variants means that the unrelated key does not matter.
  - Deterministic: one run per variant is enough.
- **D9 (layered load onto two copies of a section; claim 11).**
  - Setup: the test file of row D-l in variants (a), (b) and (c), with X and Y as in D8, and the
    shipped mission information file. Any difficulty, any game speed.
  - Procedure: as in D8. Remove the test file afterwards.
  - Observable: the briefing text shown.
  - Expected:
    - (a) the first mission's own briefing: the copy that the load wrote into still wins after it,
    - (b) the second Allied mission's briefing (X): the first copy wins after the load, though the
      load wrote into the second,
    - (c) the third Allied mission's briefing (Y): the second copy wins after the load, though the
      load wrote into the first.
  - Falsified by: any other result. The same result in all three variants would mean that the
    sections the load adds do not change which copy wins.
  - Deterministic: one run per variant is enough.
- **D10 (remembered occurrence and forgetting peaks; claims 10 and 11 and State and ordering).**
  - Setup: the loose copy of row D-m, plus the test file of row D-l without the filler section of
    D9: in variant (a) as it is, in variants (b), (c) and (d) with the filler sections of row D-n,
    and in variant (d) with the empty header of row D-m. X and Y as in D8. Any difficulty, any game speed.
  - Procedure: as in D8. Remove the test file and the loose copy afterwards.
  - Observable: the briefing text shown.
  - Expected:
    - (a) and (b) the first mission's own briefing: the loose copy's last lines leave the copy that
      they wrote into remembered, the last section is then added, and the lookup after the load
      returns the remembered copy without re-sorting,
    - (c) the second Allied mission's briefing (X): the last section is added while the table holds
      a forgetting peak (row D-n), so the table forgets the remembered copy, and the lookup after the
      load re-sorts and returns the other copy,
    - (d) the second Allied mission's briefing (X), as in (c): the empty header creates nothing, so
      it is the last section's addition that is made at the forgetting peak.
  - Falsified by: in (a) or (b), the third Allied mission's briefing (Y), which is what re-sorting
    gives, so the table would not remember; in (c), the first mission's own briefing, so an addition
    at a forgetting peak would not make the table forget; in (d), the first mission's own briefing,
    which means that the empty header was added as a section (its addition was made at the
    forgetting peak before the loose copy's lines were remembered, and nothing later made the table
    forget).
  - Deterministic: one run per variant is enough.

- **D11 (a missed lookup in a not-ordered table; claim 13, Q-0183).**
  - Setup: the earlier file of row D-l with exactly the two mission sections and no filler
    sections, plus the loose mission information file of row D-o. Variant (a) has no filler
    section in the later file; (b) adds its first filler section only; (c) adds both. Use the
    measurement setup, no engine add-on and no mods (the named test files of a scenario excepted), any game speed. This retains
    D10's original-game diagnostic route only; campaign implementation remains deferred.
  - Procedure: start the first Allied mission, open its briefing, and capture the text as in
    D8. One run per variant suffices because no random choice affects the result. Remove both
    named test files afterwards. No bench change is required; manual input or scripted input
    and a screen capture suffice.
  - Observable: which of X, Y and Z is displayed as the briefing.
  - Expected: Z in (a) and (b), Y in (c). The load writes Z into the initially winning first
    copy. In (b) the first filler's addition leaves that copy remembered and the table not
    ordered. In (c) the second filler's absent-name lookup orders the table and forgets that
    copy; after its addition, the final mission lookup finds the second copy, which still
    holds Y. There are only four section occurrences, below the first positive forgetting
    peak, so growth does not explain the forgetting.
  - Falsified by: Z in (c) with Z in both controls, supporting retention across the missed
    lookup; any other result also contradicts the predicted name-table behaviour. Y in (b)
    would already contradict retention after addition, so (c) alone is not a sufficient test.

## Open questions

- Whether the original reads a data file into a document that already has sections in any
  situation other than the one of claim 6. Only the usual way the original reads a data file was
  checked, and it does so in that one situation. Several less common ways of reading a data file
  were not checked.
