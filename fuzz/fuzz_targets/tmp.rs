#![no_main]
//! Fuzz target: `glasnost_fmt::hostile::Target::Tmp`.

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    glasnost_fmt::hostile::run(glasnost_fmt::hostile::Target::Tmp, data);
});
