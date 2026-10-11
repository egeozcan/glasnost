//! Hostile-input entry point: drives the reader and every value form over
//! arbitrary bytes. Used by `tests/hostile.rs` (stable, in CI) and the
//! `ini` cargo-fuzz target. Any panic or overflow trap is a bug.

use crate::value::{self, DecimalField};
use crate::{Document, Quirks};

/// Parses `data` under the default and the all-off quirk switches, reads
/// every stored value in every form, and layers the data onto its own
/// document.
pub fn run(data: &[u8]) {
    let off = Quirks {
        ini_final_line_needs_newline: false,
        ini_trim_defect: false,
        ini_duplicate_hash_order: false,
        ini_hash_identity: false,
        percent_list_integer_prefix: false,
    };
    for quirks in [Quirks::default(), off] {
        let mut warnings = Vec::new();
        let mut doc = Document::parse(data, quirks, &mut warnings);
        for w in &warnings {
            let _ = w.to_string();
        }
        for s in doc.sections() {
            let _ = doc.section(s.name());
            for (_, e) in s.list_positions() {
                let v = &e.value;
                let _ = s.get(&e.key);
                let _ = value::text(v, value::COMMON_TEXT_LIMIT);
                let _ = value::boolean(v);
                let _ = value::integer(v);
                let _ = value::decimal(v, DecimalField::Double);
                let _ = value::decimal(v, DecimalField::Single);
                let _ = value::int_tuple_or(v, [1, 2, 3]);
                let _ = value::rect(Some(v), [1, 1, 50, 50], true);
                let _ = value::colour_definition(v);
                let _ = value::byte_triple(v, [1, 2, 3]);
                let _ = value::list(v, value::COMMON_TEXT_LIMIT);
                let _ = value::percent_list(
                    v,
                    value::ARMOUR_CLASS_COUNT,
                    quirks.percent_list_integer_prefix,
                );
                let _ = value::named(v, 31, &["a", "b"]);
            }
        }
        let _ = doc.layer(data, &mut warnings);
    }
}
