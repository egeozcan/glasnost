---
id: S-0003
title: INI value forms
topic: rules-import
revision: 3
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-09
measurements: []
requests: []
supersedes: []
---

# S-0003: INI value forms

## Summary

How the original turns an entry's value text into each kind of field (text, booleans, integers,
decimals and percentages, integer tuples, colour and byte triples, lists, type names and named values),
including spellings, truncation, number domains, rounding, overflow and malformed values, and what our
engine does where the original's result is undefined. Claim 17 states the floating-point rounding these
value forms rely on; the rounding mode as a whole belongs to S-0205 (number domains, planned). Reading a
file into sections and entries is S-0001, and duplicate resolution S-0002.

## Terms

S-0001's terms (document, section, entry, key, value, whitespace byte, trim) apply. In addition:

The glossary's *field*, *absent-key default*, *unspecified*, *departure*, *importer warning* and
*round-toward-zero mode* apply as defined there. In particular, wherever this spec calls a result of
the original unspecified, or the original crashes, the claim also states our engine's safe,
deterministic result (a departure, with no quirk switch) and the importer warns.

- **Value in force.** The value a field has when its key is read: its absent-key default, or, in a
  later rules layer, the value an earlier layer gave it (S-0002 claim 7). A key that is missing, or
  written with an empty value (which creates no entry, S-0001 claim 11), leaves the value in force
  unchanged, except for rectangles (claim 10).
- **Component fallback.** For a rectangle field (claim 10), the four numbers that fill the
  components a value does not give. It is fixed per field and is not the field's absent-key
  default, which is 0, 0, 0, 0 for every rectangle field. For the map size it is 1, 1, 50, 50
  (S-0009).
