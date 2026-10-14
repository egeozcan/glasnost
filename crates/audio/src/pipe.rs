//! Native output (feature `native-output`): streams the software mix as raw
//! 16-bit stereo to the standard input of a command-line player
//! (`pw-cat`, `pacat` or `aplay`). It needs no audio development headers,
//! so it builds everywhere; an in-process device backend (cpal or kira,
//! PLAN.md 4.1) can replace it behind the same [`Output`] trait.
//!
//! Pacing: the output counts frames written against the wall clock and
//! renders only enough to stay a short lead ahead, so the pipe never fills
//! and a write never blocks the frame for long.

use std::io::Write;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::time::Instant;

use crate::mixer::Mixer;
use crate::output::{Output, OutputState};

/// How far ahead of the wall clock to keep written, seconds (our design).
const LEAD_MS: u64 = 100;

/// The pipe output.
pub struct PipeOutput {
    rate: u32,
    child: Child,
    stdin: Option<ChildStdin>,
    start: Option<Instant>,
    written: u64,
    buf: Vec<i16>,
    bytes: Vec<u8>,
    player: &'static str,
}

impl PipeOutput {
    /// Starts the first player found.
    pub fn new(rate: u32) -> Result<Self, String> {
        let r = rate.to_string();
        let candidates: [(&'static str, Vec<String>); 3] = [
            (
                "pw-cat",
                vec![
                    "--playback".into(),
                    "--format".into(),
                    "s16".into(),
                    "--rate".into(),
                    r.clone(),
                    "--channels".into(),
                    "2".into(),
                    "-".into(),
                ],
            ),
            (
                "pacat",
                vec![
                    "--playback".into(),
                    "--format=s16le".into(),
                    format!("--rate={r}"),
                    "--channels=2".into(),
                ],
            ),
            (
                "aplay",
                vec![
                    "-q".into(),
                    "-t".into(),
                    "raw".into(),
                    "-f".into(),
                    "S16_LE".into(),
                    "-r".into(),
                    r.clone(),
                    "-c".into(),
                    "2".into(),
                    "-".into(),
                ],
            ),
        ];
        for (player, args) in candidates {
            let spawned = Command::new(player)
                .args(&args)
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();
            if let Ok(mut child) = spawned {
                let stdin = child.stdin.take();
                return Ok(Self {
                    rate,
                    child,
                    stdin,
                    start: None,
                    written: 0,
                    buf: Vec::new(),
                    bytes: Vec::new(),
                    player,
                });
            }
        }
        Err("no command-line audio player found (pw-cat, pacat or aplay)".into())
    }
}

impl Output for PipeOutput {
    fn rate(&self) -> u32 {
        self.rate
    }

    fn pump(&mut self, mixer: &mut Mixer) {
        let Some(stdin) = self.stdin.as_mut() else {
            mixer.clear();
            return;
        };
        let start = *self.start.get_or_insert_with(Instant::now);
        let elapsed_ms = start.elapsed().as_millis() as u64;
        let target = (elapsed_ms + LEAD_MS) * u64::from(self.rate) / 1000;
        if self.written + u64::from(self.rate) < target {
            // Stalled for over a second: skip ahead instead of catching up.
            self.written = target - u64::from(self.rate) / 10;
        }
        let frames = target.saturating_sub(self.written) as usize;
        if frames == 0 {
            return;
        }
        self.buf.resize(frames * 2, 0);
        mixer.render(&mut self.buf);
        self.bytes.clear();
        for s in &self.buf {
            self.bytes.extend_from_slice(&s.to_le_bytes());
        }
        if stdin.write_all(&self.bytes).is_err() {
            self.stdin = None;
            return;
        }
        self.written += frames as u64;
    }

    fn state(&self) -> OutputState {
        if self.stdin.is_some() {
            OutputState::Running
        } else {
            OutputState::Silent(format!("{} stopped", self.player))
        }
    }
}

impl Drop for PipeOutput {
    fn drop(&mut self) {
        self.stdin = None;
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
