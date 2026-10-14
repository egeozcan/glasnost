---
id: S-0205
title: Number domains (integers, floating point, the round-toward-zero mode, conversions)
topic: simulation
revision: 6
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-14
measurements: []
requests: []
supersedes: []
---

# S-0205: Number domains (integers, floating point, the round-toward-zero mode, conversions)

## Summary

The number domains the original computes in, and the rules that hold for every computation unless
a topic spec says otherwise: integer width, overflow and division; floating-point precision; the
program-wide round-toward-zero mode and its scope; how floating-point values become integers; and
the conventions the other specs use to write these rules. This spec owns the single quirk switch
`quirks.chop_rounding` for the rounding mode, and lists the claims of other specs that depend on
it. How data text becomes numbers is S-0003; which domain each game quantity uses belongs to the
spec of that quantity. This contract covers skirmish; network games and campaigns are out of scope.

## Terms

Shared terms used here are defined in the glossary: *round-toward-zero mode*, *departure*,
*importer warning*, *quirk switch*, *field*, *per-mille*, *lepton*, *facing*, *session*. S-0003's *single* and *double* (IEEE 754 binary floating point with a
24-bit or 53-bit significand) apply. In addition:

- **Integer.** In this spec and in every spec that does not say otherwise, a signed 32-bit
  two's-complement integer, −2147483648 to 2147483647.
- **Wrap.** Reduction of an exact integer result modulo 2³² into the integer range.
- **RZ, RN.** RZ₅₃(*z*) and RZ₂₄(*z*) are the real number *z* rounded toward zero to a double and to
  a single; RN₅₃ and RN₂₄ round to nearest, ties to even. These are S-0003's notations.
- **Arithmetic result.** The result of one floating-point addition, subtraction, multiplication or
  division. A *narrowing* is the storing of a result in single precision.
- **Conversion.** Turning a floating-point value into an integer.
- **Common form, 32-bit form.** The two forms a conversion takes in the original (claim 7). Nearly
  every conversion uses the *common form*; a few conversions in one presentation-only effect use
  the *32-bit form*. Both truncate toward zero and give the same result in the integer range; they
  differ only outside it (claim 7, Formulas).
- **trunc.** Truncation toward zero of a real number to a whole number: trunc(2.7) = 2,
  trunc(−2.7) = −2.

## Behaviour

### Integers

1. **Integer width and overflow.** Integer quantities of game state (positions in leptons, health,
   credits, timers in ticks, counts) are integers in the sense of the Terms unless their spec gives
   another width. An integer result outside the range wraps; the original does not saturate or
   clamp unless a topic spec states a clamp. S-0003's integer forms (claim 3) are one visible case.
   A few calculations form a 64-bit integer product or quotient as an intermediate result; the
   topic spec of each says so.
   - Backing: `unmeasured` (a property of every integer operation; each topic spec whose result can
     overflow states and measures its own case)
   - Confidence: high

2. **Integer division and remainder.** An integer quotient is the exact quotient truncated toward
   zero. The remainder has the sign of the dividend and a magnitude smaller than the divisor's, so
   that dividend = quotient × divisor + remainder. Consequence: −7 ÷ 2 = −3 with remainder −1, and
   positions from −255 to 255 leptons all lie in cell 0 of an axis (S-0101, F7). Some
   calculations of the original instead halve a signed integer rounding toward −∞ (−7 halved is
   −4); the specs that use such a step say so and write it ⌊*a* / 2⌋ (Formulas, Notation). The
   one quotient that does not fit the integer range, −2147483648 ÷ −1, is covered by claim 3.
   - Backing: `measurable` for signed quotient truncation (scenario N3, using S-0101 M1b).
     `unmeasured` for the general remainder rule, negative cell positions and signed floor-halving:
     N3 observes a quotient, not a remainder, and provides neither off-map positions nor a
     calculation isolating floor-halving. Topic specs retain their own measurements of those cases.
   - Confidence: high

3. **Integer division by zero, and the one overflowing quotient.** An integer division whose
   divisor is 0 makes the original stop with a program error, and so does the division of
   −2147483648 by −1 (quotient or remainder). **Departure:** for a divisor of 0 our engine gives
   the quotient 0 and a remainder equal to the dividend; for −2147483648 ÷ −1 it gives the wrapped
   quotient −2147483648 and the remainder 0. The importer warns for any data that can make a
   divisor 0 where the topic spec names such a field. These results are the general rule only: a
   topic spec that gives its own result for a zero divisor in one of its calculations overrides
   them there. For example, S-0104 (claim 11) has a light source of radius 0 reach no cell, so the
   falloff quotient with divisor 0 is never formed.
   - Backing: `unmeasured` (the results of the departure do not exist in the original, which stops,
     so there is no original value to compare with; that a given calculation stops the original is
     observable only through a field that can make its divisor 0, and the topic spec that names
     such a field gives its own scenario)
   - Confidence: high (division by zero); medium (that the overflowing quotient is reachable at
     all; no calculation is known to produce it)

