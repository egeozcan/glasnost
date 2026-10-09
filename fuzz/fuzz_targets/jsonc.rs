#![no_main]
//! Fuzz target: `glasnost_rules::hostile::run` (JSONC reader, writer and
//! resolve pipeline).

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    glasnost_rules::hostile::run(data);
});
