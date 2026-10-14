//! Value forms: how a stored value becomes a field (S-0003).
//!
//! Every function takes the *stored value* (S-0003 Terms) of an entry that
//! exists. A missing key, or one with an empty value, never reaches these
//! functions: it leaves the value in force (S-0003 Terms), except for
//! rectangles ([`rect`]). Where a form's failure keeps the value in force,
//! the function returns `None` (or takes the value in force).

use crate::binfloat::{self, Binary, Conversion, DecimalText, Format};
use crate::lex::{is_whitespace, trim_full};

/// Most text and list fields (S-0003 Constants, `common_text_limit`).
pub const COMMON_TEXT_LIMIT: usize = 127; // spec: S-0003 Constants
/// Integer pairs, triples, quadruples and byte triples (S-0003 Constants,
/// `tuple_text_limit`).
pub const TUPLE_TEXT_LIMIT: usize = 63; // spec: S-0003 Constants
/// Values in type list sections (S-0003 Constants, `type_list_text_limit`).
pub const TYPE_LIST_TEXT_LIMIT: usize = 31; // spec: S-0003 Constants
/// Number of armour classes, *k* of the armour-modifier list (S-0003
/// Constants, `armour_class_count`).
pub const ARMOUR_CLASS_COUNT: usize = 11; // spec: S-0003 Constants
/// The text limit of a debris maxima list (S-0003 claim 20).
pub const DEBRIS_MAXIMA_TEXT_LIMIT: usize = 511; // spec: S-0003 Constants (debris-maxima text limit)

/// A text field (S-0003 claim 1): the first `limit` bytes, with trailing
/// whitespace of the cut text removed (no trimming defect).
pub fn text(stored: &[u8], limit: usize) -> &[u8] {
    // spec: S-0003 claim 1
    let cut = &stored[..stored.len().min(limit)];
    let end = cut
        .iter()
        .rposition(|&b| !is_whitespace(b))
        .map_or(0, |i| i + 1);
    &cut[..end]
}

/// A boolean (S-0003 claim 2): the first byte decides; `None` keeps the
/// value in force.
pub fn boolean(stored: &[u8]) -> Option<bool> {
    // spec: S-0003 claim 2 (1, T, Y true; 0, F, N false; case-insensitive)
    match stored.first()?.to_ascii_uppercase() {
        b'1' | b'T' | b'Y' => Some(true),
        b'0' | b'F' | b'N' => Some(false),
        _ => None,
    }
}

/// Digits read from `s` in `radix` (10 or 16), taken modulo 2^32 (S-0003
/// claim 4). Returns the value and the number of digit bytes read.
fn digits_mod_2_32(s: &[u8], radix: u32) -> (u32, usize) {
    let mut v = 0u32;
    let mut n = 0;
    for &b in s {
        let Some(d) = (b as char).to_digit(radix) else {
            break;
        };
        // spec: S-0003 claim 4, Formulas (u = D mod 2^32)
        v = v.wrapping_mul(radix).wrapping_add(d);
        n += 1;
    }
    (v, n)
}

/// Optional whitespace, then an optional sign (S-0003 claim 3). Returns
/// whether a `-` was read and the rest.
fn sign(s: &[u8], allow_plus: bool) -> (bool, &[u8]) {
    let start = s.iter().take_while(|&&b| is_whitespace(b)).count();
    let s = &s[start..];
    match s.first() {
        Some(b'-') => (true, &s[1..]),
        Some(b'+') if allow_plus => (false, &s[1..]),
        _ => (false, s),
    }
}

/// `(s · u) mod 2^32` read as signed (S-0003 Formulas, integer forms).
fn apply_sign(neg: bool, u: u32) -> i32 {
    // spec: S-0003 Formulas (v = (s · u) mod 2^32, signed two's complement)
    (if neg { u.wrapping_neg() } else { u }) as i32
}

/// Hexadecimal after optional whitespace, sign and `0x`/`0X` prefix
/// (S-0003 claim 3). `None` if no hexadecimal digit follows.
fn hex_form(s: &[u8]) -> Option<i32> {
    let (neg, rest) = sign(s, true);
    // spec: S-0003 claim 3 (a `0x` with no hex digit after it is the prefix, not a digit)
    let rest = match rest {
        [b'0', b'x' | b'X', tail @ ..] => tail,
        _ => rest,
    };
    let (u, n) = digits_mod_2_32(rest, 16);
    (n > 0).then(|| apply_sign(neg, u))
}

