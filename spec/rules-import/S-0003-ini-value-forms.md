---
id: S-0003
title: INI value forms
topic: rules-import
revision: 18
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-14
measurements: []
requests: [Q-0061, Q-0075, Q-0100, Q-0171, Q-0234]
supersedes: []
---

# S-0003: INI value forms

## Summary

How the original turns an entry's value text into each kind of field (text, booleans, integers,
decimals and percentages, integer tuples, colour and byte triples, lists, type names and named values),
including spellings, truncation, number domains, rounding, overflow and malformed values, what our
engine does where the original's result is undefined or not characterised here, and which inputs make
the importer warn. Reading a file into sections and entries is S-0001, duplicate resolution is
S-0002, and the floating-point rounding mode these value forms rely on (claim 17) belongs as a whole
to S-0205 (number domains). These claims cover skirmish imports; network games and campaigns
are out of scope.

**Joint hand-off checks.** Confirm claim 14's four particle-reference exclusions against
S-0004 claim 2 and its field-specific switches in the versions handed off together.
Confirm claim 9 against S-0009 claim 29 and its header-pair
Import mapping in the versions handed off together: unread components are unspecified
in the original; completion from the value in force is our engine's departure, with
an importer warning. Missing or empty pairs keep the whole value in force. Confirm claim 14's trigger-name exceptions against S-0011 claims 5 and 21
in the versions handed off together, including owner-country marker handling and
exact event type-name matching. S-0011's reveal scope and S-0409 claim 17 already
agree on every player house in our engine, including observers; that design rule
is not an extension of this spec's type-reference rule. Confirm claim 16's display-only
pip-colour correction against S-0111 claim 12 and S-0006 claim 5 in the versions
handed off together: absent reads keep the held pip value in our engine, with no
quirk switch.

## Terms

S-0001's terms (document, section, entry, key, value, whitespace byte, trim, trim defect) apply. In addition:

The glossary's *field*, *absent-key default*, *unspecified*, *departure*, *importer warning* and
*round-toward-zero mode* apply as defined there. In particular, wherever this spec calls a result of
the original unspecified, or the original crashes, the claim also states our engine's safe,
deterministic result (a departure, with no quirk switch) and the importer warns. Claim 6 makes one
further departure, of scope, for results that are deterministic but not characterised here. Every
statement that the importer warns is our importer's design: it is backed `unmeasured` (a design
choice, which never changes the imported result) with high confidence, whatever the backing of the
claim it sits in. The backing and confidence listed under a claim cover the original's behaviour and
our engine's result.

- **Value in force.** The value a field has when its key is read: its absent-key default, or, in a
  later rules layer, the value an earlier layer gave it (S-0002 claim 7). A key that is missing, or
  written with an empty value (which creates no entry, S-0001 claim 11), leaves the value in force
  unchanged, except for rectangles (claim 10).
- **Component fallback.** For a rectangle field (claim 10), the four numbers that fill the
  components a value does not give. It is fixed per field and is not the field's absent-key
  default, which is 0, 0, 0, 0 for every rectangle field. For the map size it is 1, 1, 50, 50
  (S-0009), horizontal numbers in cell widths and vertical numbers in cell heights (glossary *map
  size* and *local area*).
