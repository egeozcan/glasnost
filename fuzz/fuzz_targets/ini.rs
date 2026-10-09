#![no_main]
//! Fuzz target: `glasnost_ini::hostile::run` (INI reader and value forms).

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    glasnost_ini::hostile::run(data);
});