/// An integer (S-0003 claims 3 and 4). `None` keeps the value in force
/// (a hexadecimal form with no digit); the decimal form gives 0 when it
/// finds no digit.
pub fn integer(stored: &[u8]) -> Option<i32> {
    match (stored.first(), stored.last()) {
        // spec: S-0003 claim 3 (`$` prefix; also when the last byte is h or H)
        (Some(b'$'), _) => hex_form(&stored[1..]),
        // spec: S-0003 claim 3 (h suffix, read from the first byte)
        (_, Some(b'h' | b'H')) => hex_form(stored),
        _ => Some(decimal_integer(stored)),
    }
}

/// The decimal integer form (S-0003 claim 3): a decimal prefix, 0 when
/// there are no digits.
pub fn decimal_integer(s: &[u8]) -> i32 {
    let (neg, rest) = sign(s, true);
    let (u, n) = digits_mod_2_32(rest, 10);
    // spec: S-0003 claim 3 (no digits: the result is 0, not the value in force)
    if n == 0 { 0 } else { apply_sign(neg, u) }
}

/// The longest prefix of `s` that forms a decimal number (S-0003 claim 6),
/// or `None` if `s` starts with no number.
pub fn decimal_prefix(s: &[u8]) -> Option<DecimalText> {
    let (neg, rest) = sign(s, true);
    let int_len = rest.iter().take_while(|b| b.is_ascii_digit()).count();
    let mut digits = rest[..int_len].to_vec();
    let mut i = int_len;
    let mut frac_len = 0usize;
    if rest.get(i) == Some(&b'.') {
        let f = rest[i + 1..]
            .iter()
            .take_while(|b| b.is_ascii_digit())
            .count();
        digits.extend_from_slice(&rest[i + 1..i + 1 + f]);
        frac_len = f;
        i += 1 + f;
    }
    // spec: S-0003 claim 6 (either side of the point may be empty, not both)
    if digits.is_empty() {
        return None;
    }
    let mut exp10 = 0i64;
    if matches!(rest.get(i), Some(b'e' | b'E')) {
        // spec: S-0003 claim 6 (exponent: `e` or `E`, optional sign, digits)
        let mut j = i + 1;
        let eneg = match rest.get(j) {
            Some(b'-') => {
                j += 1;
                true
            }
            Some(b'+') => {
                j += 1;
                false
            }
            _ => false,
        };
        let n = rest[j..].iter().take_while(|b| b.is_ascii_digit()).count();
        if n > 0 {
            // Saturate absurd exponents; the converter settles them early.
            let mut e = 0i64;
            for &d in &rest[j..j + n] {
                e = (e * 10 + i64::from(d - b'0')).min(1_000_000);
            }
            exp10 = if eneg { -e } else { e };
        }
    }
    Some(DecimalText {
        neg,
        digits,
        exp10: exp10 - frac_len as i64,
    })
}

/// The precision a decimal field keeps (S-0003 claim 7).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DecimalField {
    /// Kept as a double (the common case).
    #[default]
    Double,
    /// Kept in single precision: a `%` result is narrowed toward zero.
    Single,
}

/// A decimal field's result (S-0003 claims 6 to 8).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decimal {
    /// A value, with the warning flag of S-0003 Formulas.
    Value {
        /// The value: single without `%`; with `%` a double, or a single
        /// in a single-precision field.
        value: Binary,
        /// Claim 6's warning test fired.
        uncovered: bool,
        /// A `%` was present.
        percent: bool,
    },
    /// No number at the start: the original's result is unspecified; our
    /// engine keeps the value in force and the importer warns (claim 8,
    /// a departure).
    NoNumber,
}

/// A decimal or percentage field (S-0003 claims 6 to 8), with the original
/// rounding mode (`quirks.chop_rounding` on).
pub fn decimal(stored: &[u8], field: DecimalField) -> Decimal {
    decimal_rounded(stored, field, true)
}

/// A decimal or percentage field (S-0003 claims 6 to 8); `chop` is
/// `quirks.chop_rounding`, which rounds a percentage's product and its
/// narrowing (S-0003 Formulas, Q₅₃ and Q₂₄).
pub fn decimal_rounded(stored: &[u8], field: DecimalField, chop: bool) -> Decimal {
    let Some(text) = decimal_prefix(stored) else {
        // spec: S-0003 claim 8 (our engine: keep the value in force)
        return Decimal::NoNumber;
    };
    // spec: S-0003 claim 6 (single precision)
    let Conversion { value, uncovered } = binfloat::decimal_to_binary(&text, Format::Single);
    // spec: S-0003 claim 7 and Formulas (a `%` anywhere multiplies by c: Q₅₃, RZ₅₃ with quirks.chop_rounding on, RN₅₃ off)
    let percent = stored.contains(&b'%');
    let value = if percent {
        let product = value.mul_rounded(&binfloat::percent_factor(), Format::Double, chop);
        match field {
            DecimalField::Double => product,
            // spec: S-0003 claim 7 and Formulas (single-precision fields narrow by Q₂₄)
            DecimalField::Single => product.convert_rounded(Format::Single, chop),
        }
    } else {
        value
    };
    Decimal::Value {
        value,
        uncovered,
        percent,
    }
}

