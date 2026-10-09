#![no_main]
//! Fuzz target: `glasnost_convert::map::decode` (S-0009 map decoding) on
//! arbitrary map text.

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    let doc = glasnost_ini::Document::parse(data, Default::default(), &mut Vec::new());
    let _ = glasnost_convert::map::decode(&doc, &Default::default());
});