- **Stored value.** The value text as S-0001 keeps it in the entry, before any form is applied. It has
  no leading whitespace and normally no trailing whitespace (see S-0001's trim defect).
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
   sections are limited to 31 bytes (Import mapping).
   - Backing: `measurable` (scenario V1)
   - Confidence: high (the mechanism); medium (the per-field limits, which are listed only where
     known)

### Booleans

2. **Booleans look at the first byte only.** The first byte of the stored value, case-insensitively:
   - `1`, `T` or `Y`: true,
   - `0`, `F` or `N`: false,
   - anything else: the field keeps the value in force.

   So `yes`, `Yes please`, `true` and `t` are true, and `no`, `False` and `Falsehood` are false. `on`,
   `off`, `2` and `-1` are not recognised and leave the value in force.
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
     force**:
     `abc`, `--5`, `- 5` and a `0x` prefix followed by `1F` all give 0. A `0x` prefix without an `h`
     suffix is therefore not hexadecimal.
   - Backing: `measurable` (scenario V3)
   - Confidence: high

4. **Integer overflow wraps.** In every integer form the number formed by the digits read, with the
   sign read, is taken modulo 2³² and read as signed two's complement (Formulas). `2147483648` gives
   −2147483648, `4294967296` gives 0, `99999999999` gives 1215752191, `-2147483649` gives 2147483647, `$100000000` gives 0 and
   `7FFFFFFFFh` gives −1.
   - Backing: `measurable` (scenario V3)
   - Confidence: high

5. **Interaction with the trim defect.** The `h`-suffix test looks at the stored value's last byte.
   If S-0001's trim defect left trailing whitespace on the value, the suffix is missed and the value
   is read as decimal. The importer warns for every value that the trim defect affects (S-0001).
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
   - a value whose rounding goes past the largest finite single becomes infinity (this conversion
     does not depend on the rounding mode of claim 17),
   - `1,5` gives 1, and `-0x5h` gives −0,
   - the words `inf` and `nan` are not numbers (claim 8).

   **Rounding of the conversion.** The conversion does not round to nearest. Outside the three
   cases listed below, it follows this rule. Write the exact value of the decimal as a binary
   fraction and keep its 24 leading significant bits. The result is rounded up (away from zero) by
   one unit in the last kept place when the discarded part is more than one half of that unit.
   Otherwise the discarded bits are dropped. An exact half is therefore dropped, even where
   ties-to-even would round up. When the value is a whole number below 2⁶⁴, however it is spelled (`67108870`, `67108870.0` and `6710887e1` alike), a discarded part of
   exactly three quarters of a unit is dropped too. Examples: `16777219` gives 16777218, `67108870`
   gives 67108864, `74115990` gives 74115984 and `791e7` gives 7909999616, where correct rounding
   gives 16777220, 67108872, 74115992 and 7910000128. (`16777217` gives 16777216 under both rules:
   an exact half is dropped here and rounded to even there.) Outside the three cases, a value whose
   discarded part is not exactly one half or three quarters of a unit comes out correctly rounded.
   Ordinary decimal fractions such as `0.1` or `0.35`, whose binary expansion never ends, are of
   that kind.

   Three cases are not covered by this rule; the original's result in them is unspecified:
   - magnitudes below the smallest normal single (about 1.18 × 10⁻³⁸), which come out off by one or
     two units of the smallest subnormal, or as zero,
   - a discarded part of exactly one half or exactly three quarters of a unit when the value is not
     a whole number below 2⁶⁴. Either neighbour can result, depending on the value: halves are
     mostly dropped, three quarters mostly rounded up,
   - values that are not whole numbers below 2⁶⁴ and lie within about one part in 2⁶⁰ of a value
     whose discarded part is exactly one half or three quarters of a unit.

   Whole numbers below 2⁶⁴ are in none of the three cases: the rule gives the original's result for
   them however close they lie to such a value. For example, 2⁶³ + 2³⁹ + 1, whose discarded part is
   one half plus 2⁻⁴⁰ of a unit, comes out as 2⁶³ + 2⁴⁰.

   **Our engine** applies the rule above in the second and third cases, as the Formulas state it
   exactly (so a three-quarter remainder of a value that is not a whole number below 2⁶⁴ rounds up).
   In the first case it rounds the exact value toward zero to a multiple of the smallest subnormal
   single. The importer warns for every value in one of the three cases, by the test in the Formulas.
   - Backing: `unmeasured` (internal floating-point bits, observable only through later arithmetic)
   - Confidence: high (single precision, and the rounding rule outside the three cases); low (the
     original's results in the three cases)

7. **A `%` anywhere means "divide by 100".** If the stored value contains a `%` byte anywhere, the
   decimal from claim 6 is multiplied by *c*, the double nearest to 0.01. The multiplication is done
   in double precision, **rounded toward zero**, because the original does its floating-point
   arithmetic in that rounding mode (claim 17). `50%` gives 0.5, `0.5%` gives 0.005, `50 %` gives 0.5, `100%%` gives 1.0, and
   `10%` gives 0.09999999999999999 (one unit below 0.1). Without a `%`, the number is taken as it is:
   **`50` in a percentage field means 50.0, i.e. 5000 %**, not 0.5. Percentage fields and plain
   decimal fields are therefore one and the same form.

   Some fields keep the value in single precision. For those the double result is narrowed to single,
   also rounded toward zero: `10%` in such a field is 0.0999999940395… (the single just below 0.1),
   while `0.1` without `%` stays 0.100000001490116… Which fields are single precision belongs to the
   topic specs that define them.
   - Backing: `measurable` (scenario V4, for the factor of 100)
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
   - only the first 63 bytes of the stored value are considered, trimmed,
   - whitespace is allowed before each number, and so after each comma,
   - each comma must follow the preceding number's digits *immediately* (`1, 2, 3` is fine, `1 ,2 ,3`
     stops after the first number),
   - each number is an optional sign and decimal digits, as in claim 3's decimal form, and wraps as in
     claim 4. There are no hexadecimal forms: a `0x` prefix followed by `10` reads as 0 and stops
     the reading at the `x`, and `$10` reads nothing,
   - unlike claim 3, a number with no digits is not 0: reading stops there, and that component and
     all later ones are unread,
   - items beyond the field's count are ignored.

   **In the original, components that could not be read are unspecified** (not the value in force,
   not 0): `1,2` for a triple leaves the third component unspecified, `1 ,2` leaves the second
   unspecified, and `a,2,3` leaves all three unspecified. The same holds for the map header's
   start-position pairs (Import mapping row C12): `5` gives 5 and an unspecified second number. A
   missing key, or one written with an empty value, gives the value in force for the whole pair or
   triple.

   **Our engine** gives every unread component the matching component of the value in force, and
   keeps every component that was read; the importer warns. So `1,2` for a triple whose value in
   force is (7, 8, 9) gives (1, 2, 9), and `a,2,3` gives (7, 8, 9), as a missing key would. This is
   a departure.
   - Backing: `measurable` (scenario V5, for the original's reading)
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
      does not give.

    What this means for the map size, the local area and the preview size is specified in S-0009.
    Quirk `quirks.absent_rect_is_zero` (also named in S-0009).
    - Backing: `measurable` (scenario V5b)
    - Confidence: high

### Colour and byte triples

11. **Byte triples.** Colour definitions and other byte-valued triples use claim 9's syntax with a
    63-byte limit. Each component is the low 8 bits of the integer read, so 300 becomes 44, −1 becomes
    255 and 256 becomes 0. For unread components:
    - in the colour-definition form they are 0 (`1,2` gives 1, 2, 0, and `a,2,3` gives 0, 0, 0),
    - in the generic byte-triple form they are unspecified in the original. **Our engine** gives each
      unread component the matching component of the value in force, as for claim 9, and the
      importer warns (a departure).

    A missing key, or one written with an empty value, gives the value in force.
    - Backing: `measurable` (scenario V6, jointly with S-0103 scenario M-I)
    - Confidence: high (the syntax and the low 8 bits); medium (which fields use which variant, see
      Import mapping)

### Lists

12. **List splitting.** A list value is split at commas.
    - Empty items are skipped, so consecutive commas count as one separator and leading or trailing
      commas are ignored.
    - Items are **not** trimmed. A space after a comma becomes the first byte of the next item.
    - Numeric items tolerate that leading space, because number reading skips leading whitespace.
    - Name items do not: ` Alpha` (with the space) does not match a type called `Alpha`. Depending on
      the field, the item is silently dropped (prerequisite-style lists) or names a new, different
      type (lists that create types on first mention).
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
    - an item containing `%` gives the item's *integer* decimal prefix (claim 3, decimal form) times
      *c*, in double precision rounded toward zero, so `12.5%` gives 0.12, not 0.125,
    - an item without `%` gives a decimal converted to **double** precision (not single, unlike
      claim 6), with claim 6's rounding rule applied at 53 significant bits instead of 24, so `0.125`
      gives 0.125 and `0.1` the double nearest to 0.1. Claim 6's overflow to infinity and its three
      uncovered cases carry over, with the smallest normal and subnormal *double* in place of the
      single ones; our engine's results and the importer warnings are those of claim 6, at 53 bits,
    - an item with no number gives +0.0: without `%` (`abc`, `inf`, `-abc`) the conversion yields +0.0,
      and with `%` the integer prefix is 0, so the item is 0 × *c* = +0.0. Unlike claim 8, this
      result is fully determined,
    - items beyond *k* are ignored,
    - **fewer than *k* items crash the original** while loading. **Our engine** fills the missing
      items, the last ones in class order, with 100 % (1.0 each), and the importer warns, naming the
      warhead and the field; the game starts normally. This is a departure.

    The absent-key default is *k* items of 100 %, each giving 1.0. How these values
    are used belongs to the combat topic.
    - Backing: `measurable` (scenario V8)
    - Confidence: high (conversion, *k*, the class order, and the cause of the crash, which is
      understood); medium (the crash itself, which has not been observed)

### References to types by name

14. **Type names match case-insensitively.** Wherever a value names a type (list sections, weapon,
    warhead and prerequisite references), the name is compared with existing type names using ASCII
    case folding (A–Z only). `alpha` finds the type `Alpha`. The values `none` and `<none>`, in any
    case, mean "no type" and never create one. Section and key names, by contrast, are case-sensitive
    (S-0001 claim 13). What follows from that for a type whose section is spelled in a different case
    is specified in S-0004 (type registration), claim 7.
    - Backing: `measurable` (scenario V9)
    - Confidence: high

15. **Type names longer than 24 bytes** are cut when the type is created, and they then no longer
    match their own full spelling. The consequences and the switch `quirks.long_name_duplicates` are
    specified in S-0004, claim 8. They are noted here only because they follow from how list values
    are read: a type list value may hold up to 31 bytes (claim 1, Import mapping row C2), which is
    longer than a stored type name.
    - Backing: `measurable` (S-0004's scenarios)
    - Confidence: medium

### Named values

16. **Named-value fields.** Fields that take one word from a fixed vocabulary (for example a
    movement zone, a terrain speed class, a building foundation size or a passenger pip colour) read
    the value as text (the limit is per field: 31 or 127 bytes in the cases checked) and compare it
    with the vocabulary ignoring ASCII case. The field's absent-key default is itself a vocabulary
    word, and a missing key gives the value in force. A word that matches nothing does **not** give
    the value in force. It gives a fallback that is fixed per
    field: for some fields a particular vocabulary entry, for others an invalid "no value" result.
    Each topic spec that defines such a field states its vocabulary and its fallback.
    - Backing: `measurable` (scenario V11)
    - Confidence: high (case-insensitive matching, the value in force when absent); medium (the
      per-field fallback, checked for four fields only)

### Floating-point rounding

17. **Round toward zero, 53-bit significand, from before the first data file is read.** The
    original runs in the *round-toward-zero mode* (glossary): from program start, before the first
    data file is read, its own floating-point arithmetic uses a 53-bit significand and rounds every
    result toward zero, including every store of a result to single or double precision. Nothing in
    the original's program selects another mode during a session. For this spec, that means:
    - claim 7's multiplication by *c*, and the narrowing to single in single-precision fields, round
      toward zero, as do the multiplications by *c* in claim 13,
    - the conversion of decimal text to a number (claims 6 and 13) is not done in this arithmetic and
      does not depend on the mode.

    The mode's full scope (conversions to integer, the runtime library's mathematical functions, and
    which results of other specs it governs) and its single switch `quirks.chop_rounding` belong to
    S-0205 (number domains, planned). This spec defines no switch for it.
    - Backing: `measurable` (scenario V12)
    - Confidence: high. The mode is set before any data is read, and no other change of it is
      known in the original's program. A display or sound library outside the program could in
      principle change it; our engine has no such library.

## Formulas

Notation. For a real number *z*: RN₅₃(*z*) is *z* rounded to the nearest double; RZ₅₃(*z*) and
RZ₂₄(*z*) are *z* rounded toward zero to a double and to a single. RZ is the original's rounding
for the floating-point operations of this spec (claim 17; the mode as a whole belongs to S-0205,
number domains, planned). *a* mod *n* is always in
0 ≤ … < *n*.

Integer forms (claims 3 and 4). Let *D* be the non-negative integer that the digits read (at least
one) denote in base *B* (10 or 16), and *s* the sign (−1 if a `-` was read, else +1). All arithmetic
is exact integer arithmetic; the result is a signed 32-bit integer.

> *u* = *D* mod 2³²
>
> *v* = (*s* · *u*) mod 2³²
>
> result = *v* if *v* < 2³¹, and *v* − 2³² if *v* ≥ 2³¹

Decimals (claim 6). This is the original's rule outside claim 6's three uncovered cases, and our
engine's rule for every magnitude from the smallest normal single upwards. Let *x* be the exact
rational magnitude of the decimal prefix, and *P* = 24 the number of kept bits. The result carries the
prefix's sign; *x* = 0 gives a zero of that sign. For *x* > 0:

> *x* = *m* × 2^*e*, with *e* an integer and 2^(*P*−1) ≤ *m* < 2^*P* (*m* real); *t* = ⌊*m*⌋,
> *r* = *m* − *t* (the discarded part, 0 ≤ *r* < 1)
>
> *W* = true if *x* is a whole number and *x* < 2⁶⁴, false otherwise
>
> *f* = *t* × 2^*e* if *r* ≤ 1/2, or if *r* = 3/4 and *W*; *f* = (*t* + 1) × 2^*e* otherwise
>
> magnitude of the result = *f* if *f* does not exceed the largest finite value of the format,
> and infinity otherwise

Below the smallest normal value of the format our engine instead gives *x* rounded toward zero to
a multiple of the smallest subnormal value. The result, *y*, is a single and so exactly a double.

Importer warning test (claims 6 and 13). With *n* the smallest normal value of the format, the
importer warns for a decimal exactly when

> (0 < *x* < *n*) or (*W* is false and (|*r* − 1/2| ≤ 2^(*P*−60) or |*r* − 3/4| ≤ 2^(*P*−60)))

The first part is claim 6's first case. The second part covers its second and third cases with a
margin; it never warns for a whole number below 2⁶⁴, which is in none of the three cases. The test
never changes the result.

Percentages (claim 7), with *y* the signed result of claim 6. A value with no number is claim 8's
case: in our engine it leaves the value in force, and neither line below applies.

> *c* = RN₅₃(1/100)
>
> without `%`: result = *y*
>
> with `%`: result = RZ₅₃(*y* × *c*), a double; in a single-precision field, RZ₂₄(RZ₅₃(*y* × *c*))

Fixed-count percentage list item (claim 13), with *i* the integer from the item's decimal prefix
(claim 3, decimal form):

> with `%`: RZ₅₃(*i* × *c*), where *i* = 0 if the item has no digits
>
> without `%`: claim 6's rule with *P* = 53, giving a double (largest finite, smallest normal and
> smallest subnormal values of the double format); +0.0 if the item has no number
>
> missing items (fewer than *k*), in our engine: 1.0 each

Byte triple component (claim 11): *b* = *v* mod 256, so 0 ≤ *b* ≤ 255, with *v* the signed 32-bit
integer from claim 9.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| percent_factor | RN₅₃(1/100), the double nearest to 0.01 | ratio | *c* in the formulas. Applied when `%` is present |
| tuple_text_limit | 63 | bytes | Integer pairs, triples and quadruples, colour and byte triples |
| common_text_limit | 127 | bytes | Most text and list fields |
| type_list_text_limit | 31 | bytes | Values in type list sections |
| type_name_max | 24 | bytes | Longest type name kept when a type is created (claim 15) |
| int_bits | 32 | bits | Integer results, two's complement, wrap modulo 2³² |
| armour_class_count | 11 | armour classes | *k* of the armour-modifier list (claim 13); the class order is in Import mapping row C5 |

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
- Round-toward-zero arithmetic (claims 7, 13 and 17): this spec defines no switch. The mode and its
  single switch `quirks.chop_rounding` (default: the original behaviour) belong to S-0205 (number
  domains, planned).
- Long type names: see S-0004 (`quirks.long_name_duplicates`). An importer should warn either way.
- Departures, with no quirk switch. The original's unspecified results (claim 6's three uncovered
  cases, claim 8, claim 9's unread components, claim 11's generic form, and the same cases at 53
  bits in claim 13) cannot be reproduced, and the original crashes on a too-short fixed-count list
  (claim 13). Each claim states our engine's safe, deterministic result: the rule of the Formulas
  for claim 6's cases, the value in force for claim 8, the value in force's components for claims
  9 and 11, and 100 % items for claim 13. The importer warns for every one of them, and the game
  still starts.