### Floating point

4. **Precision.** Floating-point quantities are doubles. Every arithmetic result is formed with a
   53-bit significand, also when its operands came from single-precision values: a single operand is
   widened exactly to a double first. Some fields and intermediate values are held in single
   precision; the topic specs say which (S-0003, claim 7). Decimals read from data are single
   precision for most fields (S-0003, claim 6), widened to doubles, and double precision for the
   items of the fixed-count percentage lists of S-0003, claim 13 (the armour-modifier list of a
   warhead). Widening a single to a double never changes its value, since every
   single is also a double. Comparisons of floating-point values are exact: two values compare by
   their exact real values, with no tolerance and no rounding.
   - Backing: `measurable` for the 53-bit significand (scenario N4: a 24-bit or 64-bit significand
     gives a different result there); the exact widening is a property of the two formats and needs
     no measurement; exact comparison is `unmeasured` (no scenario here isolates a game-state comparison
     between controllable unequal values close enough to distinguish exact comparison from a
     tolerance-based or rounded comparison)
   - Confidence: high

5. **The round-toward-zero mode, and its scope.** From before the first data file is read (S-0003,
   claim 17) until the program ends, every arithmetic result and every narrowing in the original,
   outside the power function of the bullet below, is rounded toward zero: RZ₅₃ for a result kept
   as a double, RZ₂₄ for a narrowing. After each
   conversion of the common form (claim 7), the round-toward-zero mode with a 53-bit significand,
   the setting selected at start-up, is in force whatever the mode was before it. So even if
   something outside the program changed the mode, the change would last only until the next such
   conversion; conversions happen in nearly every tick. Consequences:
   - A narrowing gives the exact result truncated to single precision: rounding toward zero to 53
     bits and then to 24 bits gives the same value as one rounding toward zero to 24 bits.
   - The only library routine of the program that computes game state in another mode is the
     power function (with its error handling). Its own arithmetic rounds to nearest with a 53-bit
     significand; every arithmetic result outside it, including any that uses its value, is rounded
     toward zero as above. Its
     result is not formed by multiplication: it comes from the processor's logarithm and
     exponential instructions. One game-state calculation uses it, the launch angle of projectiles
     that are affected by gravity (S-0306 claim 6 a and F4), which takes the tabulated square root
     (claim 6) of

     > *s* = RZ₅₃(*P*(*d*<sub>x</sub>) + *P*(*d*<sub>y</sub>))

     where *d*<sub>x</sub> and *d*<sub>y</sub> are integers (Terms), in leptons: the two horizontal
     components of the distance from the firing point to the aimed point of the shot, each in
     −2147483647 … 2147483647 (S-0306 names the fire point and impact point). They are
     whole numbers, converted to doubles exactly before the power function sees them, and
     *P*(*c*) is the power function's square of the integer *c*, a double. RZ₅₃ (Terms) is the rounding of the
     sum; with `quirks.chop_rounding` off it is RN₅₃ instead (Formulas, Floating-point result).
     - For |*c*| < 2²⁶, *c*² is a whole number below 2⁵², hence itself a double, and *P*(*c*) = *c*²
       whatever the program's mode, provided those instructions are accurate to within one unit in
       the last place of their 64-bit significand; *P*(−*c*) = *P*(*c*), and *P*(0) = 0
       exactly. That bound is the processor's documented accuracy and has not been checked on
       hardware. A processor outside it could give a *P*(*c*) one unit in the last place below *c*², and
       the rounding of *s* toward zero would keep that error.
     - For |*c*| ≥ 2²⁶, *c*² is in general not a double, so an exactly represented square is no longer guaranteed, and the original's
       *P*(*c*) is whatever its instructions return for that processor (not measured). No map known has a planar distance between two points near 2²⁶
       leptons (262144 cells), so this case is not expected in play.
     - **Departure** (host-dependent, no switch): our engine uses *P*(*c*) = RN₅₃(*c*²), the exact
       square rounded to nearest (an exact value for |*c*| < 2²⁶), for every integer *c*, in both
       settings of `quirks.chop_rounding`; the sum *s* is then rounded as above. The result
       differs from the original only on a processor whose instructions give another value for
       |*c*| < 2²⁶, and for |*c*| ≥ 2²⁶ wherever the original's value differs from RN₅₃(*c*²), which
       depends on the processor.
   - The random-map generator computes natural logarithms with the processor's logarithm
     instruction, in the program's own mode. Its rules belong to a future spec.

   The mode is controlled by the switch `quirks.chop_rounding`. Its known effects on other specs are
   listed with the switch under Edge cases.
   - Backing: `measurable` (scenario N1, and scenario N4, where round-to-nearest arithmetic also
     gives a different result); the power-function bullet is
     `unmeasured` (only a square that the bound does not cover would show a difference, and none is
     known; a distance of 2²⁶ leptons or more cannot be set up on a map); the renewal at every conversion is `unmeasured` (it shows only if something outside
     the program changed the mode, and nothing is known to)
   - Confidence: high (the mode for all arithmetic of the program, and its renewal at every
     conversion); medium (that no outside component changes it between two conversions in a way
     that reaches game state; that the power function is the only library routine in another
     mode that game state reaches); low (that the power function gives exact squares on every
     processor)