/// Up to `N` comma-separated integers (S-0003 claim 9): `None` for each
/// component that could not be read.
pub fn int_tuple<const N: usize>(stored: &[u8]) -> [Option<i32>; N] {
    let mut out = [None; N];
    // spec: S-0003 claim 9 (only the first 63 bytes, trimmed)
    let mut s = trim_full(&stored[..stored.len().min(TUPLE_TEXT_LIMIT)]);
    for slot in &mut out {
        let (neg, rest) = sign(s, true);
        let (u, n) = digits_mod_2_32(rest, 10);
        // spec: S-0003 claim 9 (a number with no digits stops the reading)
        if n == 0 {
            break;
        }
        *slot = Some(apply_sign(neg, u));
        // spec: S-0003 claim 9 (the comma must follow the digits immediately)
        match rest.get(n) {
            Some(b',') => s = &rest[n + 1..],
            _ => break,
        }
    }
    out
}

/// An integer pair or triple (S-0003 claim 9). Unread components take the
/// value in force's (our engine; the original's are unspecified). The flag
/// is set when a component was unread, and the importer warns.
pub fn int_tuple_or<const N: usize>(stored: &[u8], in_force: [i32; N]) -> ([i32; N], bool) {
    let read = int_tuple::<N>(stored);
    let mut out = in_force;
    let mut unread = false;
    for (o, r) in out.iter_mut().zip(read) {
        match r {
            Some(v) => *o = v,
            // spec: S-0003 claim 9 (our engine: the value in force's component)
            None => unread = true,
        }
    }
    (out, unread)
}

/// A rectangle (S-0003 claim 10). `stored` is `None` for a missing key or
/// an empty value, which gives four zeros with
/// `quirks.absent_rect_is_zero` on and the component fallback with it off.
pub fn rect(stored: Option<&[u8]>, fallback: [i32; 4], absent_rect_is_zero: bool) -> [i32; 4] {
    let Some(stored) = stored else {
        // spec: S-0003 claim 10 (a missing key gives 0, 0, 0, 0)
        return if absent_rect_is_zero {
            [0; 4]
        } else {
            fallback
        };
    };
    let read = int_tuple::<4>(stored);
    let mut out = fallback;
    for (o, r) in out.iter_mut().zip(read) {
        // spec: S-0003 claim 10 (unread components take the component fallback)
        if let Some(v) = r {
            *o = v;
        }
    }
    out
}

/// Low 8 bits of an integer (S-0003 Formulas, byte triple: `v mod 256`).
fn low_byte(v: i32) -> u8 {
    // spec: S-0003 claim 11 (each component is the low 8 bits)
    v as u8
}

/// A colour definition (S-0003 claim 11, colour-definition form; S-0103
/// claim 4): unread components are 0.
pub fn colour_definition(stored: &[u8]) -> [u8; 3] {
    // spec: S-0003 claim 11, S-0103 claim 4 (unread components are 0)
    int_tuple::<3>(stored).map(|c| c.map_or(0, low_byte))
}

/// A generic byte triple (S-0003 claim 11): unread components take the
/// value in force's (our engine), with the flag set for a warning.
pub fn byte_triple(stored: &[u8], in_force: [u8; 3]) -> ([u8; 3], bool) {
    let read = int_tuple::<3>(stored);
    let mut out = in_force;
    let mut unread = false;
    for (o, r) in out.iter_mut().zip(read) {
        match r {
            Some(v) => *o = low_byte(v),
            // spec: S-0003 claim 11 (our engine: the value in force's component)
            None => unread = true,
        }
    }
    (out, unread)
}

/// A list (S-0003 claim 12): the text cut to `limit` (claim 1), split at
/// commas, empty items skipped, items not trimmed.
pub fn list(stored: &[u8], limit: usize) -> Vec<&[u8]> {
    // spec: S-0003 claim 12
    text(stored, limit)
        .split(|&b| b == b',')
        .filter(|item| !item.is_empty())
        .collect()
}