- **Stored value.** The value text as S-0001 keeps it in the entry, before any form is applied. It has
  no leading whitespace and normally no trailing whitespace (see S-0001's trim defect).
- **Percentage field.** A decimal field (claim 6) whose value is a fraction of a whole, so that its
  meaningful values lie between 0 and 1 (0 % to 100 %): a share, a chance or a health threshold.
  The original reads it exactly like any other decimal field (claim 7). The term exists only for our
  importer's warning (claim 18). The armour-modifier list (claim 13) is treated in the same way, item
  by item. Fields whose value is a multiplier or a quantity are not percentage fields, even when
  their values are written with `%`.
- **Text limit.** The maximum number of bytes a particular field reads, which differs per field.
- **Single, double.** IEEE 754 binary floating point with a 24-bit or 53-bit significand.
- **Toward zero.** Rounding that discards the excess, also called truncation.

## Behaviour

### Text

1. **Text values.** A field read as text gets the stored value cut to its first *N* bytes, where *N*
   is the field's text limit, with any trailing whitespace of the cut text removed. So a value whose
   cut ends in whitespace yields a shorter text. This removal does not have S-0001's trim defect. A
   missing key yields the value in force, cut and trimmed in the same way. Text limits are per
   field. The most common is 127 bytes, which is also the usual limit for lists. Type names in list
   sections are limited to 31 bytes (Import mapping). The importer warns when the cut discards a
   byte that is not whitespace, naming the field's text limit. The same warning applies to every
   field that has a text limit: lists (claim 12), integer tuples (claims 9 to 11) and named values
   (claim 16).
   - Backing: `measurable` (scenario V1)
   - Confidence: high (the mechanism); medium (the per-field limits, which are listed only where
     known)

### Booleans

2. **Booleans look at the first byte only.** The first byte of the stored value, case-insensitively:
   - `1`, `T` or `Y`: true,
   - `0`, `F` or `N`: false,
   - anything else: the field keeps the value in force.

   So `yes`, `Yes please`, `true` and `t` are true, and `no`, `False` and `Falsehood` are false. `on`,
   `off`, `2` and `-1` are not recognised and leave the value in force. The importer warns for a
   value whose first byte is none of the six.
   - Backing: `measurable` (scenario V2)
   - Confidence: high

### Integers

3. **Three integer forms, chosen by the value's first and last byte.** Results are signed 32-bit
   integers.
   - **`$` prefix (hexadecimal).** A value whose first byte is `$` is read as hexadecimal after the
     `$`: optional whitespace, an optional sign, an optional `0x` or `0X`, then hexadecimal digits in
     either case, up to the first non-digit. `$1F` and `$1f` give 31, `$-1` gives −1. If no
     hexadecimal digit follows, the field keeps the value in force. A `0x` or `0X` that no
     hexadecimal digit follows is taken as the prefix, not as the digit 0, so it too leaves the
     value in force: `$0x`, `$-0x` and `$0xg` all do, while `$0` gives 0 and `$0x` followed by
     `10` gives 16.
   - **`h` suffix (hexadecimal).** A value that does not start with `$` and whose last byte is `h` or
     `H` is read as hexadecimal from its first byte, in the same way: `1Fh` gives 31, `-1h` gives −1,
     `abch` gives 2748, `0x10h` gives 16, `1F h` gives 31. If there is no hexadecimal digit (`zzh`,
     `H`, and `0xh` or `0xgh`, where the `0x` is again the prefix), the field keeps the value in
     force. `0h` gives 0.
   - **A value that starts with `$` and ends in `h` or `H` takes the `$` form.** So `$10h` gives 16
     (the hexadecimal digits stop at the `h`), not the value in force, which the `h` form would give
     since its text starts with `$`, not with a hexadecimal digit.
   - **Decimal.** Every other value is read as a decimal prefix: optional whitespace, an optional
     `+` or `-`, then decimal digits up to the first non-digit. `12abc` gives 12, `3.9` gives 3,
     `1e3` gives 1, `1,2` gives 1. **If there are no digits the result is 0, not the value in
     force**: `abc`, `--5` and `- 5` all give 0. A `0x` prefix without an `h` suffix is not
     hexadecimal: a `0x` prefix followed by `1F` reads the single digit `0` and gives 0, with the
     bytes after it ignored.
   - **Importer warnings.** The importer warns when a value has no digit in its form (the result is
     then 0 or the value in force, as above). It also warns when bytes other than whitespace follow
     the number, apart from the final `h` or `H` of the `h` form: `12abc`, `3.9`, `1e3`, `1,2` and
     `$10h` warn, and so does a `0x` prefix followed by `1F` in the decimal form, for the bytes after
     its digit `0`, while `1F h` and a `0x` prefix followed by `10h` do not. The result is unchanged.
   - Backing: `measurable` (scenario V3)
   - Confidence: high

4. **Integer overflow wraps.** In every integer form the number formed by the digits read, with the
   sign read, is taken modulo 2³² and read as signed two's complement (Formulas). `2147483648` gives
   −2147483648, `4294967296` gives 0, `99999999999` gives 1215752191, `-2147483649` gives 2147483647, `$100000000` gives 0 and
   `7FFFFFFFFh` gives −1. The importer warns when the wrap changes the number: in the decimal form
   when the signed number read lies outside −2³¹ to 2³¹ − 1, and in the hexadecimal forms when the
   digits read denote 2³² or more. A hexadecimal spelling of a 32-bit pattern, such as `$FFFFFFFF`
   for −1, therefore does not warn (Formulas).
   - Backing: `measurable` (scenario V3)
   - Confidence: high

5. **Interaction with the trim defect.** The `h`-suffix test looks at the stored value's last byte.
   If S-0001's trim defect left trailing whitespace on the value, the suffix is missed and the value
   is read as decimal. With `quirks.ini_trim_defect` off, the stored value has no trailing whitespace,
   so the suffix is found and the value is read as hexadecimal. The importer warns for every value
   that the trim defect affects (S-0001).
   - Backing: `measurable` (scenario V3b, built on S-0001 scenario L7)
   - Confidence: high

### Decimals and percentages

6. **Decimal values are single precision.** The value is read as the longest prefix that forms a
   decimal number: optional whitespace, optional sign, digits with an optional decimal point (either
   side may be empty, as in `.5` and `5.`), an optional exponent (`e` or `E`, optional sign, digits).
   The result is a number of **single** precision, which is exactly representable as a double (claim
   7 uses it as one).
   - `0.1` becomes 0.100000001490116…,
   - `16777217` becomes 16777216,
   - `.4`, `4.e-1` and `4e-1` all give the single nearest to 0.4,
   - a value whose rounding goes past the largest finite single becomes infinity, not the largest
     finite single (this conversion does not depend on the rounding mode of claim 17),
   - `1,5` gives 1, `0,4` gives 0, and `-0x5h` gives −0,
   - the words `inf` and `nan` are not numbers (claim 8).

   The importer warns when bytes other than whitespace and `%` follow the number, as in `1,5`,
   `0,4` and `-0x5h`. The result is unchanged.

   **Rounding of the conversion.** The conversion does not round to nearest. Outside the three
   cases listed below, it follows this rule. The result is the exact value of the decimal, in
   binary, cut to its 24 leading significant bits, and increased by one unit in the last kept place
   (away from zero) when the discarded part is more than one half of that unit. An exact half is
   therefore dropped, even where ties-to-even would round up. When the value is a whole number below 2⁶⁴, however it is spelled (`67108870`, `67108870.0` and `6710887e1` alike), a discarded part of
   exactly three quarters of a unit is dropped too. Examples: `16777219` gives 16777218, `67108870`
   gives 67108864, `74115990` gives 74115984 and `791e7` gives 7909999616, where correct rounding
   gives 16777220, 67108872, 74115992 and 7910000128. (`16777217` gives 16777216 under both rules:
   an exact half is dropped here and rounded to even there.) Outside the three cases, a value whose
   discarded part is not exactly one half or three quarters of a unit comes out correctly rounded.
   Ordinary decimal fractions such as `0.1` or `0.35`, whose binary expansion never ends, are of
   that kind. That the conversion is not correctly rounded is a quirk of the original, with the
   switch `quirks.decimal_conversion_rounding` (Edge cases). It is a property of the conversion, not
   of the rounding mode, and `quirks.chop_rounding` (claim 17) does not affect it.

   Three cases are not covered by this rule. In them the original's result is deterministic (the
   same text always gives the same number), but this spec does not characterise it:
   - magnitudes below the smallest normal single (about 1.18 × 10⁻³⁸), which come out off by one or
     two units of the smallest subnormal, or as zero,
   - a discarded part of exactly one half or exactly three quarters of a unit when the value is not
     a whole number below 2⁶⁴. Either neighbour can result, and which one varies from value to
     value with no rule found: halves are mostly dropped, three quarters mostly rounded up,
   - values that are not whole numbers below 2⁶⁴ and lie within about one part in 2⁶⁰ of a value
     whose discarded part is exactly one half or three quarters of a unit.

   Whole numbers below 2⁶⁴ are in none of the three cases: the rule gives the original's result for
   them however close they lie to such a value. For example, 2⁶³ + 2³⁹ + 1, whose discarded part is
   one half plus 2⁻⁴⁰ of a unit, comes out as 2⁶³ + 2⁴⁰.

   **Our engine**, with `quirks.decimal_conversion_rounding` on (the default), gives the rule's
   result for every decimal outside the three cases. In the second and third cases it applies the
   rule as the Formulas state it exactly (so a three-quarter remainder of a value that is not a
   whole number below 2⁶⁴ rounds up). In the first case it rounds the exact value toward zero to a
   multiple of the smallest subnormal single. In all three cases this is a departure as a design
   choice of scope (glossary *departure*): such decimals are not bad data, but the original's result
   for them is not characterised. In the second and third cases our result and the original's differ
   by at most one unit in the last kept place. With the switch off, every decimal, in the three
   cases too, is correctly rounded (Formulas), so those cases need no departure.

   The importer warns, whatever the switch's setting, by the tests in the Formulas:
   - for every value in one of the three cases, so that an author whose data depends on the last bit
     is told,
   - for every whole number below 2⁶⁴ whose result under the rule differs from the correctly rounded
     one, as for `16777219`, `67108870`, `74115990` and `791e7` but not `16777217`, naming both
     results.

   The result is unchanged.
   - Backing: `measurable` for the syntax of the number (a leading or trailing decimal point, an
     exponent, a comma ending the number) and for a value beyond the largest finite single giving a
     value far above 1 (scenario V4b); `unmeasured` for the precise result: the
     single precision, the rounding rule, the three cases, and whether an overflowing value is
     infinity rather than the largest finite single. No observable known to this spec shows a
     decimal field to one part in 2²⁴, and in the fields checked a threshold or factor cannot tell
     infinity from the largest finite single; if such an observable is found, these parts become
     measurable through it, as claim 17 is through scenario V12. The single-first input
     precision is now `measurable` for the red health ratio through S-0302 scenario D11d
     (also scenario V13 here); that backing supersedes the precision limitation for this field
   - Confidence: high (the syntax, single precision, the rounding rule outside the three cases, and
     overflow to infinity); low (the original's results in the three cases)

7. **A `%` anywhere means "divide by 100".** If the stored value contains a `%` byte anywhere, the
   decimal from claim 6 is multiplied by *c*, the double nearest to 0.01. In the original, the multiplication is done in double precision and rounded toward zero (claim 17). In our engine, the multiplication follows `quirks.chop_rounding`: it is rounded toward zero when that switch is on, and to nearest, ties to even, when it is off (S-0205). `50%` gives 0.5, `0.5%` gives 0.005, `50 %` gives 0.5, `100%%` gives 1.0, and with the original rounding mode `10%` gives 0.09999999999999999 (one unit below 0.1). Without a `%`, the number is taken as it is: **`50` in a percentage field means 50.0, i.e. 5000 %**, not 0.5. Percentage fields and plain decimal fields are therefore one and the same form. Our importer reproduces the original result at the default switch setting and warns for a likely forgotten `%` (claim 18).

   Some fields keep the value in single precision. In the original, and in our engine with `quirks.chop_rounding` on, the double result is narrowed to single toward zero: `10%` in such a field is 0.0999999940395… (the single just below 0.1), while `0.1` without `%` stays 0.100000001490116… With that switch off, the multiplication and narrowing use nearest rounding, ties to even, as S-0205 specifies. Which fields are single precision belongs to the topic specs that define them. The red and yellow health ratios are held as doubles, but their
   text still undergoes claim 6's single conversion first: without `%`, widening that single
   is exact, not a fresh conversion of the text to double. With `%`, the double product is kept
   without a final single narrowing. For example, `0.6` without `%` gives exactly
   0.60000002384185791015625 for either ratio. S-0302 F5 therefore gives a strength of 1000
   a red threshold of exactly 600.00002384185791015625 hit points: a hit from 601 to 600
   reports *now red*, not the later hit from 600 to 599. S-0302 owns that report and its effects.
   - Backing: `measurable` (scenario V4, for the factor of 100; V13 and S-0302 D11d,
     for the red ratio's input precision); `unmeasured` (the yellow ratio's identical input
     precision: V13's original-game observable tests the red ratio, while the original's
     yellow damage-state threshold is fixed at half strength, S-0302 claim 13)
   - Confidence: high (the rounding mode is claim 17's, which holds from before the first data
     file is read)

8. **Unparseable decimals.** If the value starts with no number at all (`abc`, `x5`, `%50`, `inf`,
   `nan`), the original's result is unspecified (glossary *unspecified*): it depends on unrelated
   internal state, not on the value. It is often a tiny positive number (effectively zero), but it
   can be any single value: large or small, of either sign, infinite or not a number. A `%` in the
   value still multiplies it by *c*. A key that is present with such a value never yields the value
   in force in the original.

   **Our engine** treats such a value as absent: the field keeps the value in force, with or without
   `%` (no multiplication by *c*), and the importer warns. This is a departure.
   - Backing: `measurable` (scenario V4, for the original's "often tiny" result in one field)
   - Confidence: medium (the original's result); high (our engine's result, a design choice)

### Integer pairs, triples and quadruples

9. **Comma-separated integers.** Coordinate-like fields read two or three decimal integers:
   - only the first 63 bytes of the stored value are considered, and trailing whitespace of that cut
     is removed without S-0001's trim defect (as in claim 1: the stored value has no leading
     whitespace, so the defect cannot act),
   - whitespace is allowed before each number, and so after each comma,
   - each comma must follow the preceding number's digits *immediately* (`1, 2, 3` is fine, `1 ,2 ,3`
     gives the first number only, and every later component is unread),
   - each number is an optional sign and decimal digits, as in claim 3's decimal form, and wraps as in
     claim 4. There are no hexadecimal forms: a `0x` prefix followed by `10` gives 0 for that
     component and leaves every later component unread, since the `x` is not a comma, and `$10`
     leaves that component and every later one unread,
   - unlike claim 3, a number with no digits is not 0: that component and all later ones are
     unread,
   - items beyond the field's count are ignored.

   **In the original, components that could not be read are unspecified** (not the value in force,
   not 0): `1,2` for a triple leaves the third component unspecified, `1 ,2` leaves the second
   unspecified, and `a,2,3` leaves all three unspecified. The same holds for the map header's
   start-position pairs (Import mapping row C12): `5` gives 5 and an unspecified second number. A
   missing key, or one written with an empty value, gives the value in force for the whole pair or
   triple. S-0009 claim 29 and its header-pair Import mapping now state this distinction explicitly: completion from the value in force belongs to our engine only. Confirm that distinction in both final versions at joint hand-off.

   **Our engine** gives every unread component the matching component of the value in force, and
   keeps every component that was read; the importer warns. So `1,2` for a triple whose value in
   force is (7, 8, 9) gives (1, 2, 9), and `a,2,3` gives (7, 8, 9), as a missing key would. This is
   a departure.

   The importer also warns when bytes other than whitespace follow the last component the field
   reads, such as a fourth number for a triple (`1,2,3,4`) or text after the last number. This
   warning and the warning for unread components apply to claims 10 and 11 too, whatever fills the
   unread components there.
   - Backing: `measurable` (the syntax shared with byte triples by scenario V6 and S-0103 scenario
     PI, which reads the same spellings, including a space before a comma and a component that
     cannot be read; scenario V5, optional, for the pair fields themselves)
   - Confidence: high