- A percentage written without `%` (claim 7) is the most likely real-world mistake. Whether the
  importer warns for it is a question for the project lead (below).

## Import mapping

| Row | Data | Form |
|---|---|---|
| C1 | `Cost` of an infantry, vehicle, aircraft or building type | Integer (claims 3 and 4). Observable on the sidebar |
| C2 | Type list sections such as `[InfantryTypes]`, `[BuildingTypes]`, `[Warheads]` | Text of at most 31 bytes per value, each naming a type (claims 1, 14 and 15) |
| C3 | `Prerequisite` of a buildable type | List of building names and the generic prerequisite words (`POWER`, `FACTORY`, `BARRACKS`, `RADAR`, `TECH`, `PROC`), text limit 127 bytes. Items that match nothing are dropped (claim 12) |
| C4 | `Owner`, `RequiredHouses`, `ForbiddenHouses`, `SecretHouses` | List of country names, text limit 127 bytes (claim 12) |
| C5 | `Verses` of a warhead | Fixed-count percentage list (claim 13): 11 items, text limit 127 bytes. Item *j* belongs to the *j*-th armour class in this fixed order, by the word a type's `Armor` takes: `none`, `flak`, `plate`, `light`, `medium`, `heavy`, `wood`, `steel`, `concrete`, `special_1`, `special_2` |
| C6 | Entries of `[Colors]` | Colour-definition byte triples (claim 11, unread components 0) |
| C7 | Other byte triples: colour fields of the rules file's `[AudioVisual]` and `[Radiation]` sections, the entries of the rules file's colour-adjustment section, and colour fields of some types (for example `LaserColor` of a particle system) | Generic byte triple (claim 11: unread components unspecified in the original; in our engine, the value in force's components) |
| C8 | `PrimaryFireFLH`, `SecondaryFireFLH` and the other fire-position triples of a type (art file) | Integer triple (claim 9) |
| C9 | Percentage-style rules fields such as `ConditionYellow`, `ConditionRed` (in `[AudioVisual]`) | Decimal with optional `%` (claims 6 to 8) |
| C10 | Boolean fields such as `Selectable` | Boolean (claim 2) |
| C11 | `Speed` and `Strength` of a type | Integer (claim 3) |
| C12 | `Waypoint1` … `WaypointN` in a map's `[Header]`, *N* = the header's start-position count (S-0009) | Integer pair (claim 9). An unread number is unspecified in the original; in our engine it is the matching number of the value in force |
| C13 | Rectangles in a map, such as `LocalSize` and `Size` in `[Map]` and the preview's `Size` | Integer quadruple (claim 10). Absent or empty: 0, 0, 0, 0. The map consequences and each field's component fallback are in S-0009 |
| C14 | `MovementZone`, `SpeedType`, `Pip` of a type, `Foundation` of a building (art file) | Named value (claim 16) |

## Measurement scenarios

Common setup: a tiny purpose-made multiplayer map whose INI part carries a rules-override section for
one buildable infantry type of the player's side (the test type). The glossary's *measurement
setup* applies: stock data, no mods and no engine add-on active, any game speed. A presentation-only
display compatibility layer may be used and is named generically in the measurement record. V6 uses
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
  by: `off` making it unselectable.
- **V3 (integers).** Set the cost (row C1) in turn to `$64`, `64h`, `0x` followed by `64`, `abc`,
  `4294967396`, `100junk`, `$0x` and `0xh`. Expected sidebar costs: 100, 100, 0, 0, 100, 100, then
  for both of the last two the cost the type has without the override (the value in force).
  Falsified by: the cost without the override appearing for the `0x` spelling or for `abc`, or 0
  appearing for `$0x` or `0xh`.
- **V3b (`h` suffix and the trim defect).** Set the cost (row C1) with the value text `····64h··`
  (· is one space), as in S-0001 scenario L7d. S-0001's trim defect then leaves one trailing space on
  the stored value. Expected sidebar cost: 64 (the value is read as decimal). Falsified by: 100.
- **V4 (percentages).** In the map's rules part, set the red health threshold (row C9) to `10%`, and
  the yellow threshold in turn to `40%`, `0.4` and `40`. Damage a unit to about 50 % and then about
  30 % of its strength. Expected: green, then yellow, for the first two settings. With `40` the
  threshold is 4000 %, so the bar is yellow even at full health. Then set the yellow threshold to
  `abc`. Expected: the bar does not turn yellow at about 30 % health (a threshold of about 0).
  Falsified by: yellow at about 30 %. That would mean the threshold is not tiny; it cannot tell the
  value in force from a large unspecified value (claim 8), so this step tests only the "often tiny"
  part of claim 8, for this field.
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
  up as in S-0103 scenario M-I, whose definitions at the positions of slots 0 to 3 are, in turn,
  `300,255,255`, `44,255,255`, `40,255` and `a,255,255`. Use S-0103 scenario M-C's map and procedure: one game per slot,
  with a screenshot of the player's infantry on flat ground. Observable: the infantry's remap pixels
  (the palette indices of the sprite's remap range), compared between slots and with S-0103 M-C's
  ramp computation. Expected:
  - slots 0 and 1 show the same yellow ramp, that of (44, 255, 255): 300 keeps its low 8 bits,
  - slot 2 shows an all-black remap range, the ramp of (40, 255, 0): the missing third component is 0,
  - slot 3 shows an all-black remap range too, the ramp of (0, 0, 0): reading stops at the
    unreadable first component, so the two 255s are never read.

  Falsified by: slot 0 differing from slot 1 (for example the red of a value clamped to 255); slot 2
  or slot 3 showing any non-black remap pixel: a missing component that is not 0, or, for slot 3,
  the red ramp of (0, 255, 255) tabulated in S-0103 scenario M-C, which reading `a` as 0 and
  carrying on would give. S-0103 scenario M-I covers further spellings of the same form (negative numbers, a space
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
  allows it.
- **V12 (rounding mode).** Use S-0104 scenario L5 (neutral test lighting, an infantry pixel on flat
  ground). Pick a pixel whose palette colour has a **red** channel that is a multiple of 8, for
  example 248. Expected: the red channel shows the 16-bit step of one below that value (247's
  step), which needs rounding toward zero in the conversion of the 1000 ‰ tint to its factor
  (S-0104, claim 7 and its Formulas). Falsified by: the step of the palette value itself (248's),
  which round-to-nearest arithmetic gives. S-0108 scenario B1 (a build-up delay one tick shorter
  than round-to-nearest gives) corroborates the claim but cannot falsify it, since how that spec's
  time value is first converted is not established.

## Open questions

- The text limit of each individual field. Only the common limits are listed here. The per-topic
  rules-import specs should state the limit for each list or text field they define.
- Which byte-triple fields use the colour-definition variant and which use the generic one, beyond
  rows C6 and C7.
- (Raised in scrub review; to be settled in S-0205, number domains, planned.) Whether a result of
  the original's arithmetic has the exponent range of a double as soon as it is computed, or only
  when it is stored to single or double precision. The Formulas' RZ₅₃ assumes the double range. The
  two readings differ only for intermediate results outside the normal range of a double, that is
  overflow and subnormals. No result of this spec is affected.

## Questions for the project lead

- Should the importer warn when a field documented as a percentage gets a value above 1 without
  `%` (claim 7)? It would help mod authors. The imported result stays 100 times the intended one
  either way.
