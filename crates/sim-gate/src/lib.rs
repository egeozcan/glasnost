//! The determinism gate: runs the synthetic scripted replay of
//! `glasnost_sim::testkit` and gives the state hash after every tick. The
//! same code runs natively (tests and `cargo xtask gate`) and as a bare
//! wasm32 module under node; both must reproduce the committed golden file
//! `golden/scripted.txt` exactly.

use glasnost_sim::{save, testkit};

/// Seed of the gate's replay.
pub const SEED: u32 = 20_261_010;

/// Ticks of the full gate run.
pub const TICKS: i32 = 3000;

/// Ticks checked in debug builds by `cargo test` (debug arithmetic panics on
/// overflow, which release builds would wrap silently).
pub const DEBUG_TICKS: i32 = 300;

/// The state hash after each of the first `ticks` ticks of the replay.
pub fn hashes(ticks: i32) -> Vec<String> {
    let rep = testkit::scripted_replay(SEED, ticks);
    let mut out = Vec::with_capacity(usize::try_from(ticks).unwrap_or(0));
    rep.run(
        testkit::rules(),
        &testkit::map(),
        testkit::catalogue(),
        |w| {
            out.push(save::state_hash(w));
        },
    )
    .expect("the synthetic replay runs");
    out
}

/// The golden file's text for `ticks` ticks: one header line, then the
/// first 16 hexadecimal digits (64 bits) of the state hash after each tick.
pub fn golden_text(ticks: i32) -> String {
    let mut s = format!(
        "# glasnost determinism gate: seed {SEED}, {ticks} ticks, state hash (64 bits) after each tick\n"
    );
    for h in hashes(ticks) {
        s.push_str(&h[..16]);
        s.push('\n');
    }
    s
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    //! Bare exports for node: `gate_run(ticks)` computes the golden text and
    //! returns its length; `gate_ptr()` gives its address in linear memory.

    use std::sync::Mutex;

    static OUT: Mutex<Vec<u8>> = Mutex::new(Vec::new());

    /// Runs the gate for `ticks` ticks; returns the length of the text.
    #[allow(unsafe_code)] // `no_mangle` is an unsafe attribute; the export is plain data
    #[unsafe(no_mangle)]
    pub extern "C" fn gate_run(ticks: i32) -> u32 {
        let text = super::golden_text(ticks).into_bytes();
        let len = u32::try_from(text.len()).unwrap_or(0);
        if let Ok(mut out) = OUT.lock() {
            *out = text;
        }
        len
    }

    /// Address of the text that the last `gate_run` produced.
    #[allow(unsafe_code)] // `no_mangle` is an unsafe attribute; the export is plain data
    #[unsafe(no_mangle)]
    pub extern "C" fn gate_ptr() -> u32 {
        OUT.lock().map_or(0, |o| o.as_ptr() as usize as u32)
    }
}