10. **Integer quadruples (rectangles).** Rectangle-like fields read four decimal integers with
    claim 9's syntax and 63-byte limit. Unlike pairs and triples:
    - components that could not be read take the corresponding component of the field's component
      fallback (Terms): `1,2` gives 1, 2 and the fallback's last two numbers, and `a` gives the
      whole fallback,
    - a **missing key gives 0, 0, 0, 0**, whatever the component fallback is. So 0, 0, 0, 0 is the
      absent-key default of every rectangle field. A key written with an empty value (`key=`, or
      only whitespace or a comment after the `=`) creates no entry (S-0001 claim 11), so it is a
      missing key and gives 0, 0, 0, 0 too. No value that a data file can store is empty after
      trimming, so the component fallback only ever fills the components that a non-empty value
      does not give. The importer warns when it does (claim 9's warnings).

    What this means for the map size, the local area and the preview size is specified in S-0009.
    Quirk `quirks.absent_rect_is_zero` (also named in S-0009).
    - Backing: `measurable` (scenario V5b)
    - Confidence: high

### Colour and byte triples

11. **Byte triples.** Colour definitions and other byte-valued triples use claim 9's syntax with a
    63-byte limit. Each component is the low 8 bits of the integer read, so 300 becomes 44, −1 becomes
    255 and 256 becomes 0. For unread components:
    - in the colour-definition form they are 0 (`1,2` gives 1, 2, 0, and `a,2,3` gives 0, 0, 0), and
      the importer warns (claim 9's warnings),
    - in the generic byte-triple form they are unspecified in the original. **Our engine** gives each
      unread component the matching component of the value in force, as for claim 9, and the
      importer warns (a departure).

    A missing key, or one written with an empty value, gives the value in force.
    - Backing: `measurable` (scenario V6, jointly with S-0103 scenario PI)
    - Confidence: high (the syntax and the low 8 bits); medium (which fields use which variant, see
      Import mapping)

### Lists

12. **List splitting.** A list value is split at commas.
    - Empty items are skipped, so consecutive commas count as one separator and leading or trailing
      commas are ignored.
    - Items are **not** trimmed. A space after a comma becomes the first byte of the next item.
    - Numeric items tolerate that leading space, because number reading skips leading whitespace.
    - Name items do not: ` Alpha` (with the space) does not match a type called `Alpha`. Depending on
      the field, the item is dropped (prerequisite-style lists) or names a new, different type
      (lists that create types on first mention). The same holds for whitespace before a comma:
      `Alpha ,Beta` gives the item `Alpha` followed by a space. Quirk `quirks.list_items_untrimmed`
      (Edge cases). While it is on, as in the original, the importer warns for every name item that
      begins or ends with a whitespace byte. With it off, name items are trimmed (S-0001's trim,
      without its defect) before they are matched, so ` Alpha` names `Alpha`, and an item made only
      of whitespace becomes empty and is skipped like an empty item (`Alpha, ,Beta` gives `Alpha`
      and `Beta`).
    - In a list that drops items matching nothing, the importer warns for every dropped item other
      than `none` and `<none>`, naming it. The result is unchanged.
    - The list is read from the field's text (claim 1). With the usual 127-byte limit, items beyond the
      limit are lost and the last kept item may be cut short.
    - There is no limit on the number of items other than the text limit, except for fixed-count lists
      (claim 13).
    - Backing: `measurable` (scenario V7)
    - Confidence: high

13. **Fixed-count percentage lists.** Some list fields expect an exact number *k* of items. The case
    known here is the armour-modifier list of a warhead, with one item per armour class, so *k* is the
    number of armour classes (Constants). The *j*-th item belongs to the *j*-th armour class in the
    fixed class order given in Import mapping (row C5). Each item is converted on its own:
    - an item containing `%` gives the item's *integer* decimal prefix (claim 3, decimal form) times *c*, in double precision. In the original this product is rounded toward zero; in our engine it follows `quirks.chop_rounding`, using toward-zero rounding when that switch is on and nearest rounding, ties to even, when it is off (S-0205). Thus, with the original rounding mode, `12.5%` gives 0.12, not 0.125, and `1e2%` gives 0.01, not 1.0. Quirk `quirks.percent_list_integer_prefix` (Edge cases). While it is on, as in the original, the importer warns for every item containing `%` whose decimal (the conversion of the next bullet, at 53 bits) differs from its integer prefix, as for `12.5%` and `1e2%` but not `12%` or `12.0%`. With it off, an item containing `%` gives its decimal at 53 bits times *c*, with the product rounded according to `quirks.chop_rounding` as above, so `12.5%` gives exactly 0.125.
    - an item without `%` gives a decimal converted to **double** precision (not single, unlike
      claim 6), with claim 6's rounding rule applied at 53 significant bits instead of 24, so `0.125`
      gives 0.125 and `0.1` the double nearest to 0.1. Claim 6's overflow to infinity and its three
      uncovered cases carry over, with the smallest normal and subnormal *double* in place of the
      single ones; our engine's results, the switch `quirks.decimal_conversion_rounding` and the
      importer warnings are those of claim 6, at 53 bits,
    - an item with no number gives +0.0: without `%` (`abc`, `inf`, `-abc`) the conversion yields +0.0,
      and with `%` the integer prefix is 0, so the item is 0 × *c* = +0.0. Unlike claim 8, this
      result is fully determined, and our engine reproduces it. The importer warns for such an item,
    - items beyond *k* are ignored, and the importer warns when there are any,
    - an item without `%` whose value is greater than 1 in magnitude is taken as it is, and the
      importer warns as for a percentage field (claim 18),
    - **fewer than *k* items crash the original** while loading. **Our engine** fills the missing
      items, the last ones in class order, with 100 % (1.0 each), and the importer warns, naming the
      warhead and the field; the game starts normally. This is a departure.

    The absent-key default is *k* items of 100 %, each giving 1.0. How these values
    are used belongs to the combat topic.
    - Backing: `measurable` (scenario V8)
    - Confidence: high (conversion, *k*, the class order, and the cause of the crash, which is
      understood); medium (the crash itself, which has not been observed)

### References to types by name

14. **Ordinary type references match case-insensitively.** For list-section values, weapon,
    warhead and prerequisite references, the name is compared with existing type names using ASCII
    case folding (A–Z only). `alpha` finds the type `Alpha`. Except for the fields excluded below,
    the values `none` and `<none>`, in any case, mean "no type" and never create one.
    Section and key names, by contrast, are case-sensitive
    (S-0001 claim 13). What follows from that for a type whose section is spelled in a different case
    is specified in S-0004 (type registration), claim 7.

    The reserved-name rule excludes four fields covered by S-0004 claim 2: a particle
    system's emitted particle type, a particle type's next particle type, an animation's
    spawned particle type and a weapon's attached particle system. In those fields bare
    `none` can name and create a type. Their field-specific `none`, `<none>`, empty-value
    and absent-key effects belong to S-0004 claim 2, including its pointer to S-0006
    claim 5 for the emitted particle type. S-0004 also owns their switches
    `quirks.particle_none_is_a_name` and `quirks.attached_system_none_keeps`; this spec
    defines no additional switch or generalisation to other references or lists.

    This rule also excludes the trigger owner-country field and the exact trigger-event type-name
    rules of S-0011 claims 5 and 21. S-0011 claim 21's type-exists and type-does-not-exist
    events require an exact, case-sensitive type name,
    and treat both no-type spellings as ordinary names, not markers. Its claim 5's trigger
    owner country instead treats the angle-bracketed no-type spelling, ignoring ASCII case,
    as the first registered country. The bare spelling has no marker handling there: it
    undergoes ordinary case-insensitive country type-name or country name-field lookup.
    S-0011 claim 5 also owns the owner-country field's special handling of `<random>`.
    This exception does not extend to ordinary type references.
    The event results, owner failures and compatibility rules remain S-0011's, not new
    switches here. A building's factory kind is a vocabulary word (claim 21), not a
    reference to a registered type, and does not use this claim's marker rules.
    - Backing: `measurable` (scenario V9; S-0011 T14 for event letter case and T22 for
      owner lookup; S-0004 A3 and I4 and S-0006 D4 for their sampled particle-reference
      exceptions); `unmeasured` for the next-particle and spawned-particle exceptions,
      for the reasons in S-0004 claim 2, and for event marker handling, as S-0011 claim 21 explains:
      ordinary lookup failure and marker rejection are indistinguishable when no type has
      that exact marker name
    - Confidence: high; medium for the next-particle and spawned-particle exceptions,
      as in S-0004 claim 2

15. **Type names longer than 24 bytes** are cut when the type is created, and they then no longer
    match their own full spelling. The consequences and the switch `quirks.long_name_duplicates` are
    specified in S-0004, claim 8. They are noted here only because they follow from how list values
    are read: a type list value may hold up to 31 bytes (claim 1, Import mapping row C2), which is
    longer than a stored type name. The importer warns for every type name longer than 24 bytes,
    whatever the switch's setting.
    - Backing: `measurable` (S-0004's scenarios)
    - Confidence: medium

### Named values

16. **Named-value fields.** Fields that take one word from a fixed vocabulary (for example a
    movement zone, a terrain speed class, a building foundation size or a passenger pip colour) read
    the value as text (the limit is per field: 31 or 127 bytes in the cases checked) and compare it
    with the vocabulary ignoring ASCII case. The field's absent-key default is itself a vocabulary
    word, and a missing key normally gives the value in force. For an infantry type's passenger
    pip colour and occupant pip image frame, our engine keeps the held value on an absent read
    as a fixed display-only departure (S-0006 claim 5; S-0111 claim 12), with no quirk switch.
    The original instead shifts the held pip image frame on successive reads of an infantry
    section that omits the corresponding field. This correction changes no passenger capacity,
    loading legality or other gameplay rule. A word that matches nothing does **not** give
    the value in force. It gives a fallback that is fixed per
    field: for some fields a particular vocabulary entry, for others an invalid "no value" result.
    Each topic spec that defines such a field states its vocabulary and its fallback. One worked
    example, used by scenario V11: the passenger pip colour of a type falls back to green, the
    vocabulary's first entry, whatever its value in force. The importer warns for a word that
    matches nothing, naming the fallback it gives.
    - Backing: `measurable` (scenario V11; S-0111 scenario OV7 for the original's absent-pip
      bug); `unmeasured` for keeping the pip value in our engine (display-only correction by policy)
    - Confidence: high (case-insensitive matching, the normal value-in-force rule and pip
      correction); medium (the
      per-field fallback, checked for four fields only)

### Floating-point rounding

17. **Round toward zero, 53-bit significand, for the whole session.** The original runs in the
    *round-toward-zero mode* (glossary) for the whole session, so it is already in force while the
    first data file is read: the original's own floating-point arithmetic uses a 53-bit significand
    and rounds every result toward zero, including every narrowing of a result to single or double
    precision. The one known exception within the original's program is the power function: the
    arithmetic that evaluates it rounds to nearest, and every result computed after it is again
    rounded toward zero (S-0205, claim 5). No result of this spec involves it. For this spec, that
    means:
    - claim 7's multiplication by *c*, and the narrowing to single in single-precision fields, round
      toward zero, as do the multiplications by *c* in claim 13,
    - the result of converting decimal text to a number (claims 6 and 13) does not depend on the
      mode. That conversion's own quirk and its switch `quirks.decimal_conversion_rounding` are
      claim 6's, and are separate from the mode.

    The mode's full scope (conversions to integer, the mathematical functions the original evaluates, and
    which results of other specs it governs) and its single switch `quirks.chop_rounding` belong to
    S-0205 (number domains). This spec defines no switch for it.
    - Backing: `measurable` (scenario V12)
    - Confidence: high. The mode is in force before any data is read, and apart from the power function
      (S-0205, claim 5) no other change of it is known in the original's program. A display or sound
      library outside the program could in principle change it; our engine has no such library.

### Importer warning for a value without `%`

18. **A percentage field without `%` keeps the original's value, with a warning.** When a value for a
    percentage field (Terms; the known ones are listed in Import mapping row C9) contains no `%`
    byte anywhere and its decimal (claim 6) is greater than 1 in magnitude, the importer warns,
    naming the section, the key and the value, and saying that the value is taken as a fraction
    (`50` as 5000 %), not as a percentage. The same applies to each item of the armour-modifier list
    (claim 13, row C5) that has no `%`, with the item's double-precision value in place of claim 6's
    decimal. The warning never changes the field's value: the result is the original's, as claims 7
    and 13 state it, so `50` still gives 50.0. A value with `%`, a value of magnitude 1 or less, and
    a value with no number (claim 8, which warns on its own) do not raise this warning. Fields
    that are not percentage fields never raise it, whatever their value.

    A legitimate value above 100 % written without `%` (for example `1.5` for 150 %) also warns.
    That is intended: the warning tells the author how the value was read, and costs nothing else.
    Which further decimal fields are percentage fields belongs to the topic specs that define them,
    as for single-precision fields (claim 7).
    - Backing: `unmeasured` (it is our importer's behaviour, a design choice; the value it reports is
      claim 7's, measured by scenario V4)
    - Confidence: high

### Combat name and integer lists

19. **No-type items in combat name lists do not occupy places.** The animation-name,
    voxel-animation-name and destruction particle-system-name lists identified in Import
    mapping rows C15, C16 and C18 omit `none` and
    `<none>`, ignoring ASCII letter case. The remaining items keep their relative order,
    including repetitions. A list written `none,Alpha,<none>,Beta` therefore has two entries,
    `Alpha` and `Beta`, not four. Selection indices and random-choice ranges use this retained
    list; a no-type item cannot itself be selected. Claim 12's splitting and whitespace rules
    still apply: a space attached to a no-type word prevents that word from matching while
    `quirks.list_items_untrimmed` is on. Other non-empty names find or create a type
    (S-0004), rather than reserving an empty place. A present non-empty value containing only
    no-type items replaces the list with an empty list; a missing or empty key keeps the list
    in force. These lists start empty before any layer supplies them.
    - Backing: `measurable` (scenario V14; S-0303 A18 for detonation lists)
    - Confidence: high

20. **Debris maxima lists use decimal integer prefixes only.** Every non-empty item contributes
    one signed 32-bit integer by claim 3's decimal form and claim 4's wrapping rule. The scalar
    hexadecimal forms do not apply: `$10` gives 0, `10h` gives 10, and a `0x` prefix followed by `10` gives 0.
    An item with no decimal digits contributes 0 and retains its place, whereas an empty
    comma-separated item contributes no place. For example, `2,,abc,3` gives three entries,
    2, 0 and 3. Claim 3's no-digit, trailing-text and overflow warnings apply to each item,
    with no suffix exception for `h`. The text limit is 511 bytes, not the common 127 bytes.
    A missing or empty key keeps the list in force; it starts empty. A present non-empty
    value replaces it. The maxima and type lists are independent: omitting a no-type name
    from the type list does not omit an integer from the maxima list. The first retained
    type pairs with the first integer, and so on; S-0303 F6 and S-0312 F1 own the effects
    and the safe results for mismatched list lengths.
    - Backing: `measurable` (scenario V15; S-0303 A19); `unmeasured` (the 511-byte
      reader limit: S-0001's physical-line limit cuts a value sooner once the key and `=`
      take part of that line, so file input cannot reach this separate limit)
    - Confidence: high

### Factory kind vocabulary

21. **A building's factory kind is a named value, not a type reference.** The four vocabulary
    words in Import mapping row C19 make it a factory building of infantry, vehicles,
    aircraft or buildings, respectively. They name categories, not individual registered
    types. The field uses claim 1's text reading with a limit of 31 bytes, then a whole-text
    comparison ignoring ASCII letter case. Leading whitespace has already been removed
    from the stored value (S-0001); trailing whitespace is removed after the cut, including
    whitespace left by the trim defect. Internal whitespace is retained. A prefix of a
    vocabulary word or a vocabulary word followed by non-whitespace text is not a match.
    Missing or empty keeps the value in force; before any layer the field says no factory.
    A non-empty unmatched word gives no factory, not the value in force, with claim 16's
    importer warning. The explicit no-factory vocabulary word in C19 also gives no factory.

    Additional vocabulary words can be recognised without naming any of those four
    production kinds; C19 gives examples. Such a building cannot serve any of the four
    production lines on that basis alone: the production specs require the corresponding
    factory kind (S-0403 Terms, S-0404 claim 7, S-0407 claim 1). In particular, the words
    for object instances are not aliases for the corresponding type-kind words. Recognition
    of another category does not grant a fifth production line. This claim concerns the
    imported production kind only, not every order or visual property of such a building.
    - Backing: `measurable` (scenario V16)
    - Confidence: high

### Auto-deploy integer list

22. **Auto-deploy delays use decimal-prefix integer items, not scalar hexadecimal forms.**
    The game-wide auto-deploy delay list uses the same item form and 511-byte text limit
    as the debris maxima (claim 20). Each non-empty comma-separated item contributes
    one signed 32-bit integer, using claim 3's decimal prefix and claim 4's wrapping.
    `$10` contributes 0, `10h` contributes 10, and a `0x` prefix followed by `10`
    contributes 0. An unreadable non-empty item contributes 0 and keeps its place,
    not a prior value at that position. Empty items take no place: `5,,40` gives
    (5, 40), while `5,abc,40` gives (5, 0, 40). A whitespace-only item remaining after claim 1's whole-text cut and trailing
    trim is non-empty and contributes 0. Claim 12's splitting rules apply. Claim 3's no-digit,
    trailing-text and overflow warnings apply item by item, with no hexadecimal
    suffix exception. The imported quantities are ticks; conversion uses exact integer
    arithmetic, with no floating-point calculation or rounding.

    A present non-empty value replaces the whole held list. A missing or empty key
    keeps the held list. S-0215 claim 31 owns which retained item a house uses, the
    deployment test, and the safe result when the list has no item at that index.
    This claim neither pads the list nor assigns meanings to extra entries. Claim 1's
    text-cut warning applies.
    - Backing: `measurable` (scenario V17 for item conversion, placement and text
      extending beyond 127 bytes); `unmeasured` (the exact 511-byte reader limit:
      S-0001's physical-line limit cuts file input sooner because the key and `=`
      occupy part of the line)
    - Confidence: high

### Economy and computer-timer integer lists

23. **The virtual-purifier, extra-credit and computer-timer lists use decimal-prefix
    integer items too.** Each of the three lists in Import mapping rows C21 to C23
    uses claim 20's item form and 511-byte text limit, not the common 127-byte limit.
    Each non-empty comma-separated item contributes one signed 32-bit integer:
    optional leading whitespace and sign, then decimal digits, stopping at the first
    other byte. There is no scalar hexadecimal dispatch. `$10` contributes 0,
    `10h` contributes 10, and a `0x` prefix followed by `10` contributes 0.
    An item with no decimal digits, such as `abc`, contributes 0 and keeps its
    place; it neither disappears nor keeps a prior value at that position.
    Claim 4's wrapping applies. Conversion is exact integer arithmetic with no
    floating-point operation or rounding. The retained integers represent purifier
    counts, integer percentages and timer durations in ticks, respectively; their
    later use is not part of the conversion.

    Claim 12's comma splitting applies: empty items take no place, so `5,,7`
    gives (5, 7), while `5,abc,7` gives (5, 0, 7). A whitespace-only item left
    after claim 1's whole-text cut and trailing trim is non-empty and gives 0.
    Claim 3's no-digit, trailing-text and overflow warnings apply to every item,
    with no hexadecimal suffix exception; claim 1's text-cut warning applies too.

    Missing or empty keys keep the entire list in force. A present non-empty
    value replaces the entire list, without retaining or adding positions from
    the previous list. A value consisting only of commas therefore replaces it
    with an empty list. Before any rules layer supplies them, these lists are
    empty. Index selection and the effects or safe results of a too-short list
    remain S-0402 claim 12 and S-0412 claim 3. In particular this conversion does
    not grant AI houses the original's computer-only income or setup advantages,
    and the computer timer has no effect in our engine.
    - Backing: `measurable` (scenario V18 for the item form, positions, replacement,
      retention and text beyond 127 bytes; part c observes the original's initial
      timer duration without requiring its computer-player behaviour), and
      `unmeasured` (the exact 511-byte reader limit: S-0001's physical-line limit
      cuts file input sooner because the key and `=` occupy part of that line)
    - Confidence: high

## Formulas

Notation. For a real number *z*: RN₅₃(*z*) and RN₂₄(*z*) are *z* rounded to the nearest double
and to the nearest single, ties to even, with the format's subnormals and with infinity for a
magnitude of at least the largest finite value plus half a unit in its last place (correct
rounding); RZ₅₃(*z*) and RZ₂₄(*z*) are *z* rounded toward zero to a double and to a single. RZ is
the original's rounding for the floating-point operations of this spec (claim 17; the mode as a
whole belongs to S-0205, number domains). In our engine every RZ₅₃ and RZ₂₄ result of this spec
has exactly the exponent range of a double or a single; S-0205 (claim 8) states the same for every
intermediate result of our engine. Whether the original gives an
intermediate result a wider exponent range makes no difference to any result of this spec: a
product by *c* cannot overflow, and rounding toward zero first to a wider range and then to a
double or a single gives the same value as one rounding toward zero. *a* mod *n* is always in
0 ≤ … < *n*.

Integer forms (claims 3 and 4). Let *D* be the non-negative integer that the digits read (at least
one) denote in base *B* (10 or 16), and *s* the sign (−1 if a `-` was read, else +1). All arithmetic
is exact integer arithmetic; the result is a signed 32-bit integer.

> result = ((*s* · *D* + 2³¹) mod 2³²) − 2³¹, the one integer from −2³¹ to 2³¹ − 1 that is
> congruent to *s* · *D* modulo 2³²

The importer's overflow warning (claim 4) is raised exactly when *s* · *D* < −2³¹ or *s* · *D* ≥ 2³¹
in the decimal form, and exactly when *D* ≥ 2³² in the two hexadecimal forms. It never changes the
result.

Decimals (claim 6). Let *x* be the exact rational magnitude of the decimal prefix, and *P* the
number of kept bits: 24 for claim 6, 53 for claim 13. The result carries the prefix's sign; *x* = 0
gives a zero of that sign. The rule below is the original's outside claim 6's three uncovered
cases, and, with `quirks.decimal_conversion_rounding` on, our engine's for every magnitude from the
smallest normal value of the format upwards. For *x* > 0:

> *x* = *m* × 2^*e*, with *e* an integer and 2^(*P*−1) ≤ *m* < 2^*P* (*m* real); *t* = ⌊*m*⌋,
> *r* = *m* − *t* (the discarded part, 0 ≤ *r* < 1)
>
> *W* is the condition that *x* is a whole number and *x* < 2⁶⁴
>
> *f* = *t* × 2^*e* if *r* ≤ 1/2, or if *r* = 3/4 and *W* holds; *f* = (*t* + 1) × 2^*e* otherwise
>
> magnitude of the result = *f* if *f* does not exceed the largest finite value of the format,
> and infinity otherwise

Below the smallest normal value of the format our engine, with the switch on, instead gives *x*
rounded toward zero to a multiple of the smallest subnormal value. With the switch off, the
magnitude of the result is RN₂₄(*x*) for *P* = 24 and RN₅₃(*x*) for *P* = 53, for every *x* > 0
and in both settings of `quirks.chop_rounding`. With *P* = 24 the result, *y*, is a single and so
exactly a double; with *P* = 53 (claim 13) it is a double.

Importer warning tests (claims 6 and 13), whatever the setting of
`quirks.decimal_conversion_rounding`. With *n* the smallest normal value of the format, the
importer warns for a decimal that the original's result is not characterised exactly when

> (0 < *x* < *n*) or (*W* does not hold and (|*r* − 1/2| ≤ 2^(*P*−60) or |*r* − 3/4| ≤ 2^(*P*−60)))

The first part is claim 6's first case. The second part covers its second and third cases with a
margin; it never warns for a whole number below 2⁶⁴, which is in none of the three cases. The
importer warns that the original's result differs from the correctly rounded one exactly when

> *W* holds and (*r* = 3/4, or *r* = 1/2 and *t* is odd)

These are exactly the whole numbers below 2⁶⁴ for which the rule and RN give different results:
with *r* = 1/2 and *t* even both keep *t*. No decimal raises both warnings. Neither test changes the
result.

Percentages (claim 7), with *y* the signed result of claim 6. Let *Q*₅₃(*z*) and *Q*₂₄(*z*) mean rounding *z* according to `quirks.chop_rounding`: RZ₅₃ and RZ₂₄ when the switch is on, and RN₅₃ and RN₂₄ when it is off (S-0205). A value with no number is claim 8's case: in our engine it leaves the value in force, and neither line below applies.

> *c* = RN₅₃(1/100)
>
> without `%`: result = *y*
>
> with `%`: result = Q₅₃(*y* × *c*), a double; in a single-precision field, Q₂₄(Q₅₃(*y* × *c*))

Missing-`%` warning (claim 18). With *y* the decimal of claim 6 (for an armour-modifier item, the
double of claim 13), the importer warns for a value of a percentage field, or an armour-modifier item,
exactly when

> the value contains no `%`, the value has a number (claim 8), and |*y*| > 1

The comparison is exact, on the value as converted; infinity counts as greater than 1. The warning
never changes the result.

Fixed-count percentage list item (claim 13), with *i* the integer from the item's decimal prefix (claim 3, decimal form), and *Q*₅₃ as defined under Percentages:

> with `%`: Q₅₃(*i* × *c*), where *i* = 0 if the item has no digits; with `quirks.percent_list_integer_prefix` off, Q₅₃(*d* × *c*) instead, where *d* is the item's decimal by claim 6's conversion with *P* = 53 (+0.0 if the item has no number)
>
> without `%`: claim 6's conversion with *P* = 53, giving a double (largest finite, smallest normal and
> smallest subnormal values of the double format); +0.0 if the item has no number
>
> missing items (fewer than *k*), in our engine: 1.0 each

The integer-prefix warning (claim 13) is raised for an item containing `%` exactly when the item has
a number and *d* ≠ *i*, compared exactly. It never changes the result.

Byte triple component (claim 11): *b* = *v* mod 256 (exact integer arithmetic, no rounding), so
0 ≤ *b* ≤ 255, with *v* the signed 32-bit
integer from claim 9.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Percentage factor | RN₅₃(1/100), the double nearest to 0.01 | ratio | *c* in the formulas. Applied when `%` is present |
| Tuple text limit | 63 | bytes | Integer pairs, triples and quadruples, colour and byte triples |
| Common text limit | 127 | bytes | Most text and list fields |
| Decimal-integer-list text limit | 511 | bytes | Debris maxima, auto-deploy delays and the three economy/setup lists, claims 20, 22 and 23; physical-line truncation imposes a smaller limit |
| Type-list text limit | 31 | bytes | Values in type list sections |
| Maximum stored type-name length | 24 | bytes | Longest type name kept when a type is created (claim 15) |
| Integer width | 32 | bits | Integer results, two's complement, wrap modulo 2³² |
| Number of armour classes | 11 | armour classes | *k* of the armour-modifier list (claim 13); the class order is in Import mapping row C5 |

## State and ordering

Value-form selection for an integer field:

| Stored value | Form used | Failure result |
|---|---|---|
| first byte `$` (whatever the last byte) | hexadecimal after the `$` | value in force |
| first byte not `$`, last byte `h` or `H` | hexadecimal from the first byte | value in force |
| any other value | decimal prefix | 0 |

Decimal field: always the decimal prefix, then × *c* if any `%` is present. On failure the result is
unspecified in the original; our engine leaves the value in force and warns (claim 8).

## Edge cases and quirks

- `quirks.absent_rect_is_zero` (YR default: on). A missing rectangle key, or one written with an
  empty value, gives four zeros. Off: it gives the field's component fallback.
- `quirks.percent_list_integer_prefix` (YR default: on). An armour-modifier item containing `%`
  uses only its integer prefix, so `12.5%` gives 0.12 while the same spelling in a percentage field
  gives 0.125 (claims 7 and 13). The importer warns for every item whose value this changes. Off:
  the item's full decimal, at 53 bits, times *c*.
- `quirks.list_items_untrimmed` (YR default: on). List items keep the whitespace around them, so a
  name item written after `, ` or before ` ,` matches no type, and is dropped or names a new type
  (claim 12). The importer warns for every such name item. Off: name items are trimmed before they
  are matched. Types created by a list item (S-0004, type registration) are created from the item
  as this switch leaves it, so with the switch off ` Alpha` finds or creates `Alpha`.
- `quirks.decimal_conversion_rounding` (YR default: on). The conversion of decimal text (claims 6
  and 13) is not correctly rounded: an exact half of a unit in the last kept place is dropped even
  where ties-to-even would round up, and so is an exact three quarters for a whole number below
  2⁶⁴, so such a value comes out one unit in the last kept place below the correctly rounded one
  (`16777219` gives 16777218, not 16777220). On: our engine reproduces this by claim 6's rule, with
  its departures in the three uncovered cases. Off: every decimal is correctly rounded, to the
  nearest single (claim 6) or double (claim 13), ties to even, subnormals included (Formulas). The
  switch governs only the conversion of text, in both settings of `quirks.chop_rounding`; the
  rounding of arithmetic is S-0205's mode. The importer warns, with the switch on or off, for every
  whole number below 2⁶⁴ whose two results differ, and for every value in the three uncovered cases
  (claim 6).
- Round-toward-zero arithmetic (claims 7, 13 and 17): this spec defines no switch. The mode and its
  single switch `quirks.chop_rounding` (default: the original behaviour) belong to S-0205 (number
  domains).
- Long type names: see S-0004 (`quirks.long_name_duplicates`). The importer warns for a type name
  longer than 24 bytes, with the switch on or off (claim 15).
- Departures, with no quirk switch of their own.
  - Bad data. The original's results for an unparseable decimal (claim 8), for unread components of
    pairs, triples (claim 9) and generic byte triples (claim 11) depend on unrelated internal state
    and cannot be reproduced, and the original crashes on a too-short fixed-count list (claim 13).
    Our engine gives the value in force for claim 8, the value in force's components for claims 9
    and 11, and 100 % items for claim 13.
  - Scope. Claim 6's three uncovered cases, and the same cases at 53 bits in claim 13, are
    deterministic in the original but not characterised by this spec. This departure applies
    while `quirks.decimal_conversion_rounding` is on (the default): our engine then uses the rule of
    the Formulas in the second and third cases and rounds toward zero below the smallest normal
    value. With the switch off, these values are correctly rounded like every other decimal, which
    is the switch's alternative behaviour, not a departure.

  The importer warns for every one of them, and the game still starts.
- The format's rules for malformed or out-of-range input, reproduced with no switch. Each of these
  gives a deterministic result that does not depend on unrelated state, and the same for stock data,
  maps and mods. These inputs lie outside the value form as the original defines it, and the result
  is that form's own rule for them, not a defect in reading a well-formed value. Reading them
  differently (`on` as true, an overlong list in full, a large integer without wrapping) would extend
  the format rather than correct it. So our engine reproduces the format's rule, as it does for
  stock data, and the importer warns, which is how a mod author learns of it:
  - an integer value with no digit in its form gives 0 (decimal form) or the value in force
    (hexadecimal forms), and bytes after the number are ignored (claim 3),
  - an integer outside the signed 32-bit range wraps (claim 4),
  - bytes after a decimal number, other than whitespace and `%`, are ignored (claim 6),
  - a value longer than the field's text limit is cut, which can drop list items or cut a name
    (claims 1 and 12),
  - a boolean whose first byte is none of the six recognised ones leaves the value in force; this
    includes `on` and `off` (claim 2),
  - unread components of a rectangle take the component fallback, and those of a colour definition
    are 0; bytes after the last component a tuple reads are ignored (claims 9 to 11),
  - a name item that matches no type is dropped from a list that does not create types (claim 12),
  - an armour-modifier item with no number gives +0.0, and items beyond *k* are ignored (claim 13),
  - a named-value word that matches nothing gives the field's fallback, not the value in force
    (claim 16).
- A percentage written without `%` (claim 7) is the most likely real-world mistake. The original
  takes the number as it is, and so does our engine, with no quirk switch; the importer warns when
  the value of a percentage field, or an armour-modifier item, exceeds 1 in magnitude (claim 18).

## Import mapping

| Row | Data | Form |
|---|---|---|
| C1 | `Cost` of an infantry, vehicle, aircraft or building type | Integer (claims 3 and 4). Observable on the sidebar |
| C2 | Type list sections such as `[InfantryTypes]`, `[BuildingTypes]`, `[Warheads]` | Text of at most 31 bytes per value, each naming a type (claims 1, 14 and 15) |
| C3 | `Prerequisite` of a buildable type | List of building names and the generic prerequisite words (`POWER`, `FACTORY`, `BARRACKS`, `RADAR`, `TECH`, `PROC`), text limit 127 bytes. Items that match nothing are dropped (claim 12) |
| C4 | `Owner`, `RequiredHouses`, `ForbiddenHouses`, `SecretHouses` | List of country names, text limit 127 bytes (claim 12) |
| C5 | `Verses` of a warhead | Fixed-count percentage list (claim 13): 11 items, text limit 127 bytes. Item *j* belongs to the *j*-th armour class in this fixed order, by the word a type's `Armor` takes: `none`, `flak`, `plate`, `light`, `medium`, `heavy`, `wood`, `steel`, `concrete`, `special_1`, `special_2`. An item without `%` above 1 in magnitude raises claim 18's warning |
| C6 | Entries of `[Colors]` | Colour-definition byte triples (claim 11, unread components 0) |
| C7 | Other byte triples: colour fields of the rules file's `[AudioVisual]` and `[Radiation]` sections, the entries of the rules file's colour-adjustment section, and colour fields of some types (for example `LaserColor` of a particle system) | Generic byte triple (claim 11: unread components unspecified in the original; in our engine, the value in force's components) |
| C8 | `PrimaryFireFLH`, `SecondaryFireFLH` and the other fire-position triples of a type (art file) | Integer triple (claim 9) |
| C9 | Percentage fields (Terms, claim 18) known here: `ConditionYellow`, `ConditionRed` (in `[AudioVisual]`); `RefundPercent`, `RepairPercent`, `SpyMoneyStealPercent`, `CrewEscape` (in `[General]`). Topic specs may mark further decimal fields | Decimal with optional `%` (claims 6 to 8). Both health ratios keep the widened-single input, or its double percent product, as doubles, as claim 7 specifies; missing or empty keeps the value in force, with starting defaults owned by S-0302. Other fields' storage precision and defaults remain their topic specs'. A value without `%` above 1 in magnitude raises claim 18's importer warning; the value is unchanged |
| C10 | Boolean fields such as `Selectable` | Boolean (claim 2) |
| C11 | `Speed` and `Strength` of a type | Integer (claim 3) |
| C12 | `Waypoint1` … `WaypointN` in a map's `[Header]`, *N* = the header's start-position count (S-0009) | Integer pair (claim 9). An unread number is unspecified in the original; our engine keeps the matching number of the value in force and warns (a departure). Missing or empty keeps the whole pair in force. S-0009 claim 29 owns which marks supply that value for pairs beyond the eighth |
| C13 | Rectangles in a map, such as `LocalSize` and `Size` in `[Map]` and the preview's `Size` | Integer quadruple (claim 10). Absent or empty: 0, 0, 0, 0. The map consequences and each field's component fallback are in S-0009 |
| C14 | `MovementZone`, `SpeedType`, `Pip` of a type, `Foundation` of a building (art file) | Named value (claim 16) |
| C15 | Warhead `AnimList`; `[CombatDamage]` `SplashList`; type `DebrisAnims`, `Explosion`, `DestroyAnim`; `[General]` `DamageFireTypes`, `MetallicDebris` | Animation-name lists, 127-byte text limit. Claims 12, 14 and 19: `none` and `<none>` are omitted, not empty slots. Start empty; missing or empty keeps the list in force |
| C16 | Type or warhead `DebrisTypes`; `[General]` `BarrelDebris` | Voxel-animation-name lists, 127-byte text limit, otherwise as C15 (claim 19) |
| C17 | Type or warhead `DebrisMaximums` | Decimal-prefix integer list (claim 20), 511-byte text limit. Unreadable non-empty items retain a place with 0; empty items are skipped. Start empty; missing or empty keeps the list in force |
| C18 | Type `DestroyParticleSystems` | Particle-system-name list, 127-byte text limit; claims 12, 14 and 19. `none` and `<none>` are omitted, not slots. Start empty; missing or empty keeps the list in force |
| C19 | Building type `Factory` | Named value (claim 21), 31-byte text limit. Producing vocabulary: `Infantry` immediately followed by `Type` = infantry, `Unit` immediately followed by `Type` = vehicles, `Aircraft` immediately followed by `Type` = aircraft, `Building` immediately followed by `Type` = buildings. Each is one word with no intervening space. Whole text compared ignoring ASCII case, after claim 1's cut and trailing trim. `<none>` explicitly means no factory; bare `none` is unmatched and also gives no factory. Missing or empty keeps the value in force, initially no factory. Non-empty unmatched text gives no factory, with a warning. Additional recognised words include `Terrain` immediately followed by `Type` (one word), and bare `Infantry`, `Unit`, `Aircraft` and `Building`; none names one of the four production kinds. These are not aliases for the four producing words ending in `Type` |

| C20 | `[General]` `AIAutoDeployFrameDelay` | Decimal-prefix integer list in ticks (claim 22), 511-byte text limit, not 127. No scalar hexadecimal forms. Unreadable non-empty items give 0 and retain their place; empty items are skipped. Present non-empty replaces the list; missing or empty keeps the held list. Index selection, starting default and too-short-list outcome remain S-0215 claim 31 and its Import mapping |
| C21 | `[General]` `AIVirtualPurifiers` | Decimal-prefix integer list of purifier counts (claim 23), 511-byte text limit. No scalar hexadecimal forms. Unreadable non-empty items retain a place with 0; empty items are skipped. Initially empty; missing or empty keeps the whole list in force, present non-empty replaces it. Index selection and too-short-list outcome: S-0402 claim 12 |
| C22 | `[General]` `MultiplayerAICM` | Decimal-prefix integer list of integer percentages (claim 23), 511-byte text limit. No scalar hexadecimal forms or decimal-percentage conversion: `10%` gives the integer 10. Unreadable non-empty items retain a place with 0; empty items are skipped. Initially empty; missing or empty keeps the whole list in force, present non-empty replaces it. Selection, subsequent percentage arithmetic and too-short-list outcome: S-0412 claim 3 |
| C23 | `[General]` `AIHateDelays` | Decimal-prefix integer list of timer durations in ticks (claim 23), 511-byte text limit. No scalar hexadecimal forms. Unreadable non-empty items retain a place with 0; empty items are skipped. Initially empty; missing or empty keeps the whole list in force, present non-empty replaces it. Original-only timer selection: S-0412 claim 3; no effect in our engine |

## Measurement scenarios

Common setup: a tiny purpose-made multiplayer map whose INI part carries an override section for
one buildable infantry type of the player's side (the test type). The glossary's *measurement
setup* applies: stock data, no mods and no engine add-on active, any game speed. A presentation-only
display compatibility layer may be used and is named generically in the measurement record. Unless a scenario gives another count, run each stated test variant once. V6 and V11 use
a named test file, a loose rules copy that is removed after the run. Scenarios built on another
spec's scenario use that scenario's named test data.

- **V1 (text limit).** Give the test type a prerequisite list (row C3) of 126 bytes of filler made of
  a long run of commas, followed by one existing building name that crosses the 127-byte mark.
  Expected: the building name is cut, so it is dropped and the type is buildable without that
  building. Shorten the filler so the name fits. Expected: the building is required. Falsified by:
  the building being required in the first setting, or not required in the second.
- **V2 (booleans).** Set the test type's selectable flag (row C10) to `no`, then to `Nope`, then to
  `off`. Expected: not selectable, not selectable, selectable (`off` is unrecognised, so the
  value in force applies: the test type is as selectable as it is without the override). Falsified
  by: `off` making it unselectable, or `no` or `Nope` leaving it selectable.
- **V3 (integers).** Set the cost (row C1) in turn to `$64`, `64h`, `0x` followed by `64`, `abc`,
  `4294967396`, `100junk`, `$0x` and `0xh`. Expected sidebar costs: 100, 100, 0, 0, 100, 100, then
  for both of the last two the cost the type has without the override (the value in force).
  Falsified by: the cost without the override appearing for the `0x` spelling or for `abc`, 0
  appearing for `$0x` or `0xh`, or a cost other than 100 for `4294967396` (for instance a clamped
  value, 0 or the cost without the override).
- **V3b (`h` suffix and the trim defect).** Set the cost (row C1) with the value text `····64h··`
  (· is one space), as in S-0001 scenario L7d. S-0001's trim defect then leaves one trailing space on
  the stored value. Expected sidebar cost: 64 (the value is read as decimal). Falsified by: 100.
- **V4 (percentages).** In the map's rules part, set the red health threshold (row C9) to `10%`, and
  the yellow threshold in turn to `40%`, `0.4` and `40`. Observable: the selected unit's health-bar
  colour at full health, at about 50 % and at about 30 % of its strength. Expected: for `40%` and
  `0.4`, green at about 50 % and yellow at about 30 %. With `40` the threshold is 4000 %, so the
  bar is yellow even at full health. Falsified by: for `40%` or `0.4`, any colour other than green
  at about 50 % or yellow at about 30 % (for `40%`, yellow at about 50 % would mean the `%` was not
  divided by 100); for `40`, a bar that is not yellow at full health (that would mean `40` was read
  as 0.4). Then set the yellow threshold to `abc`. Expected: the bar does not turn yellow at about 30 % health (a threshold of about 0).
  Falsified by (this step): yellow at about 30 %. That would mean the threshold is not tiny; it cannot tell the
  value in force from a large unspecified value (claim 8), so this step tests only the "often tiny"
  part of claim 8, for this field.
- **V4b (decimal syntax and overflow).** V4's setup and procedure, with the red threshold at `10%`
  and the yellow threshold set, one run each, to the values below. Observable: the selected unit's
  health-bar colour at full health, at about 50 % and at about 30 % of its strength.

  | Yellow threshold | Expected | Falsified by |
  |---|---|---|
  | `.4` | green at 50 %, yellow at 30 % (0.4) | green at 30 %: a leading decimal point not read as part of a number |
  | `4.e-1` | green at 50 %, yellow at 30 % (0.4) | yellow at 50 %: reading stopped at the trailing decimal point, giving 4 |
  | `4e-1` | green at 50 %, yellow at 30 % (0.4) | yellow at 50 %: the exponent not read, giving 4 |
  | `0,4` | green at 50 % and at 30 % (0: the comma ends the number) | yellow at 30 %: the comma taken as a decimal point |
  | `1e39` | yellow even at full health (the threshold overflows to infinity, far above 1) | green at full health: an overflowing value read as 0 or as no number |

  The `1e39` run cannot tell infinity from the largest finite single, which would also give yellow
  at full health (claim 6 keeps that part `unmeasured`).
- **V5 (integer pairs).** The map header's start-position pairs (row C12) are the integer-pair
  fields a map can set. Write one start position as `x, y` and another as `x ,y`. Expected: the
  first appears at (x, y) in the skirmish map preview, and the second at x with an unpredictable y.
  Falsified by: the second start position appearing at (x, y). This
  depends on whether the preview shows header start positions, so it is optional. The fire-position
  triples (row C8) live in the art file, which a map cannot override.
- **V5b (rectangles).** A tiny purpose-made map whose map size is written in full (row C13). Its
  local-area entry (row C13) is, in turn: removed; written with nothing after the `=`; and written
  in full as the whole map. One run each. Observable: whether the map loads and plays, and the area
  the view can scroll over. Expected: the empty entry behaves exactly like the removed one, and
  both differ from the whole map. Then write the local area in full and give the map-size entry (row
  C13), in turn, nothing after the `=`, no entry at all, and three numbers only (`0,0,30`). Expected:
  the empty value behaves exactly like the missing entry, not like a 50 × 50 map; the three-number
  entry gives a map 30 wide and 50 high, the height coming from the map size's component fallback
  (its legal cells follow S-0009 claim 1, so an object placed in a cell that is legal only on a map
  at least 50 high is present). Falsified by: an empty entry behaving like the whole map or like a
  50 × 50 map, or differently from the missing entry; or the three-number entry giving a map of
  another size.
- **V6 (colour triples).** A colour definition (row C6) is drawn as a house colour: in a skirmish,
  the player in lobby colour slot *s* has the remap pixels of its units drawn from the ramp of the
  definition at a fixed position chosen by *s* (S-0103 claims 6 and 10). A test map can neither
  change which position a slot shows (S-0103 claim 10) nor redefine an existing name (S-0103 claim
  5), so, unlike the common setup's stock data, the test data is a loose copy of the base rules (a
  named test file in the sense of the glossary's *measurement setup*, removed after the run), set
  up as in S-0103 scenario PI, whose definitions at the positions of slots 0 to 3 are, in turn,
  `300,255,255`, `44,255,255`, `40,255` and `a,255,255`. Use S-0103 scenario PC's map and procedure: one game per slot,
  with a screenshot of the player's infantry on flat ground. Observable: the infantry's remap pixels
  (the palette indices of the sprite's remap range), compared between slots and with S-0103 PC's
  ramp computation. Expected:
  - slots 0 and 1 show the same yellow ramp, that of (44, 255, 255): 300 keeps its low 8 bits,
  - slot 2 shows an all-black remap range, the ramp of (40, 255, 0): the missing third component is 0,
  - slot 3 shows an all-black remap range too, the ramp of (0, 0, 0): reading stops at the
    unreadable first component, so the two 255s are never read.

  Falsified by: slot 0 differing from slot 1 (for example the red of a value clamped to 255); slot 2
  or slot 3 showing any non-black remap pixel: a missing component that is not 0, or, for slot 3,
  the red ramp of (0, 255, 255) tabulated in S-0103 scenario PC, which reading `a` as 0 and
  carrying on would give. S-0103 scenario PI covers further spellings of the same form (negative numbers, a space
  before a comma, the 63-byte limit) and can be run in the same session.
- **V7 (list spacing).** Give the test type a prerequisite list (row C3) of two existing buildings
  written with a space after the comma. Expected: only the first building is required. Without the
  space, both are required. Falsified by: both buildings required with the space, or only one
  without it.
- **V8 (percentage list).** Give a weapon's warhead (row C5) one fewer item than there are armour
  classes, in a test map where that warhead is loaded. Expected: the original crashes while loading.
  Then, in the map, give the test type the first armour class of row C5's order (`none`). Pre-place
  an enemy unit whose primary weapon's warhead has a list with `12.5%` as its first item and `100%`
  for the other ten, and set that weapon's damage to 200. Let it fire on the test type. Observable:
  the test type's strength lost per shot. Expected: about 24 damage per shot (23 if the damage
  formula truncates a product just below 24), and in no case 25, which a factor of 0.125 would give. This is confirmed
  jointly with the combat topic's damage-rounding spec. Falsified by: the map loading and playing
  with the short list, or 25 damage per shot with `12.5%`.
- **V9 (type name case).** Give the test type a prerequisite naming a building in lower case.
  Expected: the requirement is enforced, since the match is case-insensitive. Then set the value
  `none` for the test type's primary weapon. Expected: the unit is unarmed. Falsified by: the type
  being buildable without the lower-case building, or the unit firing a primary weapon.
- **V10 (long names).** Covered by S-0004's scenarios.
- **V11 (named values).** Make the test unit a ground vehicle of the player's side and give it, in
  the map's override section, the amphibious movement zone and the amphibious terrain speed class
  (row C14), both written in lower case. Expected: the vehicle drives onto water exactly as it does
  with the words capitalised. Falsified by: the vehicle refusing water while the capitalised spelling
  allows it. Second part, a word that matches nothing. Named test file: a loose copy of the base
  rules in which the test type's passenger pip colour (row C14) is `yellow`, so that the value in
  force when the map layer is read is not the field's fallback. Run 1, control: the map without a
  pip entry. Run 2: the map's override section gives the test type's passenger pip colour a word
  that matches no vocabulary entry, such as `bogus`. In each run load one test infantry into a
  transport and select the transport. Expected: run 1 shows a yellow passenger pip; run 2 shows a
  green one, the fallback of that field. Falsified by: a yellow pip in run 2, which would mean the
  unmatched word gave the value in force (or a run 1 pip that is not yellow, which voids the run).
- **V12 (rounding mode).** Use S-0104 scenario L5 (neutral test lighting, an infantry pixel on flat
  ground). Pick a pixel whose palette colour has a **red** channel that is a multiple of 8, for
  example 248. Expected: the red channel shows the 16-bit step of one below that value (247's
  step), which needs rounding toward zero in the conversion of the 1000 ‰ tint to its factor
  (S-0104, claim 7 and its Formulas). Falsified by: the step of the palette value itself (248's),
  which round-to-nearest arithmetic gives. S-0108 scenario B1 (a build-up delay one tick shorter
  than round-to-nearest gives) corroborates the claim but cannot falsify it, since how that spec's
  time value is first converted is not established.

- **V13 (single-first health ratios, Q-0075).** Use S-0302 D11d's purpose-made skirmish
  map, one human player and one distant bystander computer player, game speed 2, director
  and mods off. Named map overrides give a test vehicle strength 1000, starting health 601,
  red ratio written `0.6`, yellow ratio `0.75`, one smoke damage effect, and an attacking
  weapon that delivers exactly 1 damage per hit. Record health and the first tick of smoke
  with video and a read-only recorder, over one run of two hits. Expected: smoke begins
  on the hit to 600, not on the hit to 599. The latter falsifies the single-first red-ratio
  reading. Health-bar colour alone is not the observable because its comparison differs
  (S-0302 D11d). The yellow ratio's original-game storage precision remains unmeasured
  here for claim 7's stated reason.
- **V14 (no-type omissions, Q-0061).** Common setup, game speed 2, one human player and
  one distant bystander computer player, director and mods off. Named map overrides use
  two existing, visibly distinct animation types, called `Alpha` and `Beta` here, and two
  visibly distinct voxel-debris types. Compare each C15/C16/C18 list written with the corresponding
  two names against the same list with `none`, `<NONE>` and repeated empty comma-separated
  items inserted before and between them. Also compare a previously populated list overridden
  with `none` against one left absent. For the warhead animation list choose damage selecting
  entry 0 under S-0303 F5; observe `Alpha`, not no animation. For splash, damage-fire,
  explosion, destruction, destruction particle-system and debris lists, trigger their effect as S-0303 A18 and S-0312's
  death scenarios describe, recording animation types/counts and the scenario random stream.
  Use 100 events per list form; predict selections and draw counts from each recorded initial
  random state and the owning formula, rather than comparing unmatched seeds. Expected:
  inserted no-type/empty items leave the two-entry choice and its draw rules unchanged.
  An all-no-type override gives an empty list, not the previously populated list. Any selection
  of an empty slot, changed list-selection range, shifted entry-0 animation, or retained prior
  list in the all-no-type run falsifies claim 19. Observe only the test objects and their effects.
  Void the run under the bystander's rule if it interferes.
- **V15 (decimal-prefix maxima, Q-0061).** Common setup at game speed 2 with director and
  mods off; use S-0303 A19's warhead-debris test and a killed test vehicle under S-0312 F1.
  Named map overrides supply two distinct voxel-debris types and enough requested debris
  to distinguish the per-deal maxima through recorded random draws. Run each role ten times per maxima spelling `2,abc,3`,
  `2,,3`, `$10,3`, `10h,3`, a `0x` prefix followed by `10,3`, and `none,3`; also insert a no-type name before
  the first real debris type without changing the maxima. Record debris types and counts,
  and random state for the owning formula's predictions. Expected imported integer lists:
  (2, 0, 3), (2, 3), (0, 3), (10, 3), (0, 3), (0, 3), respectively. The inserted no-type
  name does not shift the maxima paired with real types. Choose safe configurations with
  a positive total and sufficient maxima entries; unused trailing maxima are harmless.
  Counts inconsistent with these per-deal maxima and S-0303 F6 or S-0312 F1 falsify
  claim 20; in particular a positive first-type count for `$10` or the `0x` spelling,
  a per-deal maximum of 16 rather than 10 for `10h`, or
  second-type debris for `2,abc,3` would do so. Observe the test kill/detonation area only
  and apply the bystander's void rule.

- **V16 (factory-kind words, Q-0100).** Tiny purpose-made skirmish map, game speed 2,
  one human player and one distant bystander computer player, director and mods off.
  Named map overrides change the factory-kind entry of one pre-placed factory building.
  remove every other factory of that kind from the observed house. Keep prerequisites,
  power, tech level and credits sufficient. Test each of C19's four producing words on a
  building suitable to release that kind, comparing a control without the override with
  lower-case and mixed-case forms and forms surrounded by spaces. Request an eligible
  item and record whether production begins and completes, using sidebar video and a
  read-only production recorder. Expected: all case/outer-space variants allow the same
  production as the canonical word. Then use a nonsense word, a producing word followed
  by `x`, its object-instance counterpart, the terrain-type word, and each no-factory
  spelling in C19. Expected: none serves any of the four production lines; the earlier
  producing kind is not retained. Empty and absent entries do retain the earlier kind.
  To test the 31-byte cut, append spaces to a producing word through byte 31 and an `x`
  at byte 32: expected producing kind, while an `x` at byte 31 gives no factory. One run
  per form and kind, recording the imported kind on our engine as well. Production in a
  negative variant, lost production in a positive variant, or the byte-32 suffix disabling
  production falsifies claim 21. The bystander's void rule applies to the factory and its
  release area. No claim that unrelated orders are unchanged is tested here.

- **V17 (auto-deploy integer items, Q-0171).** Tiny purpose-made skirmish map,
  game speed 2, one human and one distant bystander computer player, director and
  mods off. Use S-0215 B19's auto-deploy setup with an undeployed neutral-house
  infantry unit meeting claim 31's last-row conditions, no enemy or radiation in
  its test area. The neutral house uses difficulty index 1. Named map overrides
  set row C20's list; one run per spelling below. Record the standing-order start
  tick, its processing ticks and the first start of the deploy sequence, with
  tick-labelled video and a read-only recorder. Predict the first eligible
  processing tick using S-0215 claim 31's strict delay test and its processing
  delays; do not mistake a wait until the next processing tick for a parse change.

  | List text | Retained entries | Delay at index 1, ticks |
  |---|---|---|
  | `5,abc,90` | 5, 0, 90 | 0 |
  | `5,$10,90` | 5, 0, 90 | 0 |
  | `5,10h,90` | 5, 10, 90 | 10 |
  | `5,,40` | 5, 40 | 40 |
  | `5, ,40` | 5, 0, 40 | 0 |
  | `5,4294967336,90` | 5, 40, 90 | 40 |

  Text-limit control: use `5` followed by 130 commas and then `40`, all on one
  physical line. Expected delay: 40 ticks, matching `5,,40`; the second item lies
  beyond byte 127 but below both the physical-line and 511-byte limits. For layer
  retention, use a named loose base-rules copy with this list `5,40,90`, removed
  after the test. Compare absent and empty map entries with `5,abc,90`: the first
  two retain delay 40; the last replaces it with delay 0. Our engine also records
  the imported list and warnings. A deployment tick inconsistent with the indicated
  delay and S-0215's processing schedule falsifies the claim. In particular, delay
  16 for the hexadecimal spellings, delay 90 for the unreadable item, or a missing
  second entry in the long-comma control contradicts the proposed reader. Void a
  run if the bystander interferes in the test area or is defeated.

- **V18 (economy and computer-timer integer items, Q-0234).** Purpose-made
  skirmish test maps, game speed 2, 800 × 600, director and mods off. Named test
  data are the maps' override sections and, for the layering controls, a loose
  base-rules copy removed after each test. One human and one normal-difficulty
  bystander computer player, whose difficulty index is 1 (S-0412 claim 4).
  Keep all unrelated difficulty-indexed lists long enough. Test C21, C22 and C23
  separately, one run per spelling and role below. The second retained entry is
  the quantity selected in each run.

  | Value text | Retained integers | Selected quantity |
  |---|---|---|
  | `5,abc,9` | 5, 0, 9 | 0 |
  | `5,$10,9` | 5, 0, 9 | 0 |
  | `5,10h,9` | 5, 10, 9 | 10 |
  | `5,` followed by `0x` immediately followed by `10,9` | 5, 0, 9 | 0 |
  | `5,,7,9` | 5, 7, 9 | 7 |
  | `5, ,9` | 5, 0, 9 | 0 |
  | `5,4294967303,9` | 5, 7, 9 | 7 |

  - *Part a, virtual purifiers:* use S-0402 H6 part c's computer-owned refinery
    and harvester, with no real purifier, but set named test values to 8 ore
    bales, value per bale 30, country income factor 1.0 and purifier bonus 0.25.
    Observe only the refinery, its ore patch and this unload. A read-only balance
    recorder logs the payments per tick. Expected: main payment 240, and bonus
    payments 0, 600 or 420 for selected quantities 0, 10 or 7, respectively,
    under S-0402 F3. No other computer purchase, sale or income may change the
    balance during the observed unload; void such a run. A nonzero bonus for
    `abc`, `$10` or the `0x` spelling, a bonus for 16 rather than 10 purifiers
    with `10h`, or a bonus for 9 rather than 7 with the empty-item spelling
    falsifies the conversion or placement claim.
  - *Part b, extra credits:* use S-0412 K9 part b's setup with named starting
    credits 2500, unit count 0, bases on and crates off. Record the computer
    house's balance at game start before it spends or receives other credits,
    using the same calibrated trigger observations or a read-only recorder.
    Expected: 2500 plus S-0412 claim 3's extra-credit formula with the selected
    integer as its percentage. Thus zero items give no extra; 10 and 7 give
    the distinct results of that formula, not hexadecimal 16 or the third
    entry 9. Any different balance, after ruling out other transactions,
    falsifies the claimed reader or placement.
  - *Part c, computer timer:* use part b's setup. A read-only recorder records
    the timer's assigned duration at game start, not the later time of an AI
    decision. Expected durations are exactly the selected integers in ticks.
    A duration of 16 for either hexadecimal-looking form, 9 for `abc`, or a
    result other than 7 for the empty-item spelling falsifies claim 23. This
    observes setup only; it requires no reproduction of the original's AI.

  For each role also test `5` followed by 130 commas and then `7,9` on one
  physical line. Expected: selected quantity 7, proving text beyond byte 127
  is read. For layer retention, the named base-rules copy supplies `5,4,9`.
  Compare absent and empty map keys with a present `5,abc,9`: expected selected
  quantities 4, 4 and 0. These controls distinguish keeping the prior whole
  list from replacing it. Our engine additionally records the imported lists
  and warnings, including `,,,` replacing the list with an empty list; do not
  require an original-game effect from a too-short list. For C21/C22 compare
  original-game effects only, not extra advantages on our engine's AI houses.
  C23 has no runtime effect in our engine. Loss of the long-comma second entry,
  retention of the prior 4 after a non-empty replacement, or loss of that 4
  for an absent/empty key falsifies the corresponding claim. Apply the
  bystander's void rule to every test object and observed transaction.

## Open questions

- The text limit of each individual field. Only the common limits are listed here. The per-topic
  rules-import specs should state the limit for each list or text field they define.
- Which byte-triple fields use the colour-definition variant and which use the generic one, beyond
  rows C6 and C7.
- The original's results in claim 6's three uncovered cases (and at 53 bits in claim 13). They are
  deterministic, but no rule for them has been found. For values that are not whole numbers below
  2⁶⁴, the results near a tie mostly match rounding a value slightly below the exact one, so exact
  halves are mostly dropped and exact three quarters mostly rounded up, but not always. The question is moot for our engine as
  long as these cases stay a departure (claim 6); a revision that wants to reproduce them would need
  the rule.