6. **No library functions for roots and angles.** The original computes no square root, sine,
   cosine, tangent or angle of game state with the processor's or the library's functions. It
   reads them from its own tables of values, and any arithmetic around a table read follows claims
   4 and 5. The tables given so far are:
   - the tabulated square root, which S-0204 (claim 5, Formulas F1) gives for distances between
     objects; S-0009 (claim 19) and S-0105 (F1) use the same function,
   - the sine table of S-0105 (F1, table trigonometry),
   - the arctangent table of S-0204 (claim 8, Formulas F3), for the direction from one point to
     another, and
   - the arcsine table of S-0106 (F3), for the interpolation of tilts.
   Our engine must use those tables, not a mathematical library, wherever a spec gives one.
   The one use of a library function in game state is the power function of claim 5, used only
   for squares.
   - Backing: `measurable` for the tabulated square root and arctangent (scenario N6).
     `unmeasured` here for table sines and arcsines: N6 has no isolated test of those table values
     against mathematical functions; their topic specs own any such tests. The absence of library
     functions from every other game-state calculation is `unmeasured` (it is a statement about
     every calculation, which shows only through each quantity's own spec)
   - Confidence: high

7. **Conversions to integers.** A conversion truncates toward zero, in every rounding mode:
   2.9 gives 2 and −2.9 gives −2. Values outside the integer range are handled as follows by the
   form that nearly every conversion uses, the *common form*:
   - when |*x*| < 2⁶³, the result is trunc(*x*) wrapped (Terms), the integer congruent to
     trunc(*x*) modulo 2³²: 2147483648.0 gives −2147483648, 3000000000.0 gives −1294967296,
     −3000000000.0 gives 1294967296, 4294967301.0 gives 5,
   - when |*x*| ≥ 2⁶³, or *x* is infinite or not a number, the result is 0.

   A few conversions in one full-screen picture-distortion effect of a superweapon (presentation
   only) use a 32-bit form instead: in range they give the same result, and out of range
   (trunc(*x*) below −2³¹ or at least 2³¹, infinite or not a number) they give −2147483648. The effect clamps those
   results to the picture's bounds at once, and the rendering spec that covers it will say so.
   Conversions are not affected by `quirks.chop_rounding`.
   - Backing: `measurable` for positive in-range truncation (scenario N7), a positive
     out-of-range wrap (N5) and a positive huge value giving zero (N2). `unmeasured` here for
     negative inputs, infinities, not-a-number and the exact 2⁶³ boundary: these scenarios do not
     isolate those inputs. The 32-bit form is `unmeasured` (it is presentation only, its inputs
     are picture coordinates that never come near 2³¹, and the effect clamps its results at once)
   - Confidence: high (truncation, wrap, the zero result, and that only that effect uses the
     32-bit form)

8. **Exponent range of intermediate results.** In the original, a result that is used again at once
   may keep a wider exponent range than a double (its significand is still 53 bits, rounded as in
   claim 5), while a result that is stored has exactly the range of a double or single. Which
   results are stored depends on each calculation of the original. This differs from strict
   double arithmetic only when an intermediate result exceeds the largest double or falls below
   the smallest normal double; no game-state calculation is known to come near either with
   meaningful data. **Documented departure** (approved by the project lead; no switch, and no
   importer warning): our engine gives every intermediate result the range of a double. This closes
   S-0003's open question on the exponent range.
   - Backing: `unmeasured` (only extreme data could show it, and no data path to such an
     intermediate result is known)
   - Confidence: medium

9. **Data fractions and percentages.** A percentage or ratio read from data as a decimal is held
   as a double fraction, 1.0 meaning 100 % (S-0003, claim 7); its value is a single or a double
   as the consequences below say. The original has no general fixed-point
   number type: where a quantity is a scaled integer (leptons, facing units, per-mille, a factor in
   units of 1/65536), the spec that defines it gives the scale. A conversion between a fraction and
   a scaled integer is a floating-point multiplication followed by a conversion (claim 7), unless the
   topic spec gives another rule. Consequences:
   - A decimal written without a percent sign is a single (S-0003, claim 6), with at most 24
     significant bits; the exception is an item of a fixed-count percentage list (S-0003, claim 13),
     which is a double, so the exactness below is for singles only. Its product with a whole number whose odd part has at most 29 significant
     bits, such as 900 ticks per game minute or 256 leptons per cell, is exact in double precision.
     Such a conversion is the exact product of the single value, truncated toward zero, and does
     not depend on the rounding mode. For example, a field written as 0.3 is the single just above
     0.3, and its product with 900 is 270.0000107…, which converts to 270 in both settings of
     `quirks.chop_rounding`.
   - A short decimal whose single lies below it still gives one less than the decimal times the
     factor: 0.06 × 900 gives 53 (S-0215, F1). This comes from the single-precision reading, not
     from the rounding mode.
   - A decimal written with a percent sign is a double-precision product (S-0003, claim 7), and its
     product with the factor is rounded: `30%` times 900 gives 269 with `quirks.chop_rounding` on and
     270 with it off.
   - Backing: `measurable` (the multiplication and truncating conversion, and the exact product of a
     single: scenario N7; the fraction 1.0 = 100 %: S-0003 scenario V4); that the product is exact is
     a property of the two formats and needs no measurement; the absence of a general fixed-point
     type is `unmeasured` (it is a statement about how quantities are represented, which shows only
     through each quantity's own spec)
   - Confidence: high (the multiplication, the conversion and the exact products); medium (the
     absence of a general fixed-point type is a summary over the specs written so far)

## Formulas

**Notation used by the specs.** These are the conventions a spec may use without defining them
again. A spec that defines a notation differently keeps its own definition within that spec: S-0104,
for example, writes ⌊ ⌋ for truncation toward zero.

| Notation | Meaning |
|---|---|
| *a* ÷ *b*, *a* quo *b*, for integers *a*, *b* | Integer quotient truncated toward zero (claim 2) |
| *a* rem *b* | Remainder with the sign of *a* (claim 2) |
| *a* mod *b* | Remainder in 0 … *b* − 1 for *b* > 0 |
| ⌊*z*⌋ | Floor (toward −∞), unless the spec defines it otherwise; for *z* ≥ 0 it equals trunc |
| ⌊*a* / 2⌋, for an integer *a* | Halving toward −∞ (claim 2): −7 gives −4, 7 gives 3; for *a* ≥ 0 it equals *a* ÷ 2 |
| trunc(*z*) | Truncation toward zero |
| RZ₅₃, RZ₂₄, RN₅₃, RN₂₄ | Rounding of a real number to a double or single (Terms) |

**Floating-point result** (claims 4, 5). Outside claim 5's power-function exception, for operands
*p* and *q* (doubles) and an operation ∘ of +, −, ×, ÷:

> result = RZ₅₃(*p* ∘ *q*) with `quirks.chop_rounding` on (the original), RN₅₃(*p* ∘ *q*) with it off.
>
> A narrowing of a double *y*: RZ₂₄(*y*) on, RN₂₄(*y*) off. With the switch on,
> RZ₂₄(RZ₅₃(*z*)) = RZ₂₄(*z*) for every real *z* in the normal range.

**Conversion** (claim 7). For a double *x*:

> *t* = trunc(*x*), a whole number.
>
> Common form. If |*x*| < 2⁶³: result = *t* wrapped (Terms), i.e. the integer congruent to *t* modulo 2³².
>
> Otherwise (including infinities and not-a-number): result = 0.
>
> 32-bit form: result = *t* if −2³¹ ≤ *t* < 2³¹, otherwise (including infinities and not-a-number)
> −2147483648.

**Adding one half before converting.** Some calculations round a double *x* by adding one half and
converting. With the conversion above, the result is trunc(RZ₅₃(*x* + 1/2)) (`quirks.chop_rounding`
on, the original), provided the rounded sum is finite and its truncation fits the integer
range. Otherwise claim 7's common-form conversion applies to the rounded sum, including wrap
and the zero result. With the switch off the in-range result is trunc(RN₅₃(*x* + 1/2)), the round-to-nearest
arithmetic of the Floating-point result above. With the switch on, for *x* ≥ 0 this
rounds halves up, except where adding one half rounds down below a whole number: *x* =
0.49999999999999994 (the double just below 1/2) gives 0, while round-to-nearest arithmetic gives 1.
For negative *x* it is not "round half away from zero": −1.2 gives 0 and −1.7 gives −1. A spec
that uses this rounding cites this paragraph. Backing: `unmeasured` here (N1–N7 isolate no
calculation that adds one half); confidence: high. Each topic spec using it owns its measurement.

**Integer division by zero and the overflowing quotient** (claim 3, departure): *a* ÷ 0 = 0 and
*a* rem 0 = *a*; −2147483648 ÷ −1 = −2147483648 and −2147483648 rem −1 = 0. Where a topic spec
gives its own result for a zero divisor (S-0104, claim 11, radius 0), that result applies instead.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Integer range | −2147483648 … 2147483647 | integer (dimensionless) | Claim 1 |
| Significand of arithmetic | 53 | bits | Claims 4, 5 |
| Significand of single values | 24 | bits | Claims 4, 5 |
| Conversion zero threshold | 2⁶³ | magnitude of a double (dimensionless) | Claim 7: magnitudes at or above it convert to 0 |
| 32-bit-form out-of-range result | −2147483648 | integer (dimensionless) | Claim 7 |

## State and ordering

| State | Event | Result |
|---|---|---|
| Before start-up | Program start, before the first data file is read | Round-toward-zero mode with a 53-bit significand, in force for the session (S-0003, claim 17) |
| Round-toward-zero mode, or a mode changed by something outside the program | A conversion of the common form (claim 7) | Round-toward-zero mode with a 53-bit significand in force (no change if nothing altered it) |
| Round-toward-zero mode | The library power function computes a value (claim 5) | Its own arithmetic rounds to nearest with a 53-bit significand; every arithmetic result outside it, including any that uses its value, is rounded toward zero (the mode is unchanged afterwards) |

Observable ordering: none; the mode applies to every calculation alike.

## Edge cases and quirks

- **`quirks.chop_rounding`** (default: on, the original). On: outside claim 5's power-function exception, every arithmetic result is rounded
  toward zero to 53 bits and every narrowing toward zero to 24 bits (claim 5). Off ("fixed"): every
  arithmetic result outside that exception is rounded to the nearest double (ties to even) and every narrowing to the
  nearest single, from that double. In both settings: arithmetic uses a 53-bit significand,
  conversions truncate toward zero (claim 7), decimal text is read as S-0003 says, and table
  values are used as stored. This is the only switch for the mode; the specs in the table below
  define none of their own. The original's power function retains claim 5's nearest rounding
  in both settings; our engine uses the deterministic square departure stated there.

  **Effects of the switch.** The table lists the claims of the other specs written so far whose
  results depend on it, and what changes when `quirks.chop_rounding` is off. Claims that state they
  do not depend on it, or depend on it only for some data, are listed too. This is a dependency
  index: each row inherits the backing and confidence of the cited owning claim or formula,
  not a claim that N1–N7 measure every listed effect. "Percent sign" means a
  decimal written with a percent sign (S-0003, claim 7); a decimal without one is a single (except for the items of fixed-count lists, S-0003 claim 13,
  which are doubles), and its product with 900 or 256 is exact (claim 9).

  | Spec and claim | Depends on the mode? | With the switch off |
  |---|---|---|
  | S-0003, claims 7, 13 and 17 (percent factor, narrowing of percent fields; the percent items of fixed-count lists) | Yes | `10%` gives the double nearest to 0.1, and its single narrowing is the single nearest to 0.1 |
  | S-0004, Formulas (superweapon recharge-time sort key, *m* × 900) | Only for a value with a percent sign | The product rounds to nearest: `30%` gives 270 ticks instead of 269. A value without a percent sign gives the same ticks in both settings (a field written as 0.3 gives 270) |
  | S-0006, claim 5 (light values truncated when re-read) | No (stated there) | — |
  | S-0009, distance sum (table square root) | No (stated there) | — |
  | S-0010, claim 19 (tile animation position, by S-0101 F6) | Yes | As S-0101, F6 |
  | S-0101, F2 (height lift) | No, at every height a map can produce (stated there) | — |
  | S-0101, F6 (ground position of a world pixel) | Yes | About 0.14 % of lepton results move by one lepton; no cell changes (as S-0101 states) |
  | S-0103, claim 6 (ramp products) | No (exact, stated there) | — |
  | S-0104, claim 1 (map value storage), claim 7 (shade table, neutral light darkens by one), tint normalisation | Yes | 1000 ‰ maps to exactly 65536, neutral light reproduces the palette, tint quotients are exact (as S-0104 states) |
  | S-0105, F0 (every floating-point step of voxel rendering) | Yes | Every step rounds to nearest; conversions still truncate (as S-0105 states) |
  | S-0106, claims 2 and 3, F1 (surface height on slopes) | No (exact, stated there) | — |
  | S-0106, F2 (tilt transforms), F3 (interpolated tilt) | Yes (by S-0105, F0) | Every floating-point step and every single-precision narrowing rounds to nearest; conversions still truncate |
  | S-0107, F4 (idle timer length), F5 (health ratio) | Yes | Each step of F4 and the ratio of F5 round to nearest |
  | S-0108, F1 (build-up delay), F3, F5, F6, F8, F9 | Yes | Every floating-point step rounds to nearest; F1 can give one tick more |
  | S-0109, F4 (translucent fade) | Yes | The products round to nearest: for a positive end value *E* with 0.6 *E* a whole number, code 3 then needs 5*k* > 3*E* instead of 5*k* ≥ 3*E* (as S-0109 states). The working precision is 53 bits (claim 4) |
  | S-0109, F5 (damage accumulator) | Yes | Each sum and difference rounds to nearest |
  | S-0111, F1 (filled health segments) | Yes | The health ratio and its product with the segment count round to nearest: a ratio of 3/17 with 17 segments shows 3 segments instead of 2 (as S-0111 states) |
  | S-0111, F2 (resource pips) | Yes | Each operation rounds to nearest |
  | S-0111, claim 12 (total passenger size) | Only where a sum is inexact, which with sizes read from data needs a negative size or a total of 2²⁹ or more | Each sum rounds to nearest, so it can reach the whole number above instead of stopping just below it |
  | S-0111, claim 14 (sum of the resource amounts) | Only when two non-zero amounts differ by a factor of 2²⁷ or more (the sum is exact otherwise, as S-0111 states) | The sum rounds to nearest |
  | S-0201 (pacing and freeze rules) | No (no floating point, stated there) | — |
  | S-0202 (random streams, the companion draft of this spec), claim 6 (unit-interval value) | Yes | The largest unit-interval value is exactly 1 instead of 1 − 2⁻⁵³ |
  | S-0204, claim 5, F1 (table square root) | Only for *n* ≥ 2²⁴ square leptons (distances of 4096 leptons or more) | *n* is rounded to the nearest single instead of toward zero; about 1 in 2000 squared distances between 16 and 512 cells moves by one bucket of the table (as S-0204 states). Every spec that uses the table square root (S-0207 F5, S-0208 F1, S-0209 F1, S-0210 F2, S-0212 F6, S-0213 claim 7, S-0215 F4) inherits this |
  | S-0204, claim 8, F3 (direction) | No (stated there) | — |
  | S-0302, Formulas F1, F2, F3, F5, F6 and F7 | Yes | Each RZ₅₃ is RN₅₃ and each RZ₂₄ is RN₂₄; exact operations and results may coincide in both modes |
  | S-0207, F2 (progress per tick), F3 (track cost), F4 (interpolation factor), F6 (target fraction) | Yes | Every floating-point step rounds to nearest; in F4 *A* = 7 gives *t* = 1 exactly, so the seventh displacement of scenario D8 is 22 instead of 21 (as S-0207 states) |
  | S-0208, F3 (step), F5 (walking speed) | Yes | Every floating-point step rounds to nearest; the sideways drift on the map axes (claim 14) disappears (as S-0208 states) |
  | S-0209, F2 (close-enough distance) | Only for a value with a percent sign | The fraction of S-0003 claim 7 rounds to nearest |
  | S-0210, F2 (crush range) | Only for a value with a percent sign; the distance as S-0204, F1 | As S-0209, F2 |
  | S-0211, Formulas (σ, the single-precision narrowing of land-type speed multipliers) | Only for a value with a percent sign (without one σ changes nothing, as S-0211 states) | σ rounds to the nearest single |
  | S-0212, F1 (step cost), F2 (estimate), F9 (coarse route cost) | Yes | Every sum, product and single-precision narrowing rounds to nearest |
  | S-0212, F3 (retry delay), F5 (threat term), F7 (threat tests) | Only for a field with a percent sign, or (F5, F7) a threat value of 2²⁹ or more; otherwise each product is exact, or (the absent-key default of F3) far from a whole number | Each product rounds to nearest |
  | S-0213, claim 7 (group destinations: the points along a placement line) | Yes | The floating-point steps along the line round to nearest; the conversion of each point to a cell still truncates |
  | S-0215, F1 (rate delay) | Only for a rate with a percent sign | The product rounds to nearest: `6%` gives 54 ticks instead of 53. A rate without a percent sign gives the same ticks in both settings (a rate written as 0.06 gives 53, from its single-precision reading) |

- **Integer division by zero, and −2147483648 ÷ −1** (departure, claim 3).
- **Squares from the library power function** (claim 5). Our engine uses the exact square rounded
  to nearest, for every integer distance component, in the launch-angle calculation; a processor
  that gave another value would make the original host-dependent there, so this is a departure
  without a switch.
- **Wider exponent range of intermediate results** (departure, claim 8). A mod with extreme
  values could in principle give a different result in the original. The lead approved this as a
  documented departure with no importer warning.
- **Out-of-range conversions** (claim 7) are the original's behaviour and are reproduced exactly:
  they need no switch. A topic spec whose imported fields can produce an out-of-range conversion
  says so, and the importer warns for such data.

## Import mapping

This spec reads no keys. How data text becomes integers, decimals and percentages is S-0003.

## Measurement scenarios

All scenarios use the glossary's measurement setup, with the director and unrelated mods off.
Unless a referenced scenario fixes another value, use game-speed index 2 and 800 × 600 output.
Each standalone test map includes one bystander computer player, farthest from the observed area,
subject to the glossary's bystander void rule. The observed area is the test infantry patch for N2 and the human
base/sidebar for N5 and N7. Record screenshots or tick-labelled video through a read-only recorder.
Named test data below are the only data overrides.

- **N1, rounding toward zero (claim 5).** Use S-0104 scenario L5 as S-0003 scenario V12 describes it:
  neutral test lighting, an infantry pixel on flat ground whose palette colour has a red channel that
  is a multiple of 8, for example 248. 1 run. *Expected:* the red channel shows the 16-bit step of
  one below that value (247's step), which needs rounding toward zero where the 1000 ‰ tint is
  turned into its factor. *Falsified if* the step of the palette value itself (248's) appears,
  which round-to-nearest arithmetic gives. S-0108 scenario B1 corroborates.
- **N2, conversion of a huge value gives 0 (claim 7).** Named test file: a copy of the rules
  file with the idle-action frequency of S-0107 (its Formulas, F4; Import mapping there) set to
  `1e18`. A test map with five infantry of the human house that have idle animations, standing on
  open ground, no other objects in the observed infantry patch, and the bystander outside it. Video-record 2 game minutes (1800 ticks)
  at a fixed game speed, 1 run, and a control run with the stock value. By S-0107, F4, the timer
  length is then at least about 4.5 × 10²⁰ ticks before conversion (450 times the single-precision
  value of 10¹⁸), which is above 2⁶³, so it converts to 0 and the idle timer has run out at every
  check. *Expected:* each infantry takes idle actions
  (idle animations or turns) again and again, one starting within a few ticks of the previous one
  ending, at least 20 per infantry in the 1800 ticks. *Falsified if* any infantry takes no idle
  action after its first one (which a conversion that saturates at 2147483647 would give) or idles
  only as often as in the control run.
- **N3, signed integer quotient truncation (claim 2).** Use S-0101 scenario M1b: a slowly
  moving vehicle crosses the X = Y diagonal, so its projected screen column involves a signed
  integer quotient. Record column hold times. *Expected:* the diagonal column lasts about twice
  as long as each neighbour. *Falsified if:* its hold time is less than 1.5 times the neighbours'
  average, or a neighbouring column has the doubled hold time instead. This does not test
  floating-point conversion, negative world heights or off-map cell positions.
- **N4, the 53-bit significand (claims 4, 5).** Use S-0104 scenario L19: three runs with grey map
  tints (1.01, 1.01, 1.01) and (1.03, 1.03, 1.03) and a white control, detail low, a plain ground
  tile as the subject, pixels compared across the runs. In S-0104's tint normalisation the divisor
  *D*(*X*) and the quotient of each channel are each one arithmetic result. For *X* = 1010 and
  1030, the stored map values and the scale *K* are the same for every row of the table below,
  and only *D*(*X*) and the quotient change. Computed with
  exact rational arithmetic:

  | Arithmetic | Run 1 (1010) normalised channel | Run 2 (1030) normalised channel |
  |---|---|---|
  | toward zero, 53-bit significand (the original) | 999 | 1000 |
  | toward zero, 24-bit significand | 1000 | 1000 |
  | toward zero, 64-bit significand | 999 | 999 |
  | to nearest, 53-bit significand | 1000 | 1000 |
  | to nearest, 64-bit significand | 999 | 999 |

  *Expected:* as S-0104 states: run 1 is darker than the control (its normalised tint is
  (1000, 999, 999), so it uses the quantised-white table) and run 2 is identical to the control.
  *Falsified if* run 1 is identical to the control, which a 24-bit significand gives, and so does
  rounding to nearest at 53 bits (the quotient then rounds up to exactly 1000); or if run 2
  differs from the control, which a 64-bit significand gives in either rounding direction (every
  grey tint from 2 ‰ up then misses white). A run 1 identical to the control is also what S-0104's
  `quirks.red_normalisation_divisor` off describes; either way the original would contradict this
  spec or S-0104, and both are re-investigated.
- **N5, wrap of conversions from 2³¹ up (claim 7).** Named test map: its superweapon list adds three new superweapons, each a copy of an existing one, granted by
  three new buildable buildings. Their recharge times are *P* = 0.5, *W* = 4772186.5 and *Q* =
  0.75 game minutes, and their display names are chosen so that by name alone they sort *Q*, *P*,
  *W*. These three decimals are exact in single and double precision and their products with 900
  are exact doubles, so the result depends neither on the rounding mode nor on how the field is
  read. By S-0004's sort key (recharge time × 900, converted to ticks), *P* has 450 ticks and *Q*
  675. The exact product for *W* is 4294967850 = 2³² + 554, which wraps to 554 ticks. Procedure:
  build all three buildings and look at the superweapon icons once all three are shown, in two
  runs with the build orders *P*, *W*, *Q* and *Q*, *W*, *P*. *Observable:* the order of the three
  icons on the sidebar tab of S-0004, claim 11. *Expected:* *P*, *W*, *Q* in both runs. *Falsified
  if* *W* comes first (a conversion that gives 0 for every value outside the integer range), *W*
  comes last (a conversion that saturates at 2147483647, or an integer wider than 32 bits without
  the wrap), or the order follows the display names (*Q*, *P*, *W*) or changes with the build
  order.
- **N6, tables instead of library functions (claim 6).** Use S-0204 scenario C3 and S-0009
  scenario M10b (the tabulated square root) and S-0204 scenario C1 (the arctangent table:
  quantised directions and the never-taken facings). *Expected and falsified as* stated there.
  No sine or arcsine measurement is asserted by this scenario.
- **N7, data fractions (claim 9).** The fraction 1.0 = 100 %: use S-0003 scenario V4, *expected and
  falsified as* stated there. The multiplication and conversion: a named test map built like N5,
  with four new superweapons *Y*, *Z*, *X* and *P*, each a copy of an existing one granted by its own
  new buildable building. Their recharge times are written as `30%`, `0.3`, `0.5009765625` and
  `0.5` game minutes, and their display names are chosen so that by name alone they sort *Z*, *Y*,
  *X*, *P*. By S-0004's sort key (recharge time × 900, converted to ticks; S-0004 claim 11): *Y* is
  the double-precision fraction of `30%`, whose product with 900 rounds toward zero to just below
  270, so 269 ticks; *Z* is the single just above 0.3, exact product 270.0000107…, so 270 ticks; *X*
  is exact (0.5 + 2⁻¹⁰), product 450.87890625, so 450 ticks by truncation; *P* has 450 ticks. Procedure:
  build all four buildings and look at the superweapon icons once all four are shown, in two runs
  with the build orders *Y*, *Z*, *X*, *P* and *P*, *X*, *Z*, *Y*. *Observable:* the order of the
  four icons on the sidebar tab of S-0004, claim 11. *Expected:* *Y*, *Z*, *X*, *P* in both runs
  (*X* and *P* tie at 450 ticks and sort by name). *Falsified if* *Z* comes before *Y* (they tie,
  which a value of `0.3` read in double precision, or a product of `30%` rounded to nearest, gives),
  *P* comes before *X* (*X* then has 451 ticks: a conversion that rounds to nearest or up), or the
  order changes with the build order.

## Open questions

- Whether any component outside the program (a display or sound library) changes the rounding mode
  during play, and whether such a change could reach game state before the next conversion renews
  the mode. No such change is known; claim 5's renewal makes any effect short-lived.
- Whether the processor's logarithm and exponential instructions give exactly the squares of
  claim 5 on every processor in use, for distance components below 2²⁶ leptons. S-0306 owns the launch angle of
  gravity-affected projectiles; no practical discriminating measurement is currently known.

**Cross-spec consistency checks**

- **Other specs checked against this one.**
  - S-0004 agrees: scenario F3 writes superweapon *A*'s recharge time as
    `30%`, which gives 269 ticks (claim 9), and keeps `0.3` for *B* and for the control pair, which
    tie at 270 ticks.
  - S-0215 agrees: it attributes the one-tick shortfall of a rate written as
    0.06 to the single-precision reading alone. The product of the single value with 900 is exact
    (claim 9), so the rate delay depends on `quirks.chop_rounding` only for a rate written with a
    percent sign.
  - S-0109 uses the 53-bit significand for the floating-point products of F4,
    including for a negative end value (claim 4).
  - The approved S-0004, S-0101, S-0105 and S-0108 versions no longer call this spec planned. The
    approved glossary also no longer has the old planned-spec wording for S-0205 in its planned
    specs and round-toward-zero mode entries. No companion-spec wording change is required for
    these points.