/// A list of names (S-0003 claim 12): as [`list`], and with
/// `quirks.list_items_untrimmed` off (`untrimmed` false) each item trimmed
/// of its leading and trailing whitespace, an item left empty being skipped.
pub fn name_items(stored: &[u8], limit: usize, untrimmed: bool) -> Vec<&[u8]> {
    let items = list(stored, limit);
    if untrimmed {
        return items;
    }
    // spec: S-0003 claim 12 and Edge cases (quirks.list_items_untrimmed off: S-0001's trim without its defect; a whitespace-only item is skipped like an empty one)
    items
        .into_iter()
        .map(trim_full)
        .filter(|i| !i.is_empty())
        .collect()
}

/// Whether a name item begins or ends with a whitespace byte (S-0003
/// claim 12: with `quirks.list_items_untrimmed` on the importer warns).
pub fn edge_whitespace(item: &[u8]) -> bool {
    item.first().is_some_and(|&b| is_whitespace(b))
        || item.last().is_some_and(|&b| is_whitespace(b))
}

/// A debris maxima list (S-0003 claim 20): each non-empty item of the
/// list, cut at 511 bytes, gives one decimal integer prefix (claims 3 and
/// 4: no hexadecimal forms; an item with no digits gives 0 and keeps its
/// place); empty items take no place.
pub fn decimal_prefix_list(stored: &[u8]) -> Vec<i32> {
    // spec: S-0003 claim 20 (decimal prefixes only, 511-byte limit, empty items skipped)
    list(stored, DEBRIS_MAXIMA_TEXT_LIMIT)
        .into_iter()
        .map(decimal_integer)
        .collect()
}

/// What S-0003 asks the importer to warn about in a decimal-prefix integer
/// list (claims 20, 22 and 23, with claim 3's warnings item by item and
/// claim 1's text-cut warning).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DecimalListNotes {
    /// Items with no decimal digit (each gives 0 and keeps its place).
    pub no_digit: usize,
    /// Items with bytes other than whitespace after their number (no
    /// hexadecimal suffix exception).
    pub trailing: usize,
    /// Items whose digits exceed 32 bits (they wrap, claim 4).
    pub overflow: usize,
    /// The 511-byte cut discarded a byte that is not whitespace.
    pub cut: bool,
}

impl DecimalListNotes {
    /// Whether there is anything to warn about.
    pub fn any(&self) -> bool {
        self.no_digit + self.trailing + self.overflow > 0 || self.cut
    }
}

/// The importer notes of [`decimal_prefix_list`] for one stored value.
pub fn decimal_prefix_list_notes(stored: &[u8]) -> DecimalListNotes {
    let mut notes = DecimalListNotes {
        // spec: S-0003 claim 1 (the importer warns when the cut discards a byte that is not whitespace)
        cut: stored
            .iter()
            .skip(DEBRIS_MAXIMA_TEXT_LIMIT)
            .any(|&b| !is_whitespace(b)),
        ..DecimalListNotes::default()
    };
    for item in list(stored, DEBRIS_MAXIMA_TEXT_LIMIT) {
        // spec: S-0003 claim 3 (warnings: no digit; bytes other than whitespace after the number) and claim 23 (no hexadecimal suffix exception)
        let (neg, rest) = sign(item, true);
        let n = rest.iter().take_while(|b| b.is_ascii_digit()).count();
        if n == 0 {
            notes.no_digit += 1;
        }
        if rest[n..].iter().any(|&b| !is_whitespace(b)) {
            notes.trailing += 1;
        }
        // spec: S-0003 claim 4 (decimal form: the importer warns when the signed number read lies outside the 32-bit range)
        let magnitude = rest[..n].iter().fold(0u64, |v, &d| {
            v.saturating_mul(10).saturating_add(u64::from(d - b'0'))
        });
        let bound = if neg { 1u64 << 31 } else { (1u64 << 31) - 1 };
        if magnitude > bound {
            notes.overflow += 1;
        }
    }
    notes
}

/// A fixed-count percentage list (S-0003 claim 13), such as a warhead's
/// armour modifiers. Returns `k` doubles and what claim 13 asks the
/// importer to warn about: missing items (our engine fills them with 1.0;
/// the original crashes), items beyond `k`, items with no number, items
/// whose `%` reading drops a fraction, items without `%` above 1 in
/// magnitude (claim 18) and the uncovered conversion cases.
///
/// `integer_prefix` is `quirks.percent_list_integer_prefix`: on, an item
/// containing `%` gives its integer decimal prefix times *c*; off, its
/// decimal at 53 bits times *c*. `chop` is `quirks.chop_rounding`: the
/// product is rounded toward zero when it is on, to nearest (ties to even)
/// when it is off (Q₅₃ of S-0003 Formulas).
pub fn percent_list(
    stored: &[u8],
    k: usize,
    integer_prefix: bool,
    chop: bool,
) -> (Vec<Binary>, PercentListNotes) {
    let c = binfloat::percent_factor();
    let items = list(stored, COMMON_TEXT_LIMIT);
    let mut notes = PercentListNotes {
        // spec: S-0003 claim 13 (items beyond k are ignored; the importer warns)
        extra: items.len().saturating_sub(k),
        ..PercentListNotes::default()
    };
    let mut out: Vec<Binary> = items
        .iter()
        .take(k)
        .map(|item| {
            // spec: S-0003 claim 13 (the decimal of an item at 53 bits, claim 6's rule)
            let full = decimal_prefix(item).map(|text| {
                let conv = binfloat::decimal_to_binary(&text, Format::Double);
                notes.uncovered |= conv.uncovered;
                conv.value
            });
            if full.is_none() {
                // spec: S-0003 claim 13 (an item with no number gives +0.0; the importer warns)
                notes.no_number += 1;
            }
            if item.contains(&b'%') {
                // spec: S-0003 claim 13 (with `%`: the integer decimal prefix × c, RZ53)
                let i = decimal_integer(item);
                let (neg, m) = (i < 0, i.unsigned_abs());
                let i = Binary::round_toward_zero(Format::Double, neg && m != 0, u128::from(m), 0);
                if let Some(d) = full
                    && d.to_exact_decimal().trim_start_matches('-')
                        != i.to_exact_decimal().trim_start_matches('-')
                {
                    // spec: S-0003 claim 13, Formulas (the integer-prefix warning: d ≠ i, exactly)
                    notes.prefix_differs += usize::from(integer_prefix);
                }
                // spec: S-0003 Formulas (the product rounded by Q₅₃: quirks.chop_rounding)
                if integer_prefix {
                    i.mul_rounded(&c, Format::Double, chop)
                } else {
                    // spec: S-0003 Edge cases (switch off: the item's decimal at 53 bits × c, Q₅₃)
                    full.unwrap_or(Binary::zero(Format::Double, false))
                        .mul_rounded(&c, Format::Double, chop)
                }
            } else {
                // spec: S-0003 claim 13 (without `%`: claim 6's rule at 53 bits; no number gives +0.0)
                let v = full.unwrap_or(Binary::zero(Format::Double, false));
                if above_one(&v) {
                    // spec: S-0003 claim 13 and claim 18 (an item without `%` above 1 in magnitude)
                    notes.above_one += 1;
                }
                v
            }
        })
        .collect();
    if out.len() < k {
        // spec: S-0003 claim 13 (our engine: missing items are 100 %)
        notes.missing = k - out.len();
        out.resize(k, Binary::one(Format::Double));
    }
    (out, notes)
}

/// Whether a value is greater than 1 in magnitude (infinity included).
fn above_one(v: &Binary) -> bool {
    let exact = v.to_exact_decimal();
    let m = exact.trim_start_matches('-');
    if m == "inf" {
        return true;
    }
    let (int, frac) = m.split_once('.').unwrap_or((m, ""));
    let int = int.trim_start_matches('0');
    !int.is_empty() && (int != "1" || frac.bytes().any(|b| b != b'0'))
}

/// What [`percent_list`] found worth a warning.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PercentListNotes {
    /// Items missing below the fixed count (filled with 1.0).
    pub missing: usize,
    /// Items beyond the fixed count (ignored).
    pub extra: usize,
    /// Items with no number (each +0.0).
    pub no_number: usize,
    /// Items with `%` whose decimal differs from their integer prefix,
    /// counted only while the integer-prefix reading is on.
    pub prefix_differs: usize,
    /// Items without `%` above 1 in magnitude (S-0003 claim 18).
    pub above_one: usize,
    /// Some item fell in claim 6's uncovered cases.
    pub uncovered: bool,
}

/// Whether two type names match (S-0003 claim 14): ASCII case folding of
/// A to Z only.
pub fn type_name_eq(a: &[u8], b: &[u8]) -> bool {
    // spec: S-0003 claim 14
    a.eq_ignore_ascii_case(b)
}

/// Whether a type-name value means "no type" (S-0003 claim 14): `none`
/// or `<none>`, in any case.
pub fn is_no_type(name: &[u8]) -> bool {
    // spec: S-0003 claim 14, S-0004 claim 2
    name.eq_ignore_ascii_case(b"none") || name.eq_ignore_ascii_case(b"<none>")
}

/// A named value (S-0003 claim 16): the text, cut to `limit`, compared
/// with `vocabulary` ignoring ASCII case. `None` means no word matched,
/// which gives the field's own fallback (not the value in force).
pub fn named(stored: &[u8], limit: usize, vocabulary: &[&str]) -> Option<usize> {
    let word = text(stored, limit);
    vocabulary
        .iter()
        .position(|v| word.eq_ignore_ascii_case(v.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// spec: S-0003 claims 1 and 12 (scenario V1, V7)
    #[test]
    fn text_and_lists() {
        assert_eq!(text(b"abc   def", 5), b"abc");
        assert_eq!(text(b"abcdef", 3), b"abc");
        assert_eq!(list(b",,A,,B, C,", 127), [&b"A"[..], b"B", b" C"]);
        let long = format!("{}GAPOWR", ",".repeat(124));
        assert_eq!(list(long.as_bytes(), 127), [&b"GAP"[..]]);
    }

    /// spec: S-0003 claim 20 (scenario V15: decimal prefixes only; an item without digits gives 0 and keeps its place; empty items take none; 511-byte limit)
    #[test]
    fn claim_20_debris_maxima_lists() {
        assert_eq!(decimal_prefix_list(b"2,,abc,3"), [2, 0, 3]);
        assert_eq!(decimal_prefix_list(b"2,,3"), [2, 3]);
        assert_eq!(decimal_prefix_list(b"$10,3"), [0, 3]);
        assert_eq!(decimal_prefix_list(b"10h,3"), [10, 3]);
        assert_eq!(decimal_prefix_list(b"0x10,3"), [0, 3]);
        assert_eq!(decimal_prefix_list(b"none,3"), [0, 3]);
        assert_eq!(decimal_prefix_list(b"-4,4294967297"), [-4, 1]);
        // 513 bytes: the cut at 511 keeps "7" and drops ",8".
        let long = format!("{}7,8", "1,".repeat(255));
        let read = decimal_prefix_list(long.as_bytes());
        assert_eq!((read.len(), read[255]), (256, 7));
    }

    /// spec: S-0003 claim 2 (scenario V2)
    #[test]
    fn booleans() {
        for t in ["yes", "Yes please", "true", "t", "1"] {
            assert_eq!(boolean(t.as_bytes()), Some(true), "{t}");
        }
        for f in ["no", "False", "Falsehood", "0", "Nope"] {
            assert_eq!(boolean(f.as_bytes()), Some(false), "{f}");
        }
        for k in ["on", "off", "2", "-1"] {
            assert_eq!(boolean(k.as_bytes()), None, "{k}");
        }
    }

    /// spec: S-0003 claims 3 and 4 (scenario V3)
    #[test]
    fn integers() {
        let cases: [(&str, Option<i32>); 34] = [
            ("$1F", Some(31)),
            ("$1f", Some(31)),
            ("$-1", Some(-1)),
            ("$0x", None),
            ("$-0x", None),
            ("$0xg", None),
            ("$0", Some(0)),
            ("$0x10", Some(16)),
            ("1Fh", Some(31)),
            ("-1h", Some(-1)),
            ("abch", Some(2748)),
            ("0x10h", Some(16)),
            ("1F h", Some(31)),
            ("zzh", None),
            ("H", None),
            ("0xh", None),
            ("0xgh", None),
            ("0h", Some(0)),
            ("$10h", Some(16)),
            ("12abc", Some(12)),
            ("3.9", Some(3)),
            ("1e3", Some(1)),
            ("1,2", Some(1)),
            ("abc", Some(0)),
            ("--5", Some(0)),
            ("- 5", Some(0)),
            ("0x1F", Some(0)),
            ("2147483648", Some(-2_147_483_648)),
            ("4294967296", Some(0)),
            ("99999999999", Some(1_215_752_191)),
            ("-2147483649", Some(2_147_483_647)),
            ("$100000000", Some(0)),
            ("7FFFFFFFFh", Some(-1)),
            ("1Fh ", Some(1)),
        ];
        for (s, want) in cases {
            assert_eq!(integer(s.as_bytes()), want, "{s:?}");
        }
        // Scenario V3's spellings.
        assert_eq!(integer(b"$64"), Some(100));
        assert_eq!(integer(b"64h"), Some(100));
        assert_eq!(integer(b"4294967396"), Some(100));
        assert_eq!(integer(b"100junk"), Some(100));
    }

    fn dec_str(s: &str) -> String {
        match decimal(s.as_bytes(), DecimalField::Double) {
            Decimal::Value { value, .. } => value.to_exact_decimal(),
            Decimal::NoNumber => "none".into(),
        }
    }

    /// spec: S-0003 claims 6, 7 and 8 (scenario V4)
    #[test]
    fn decimals_and_percentages() {
        assert_eq!(dec_str("0.1"), "0.100000001490116119384765625");
        assert_eq!(dec_str(".5"), "0.5");
        assert_eq!(dec_str("5."), "5");
        assert_eq!(dec_str("1,5"), "1");
        assert_eq!(dec_str("-0x5h"), "-0");
        assert_eq!(dec_str("50%"), "0.5");
        assert_eq!(dec_str("50 %"), "0.5");
        assert_eq!(dec_str("100%%"), "1");
        assert_eq!(dec_str("50"), "50");
        assert_eq!(dec_str("1e2"), "100");
        assert_eq!(dec_str("1e"), "1");
        assert_eq!(dec_str("1e+2"), "100");
        for none in ["abc", "x5", "%50", "inf", "nan", ".", "-"] {
            assert_eq!(dec_str(none), "none", "{none}");
        }
        // 0.5%: 0.5 × c is exact, the double nearest to 0.005.
        let Decimal::Value { value, .. } = decimal(b"0.5%", DecimalField::Double) else {
            panic!()
        };
        assert_eq!(value.bits, 0x3F74_7AE1_47AE_147B);
        let Decimal::Value { value, .. } = decimal(b"10%", DecimalField::Single) else {
            panic!()
        };
        assert_eq!(value.to_exact_decimal(), "0.0999999940395355224609375");
    }

    /// spec: S-0003 claim 7 and Formulas (with quirks.chop_rounding off the product is RN₅₃ and a single field narrows by RN₂₄)
    #[test]
    fn percent_rounding_follows_chop() {
        let bits = |s: &[u8], f, chop| match decimal_rounded(s, f, chop) {
            Decimal::Value { value, .. } => value.bits,
            Decimal::NoNumber => panic!(),
        };
        assert_eq!(
            bits(b"10%", DecimalField::Double, true),
            0x3FB9_9999_9999_9999
        );
        assert_eq!(
            bits(b"10%", DecimalField::Double, false),
            0x3FB9_9999_9999_999A
        );
        assert_eq!(bits(b"10%", DecimalField::Single, true), 0x3DCC_CCCC);
        assert_eq!(bits(b"10%", DecimalField::Single, false), 0x3DCC_CCCD);
        // Exact products do not depend on the switch.
        assert_eq!(
            bits(b"50%", DecimalField::Double, false),
            bits(b"50%", DecimalField::Double, true)
        );
        // Without `%` the switch plays no part.
        assert_eq!(
            bits(b"0.1", DecimalField::Double, false),
            bits(b"0.1", DecimalField::Double, true)
        );
    }

    /// spec: S-0003 claim 13 and Formulas (a `%` item's product is Q₅₃: RZ₅₃ with quirks.chop_rounding on, RN₅₃ off, under either integer-prefix setting)
    #[test]
    fn percent_list_rounding_follows_chop() {
        for prefix in [true, false] {
            let (on, _) = percent_list(b"10%,0.1", 2, prefix, true);
            let (off, _) = percent_list(b"10%,0.1", 2, prefix, false);
            assert_eq!(on[0].bits, 0x3FB9_9999_9999_9999);
            assert_eq!(off[0].bits, 0x3FB9_9999_9999_999A);
            assert_eq!(on[1], off[1], "an item without `%` is unaffected");
        }
    }

    /// spec: S-0003 claim 9 (scenario V5)
    #[test]
    fn tuples() {
        assert_eq!(int_tuple::<3>(b"1, 2, 3"), [Some(1), Some(2), Some(3)]);
        assert_eq!(int_tuple::<3>(b"1 ,2 ,3"), [Some(1), None, None]);
        assert_eq!(int_tuple::<3>(b"a,2,3"), [None, None, None]);
        assert_eq!(int_tuple::<2>(b"0x10,5"), [Some(0), None]);
        assert_eq!(int_tuple::<2>(b"$10,5"), [None, None]);
        assert_eq!(int_tuple::<2>(b"1,2,3"), [Some(1), Some(2)]);
        assert_eq!(int_tuple_or(b"1,2", [7, 8, 9]), ([1, 2, 9], true));
        assert_eq!(int_tuple_or(b"a,2,3", [7, 8, 9]), ([7, 8, 9], true));
        let long = format!("1,{}2", " ".repeat(70));
        assert_eq!(int_tuple::<2>(long.as_bytes()), [Some(1), None]);
    }

    /// spec: S-0003 claim 10 (scenario V5b)
    #[test]
    fn rectangles() {
        let fb = [1, 1, 50, 50];
        assert_eq!(rect(None, fb, true), [0, 0, 0, 0]);
        assert_eq!(rect(None, fb, false), fb);
        assert_eq!(rect(Some(b"0,0,30"), fb, true), [0, 0, 30, 50]);
        assert_eq!(rect(Some(b"a"), fb, true), fb);
    }

    /// spec: S-0003 claim 11 (scenario V6), S-0103 claim 4
    #[test]
    fn byte_triples() {
        assert_eq!(colour_definition(b"300,255,255"), [44, 255, 255]);
        assert_eq!(colour_definition(b"40,255"), [40, 255, 0]);
        assert_eq!(colour_definition(b"a,255,255"), [0, 0, 0]);
        assert_eq!(colour_definition(b"10,abc,20"), [10, 0, 0]);
        assert_eq!(colour_definition(b"10 ,20,30"), [10, 0, 0]);
        assert_eq!(colour_definition(b"-1,-86,256"), [255, 170, 0]);
        assert_eq!(byte_triple(b"1,2", [7, 8, 9]), ([1, 2, 9], true));
    }

    /// spec: S-0003 claim 13 (scenario V8)
    #[test]
    fn percent_lists() {
        let (v, notes) = percent_list(
            b"12.5%,0.125,abc,-abc,inf,50%",
            ARMOUR_CLASS_COUNT,
            true,
            true,
        );
        let s: Vec<String> = v.iter().map(Binary::to_exact_decimal).collect();
        assert!(
            s[0].starts_with("0.11999999") || s[0].starts_with("0.12"),
            "{}",
            s[0]
        );
        assert_eq!(s[1], "0.125");
        assert_eq!(s[2], "0");
        assert_eq!(s[3], "0");
        assert_eq!(s[4], "0");
        assert_eq!(s[5], "0.5");
        assert_eq!(&s[6..], ["1", "1", "1", "1", "1"]);
        assert_eq!(notes.missing, 5);
        // `12.5%` uses the integer prefix 12, never 0.125.
        let twelve = Binary::round_toward_zero(Format::Double, false, 12, 0)
            .mul_rz(&binfloat::percent_factor(), Format::Double);
        assert_eq!(v[0], twelve);
        assert_eq!((notes.no_number, notes.prefix_differs), (3, 1));
        assert_eq!((notes.extra, notes.above_one), (0, 0));
    }

    /// spec: S-0003 claim 13 and Edge cases (`quirks.percent_list_integer_prefix` off; the warnings of claim 13)
    #[test]
    fn percent_list_switch_and_notes() {
        let c = binfloat::percent_factor();
        let full = |s: &str| {
            let t = decimal_prefix(s.as_bytes()).unwrap();
            binfloat::decimal_to_binary(&t, Format::Double)
                .value
                .mul_rz(&c, Format::Double)
        };
        let (v, notes) = percent_list(
            b"12.5%,1e2%,12%,12.0%,2,-3.5",
            ARMOUR_CLASS_COUNT,
            false,
            true,
        );
        assert_eq!(v[0], full("12.5"));
        assert_eq!(v[0].to_exact_decimal(), "0.125");
        assert_eq!(v[1].to_exact_decimal(), "1");
        // Switch off: no integer-prefix warning.
        assert_eq!(notes.prefix_differs, 0);
        assert_eq!(notes.above_one, 2);
        let (v, notes) = percent_list(b"12.5%,1e2%,12%,12.0%", ARMOUR_CLASS_COUNT, true, true);
        // 1e2% reads its integer prefix 1: 0.01.
        assert_eq!(v[1], Binary::one(Format::Double).mul_rz(&c, Format::Double));
        assert_eq!(v[2], v[3]);
        assert_eq!(notes.prefix_differs, 2);
        let twelve = "1,1,1,1,1,1,1,1,1,1,1,1";
        let (_, notes) = percent_list(twelve.as_bytes(), ARMOUR_CLASS_COUNT, true, true);
        assert_eq!((notes.extra, notes.missing), (1, 0));
    }

    /// spec: S-0003 claims 14 and 16
    #[test]
    fn names() {
        assert!(type_name_eq(b"alpha", b"Alpha"));
        assert!(is_no_type(b"NONE"));
        assert!(is_no_type(b"<None>"));
        assert!(!is_no_type(b"nonesuch"));
        assert_eq!(named(b"amphibious", 31, &["Normal", "Amphibious"]), Some(1));
        assert_eq!(named(b"Swim", 31, &["Normal", "Amphibious"]), None);
    }
}
