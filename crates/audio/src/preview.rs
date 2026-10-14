//! Audio playback.
//!
//! **Milestone M1**: preview playback of decoded 16-bit PCM for the asset
//! viewer ([`Player`]). On the web it uses WebAudio. Natively it is a
//! development stopgap: the sound is written to a temporary WAV file and
//! handed to the first command-line player found (`pw-play`, `paplay`,
//! `aplay`, `afplay`), because an in-process backend (kira or cpal, PLAN.md
//! 4.1) needs system audio headers that not every development machine has.
//! **M5** replaces the native path with an in-process mixer for SFX, voices
//! and EVA.

/// A 16-bit PCM WAV file holding `samples` (interleaved).
pub fn wav_bytes(sample_rate: u32, channels: u16, samples: &[i16]) -> Vec<u8> {
    let data_len = (samples.len() * 2) as u32;
    let block_align = channels * 2;
    let mut out = Vec::with_capacity(44 + samples.len() * 2);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&channels.to_le_bytes());
    out.extend_from_slice(&sample_rate.to_le_bytes());
    out.extend_from_slice(&(sample_rate * u32::from(block_align)).to_le_bytes());
    out.extend_from_slice(&block_align.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    for s in samples {
        out.extend_from_slice(&s.to_le_bytes());
    }
    out
}

/// Plays one sound at a time; starting a sound stops the previous one.
#[derive(Default)]
pub struct Player {
    #[cfg(not(target_arch = "wasm32"))]
    child: Option<std::process::Child>,
    #[cfg(not(target_arch = "wasm32"))]
    wav: Option<std::path::PathBuf>,
    #[cfg(target_arch = "wasm32")]
    context: Option<web_sys::AudioContext>,
    #[cfg(target_arch = "wasm32")]
    source: Option<web_sys::AudioBufferSourceNode>,
}

impl Player {
    /// A player with nothing playing.
    pub fn new() -> Self {
        Self::default()
    }

    /// Stops whatever is playing.
    pub fn stop(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(mut child) = self.child.take() {
                let _ = child.kill();
                let _ = child.wait();
            }
            if let Some(path) = self.wav.take() {
                let _ = std::fs::remove_file(path);
            }
        }
        #[cfg(target_arch = "wasm32")]
        if let Some(source) = self.source.take() {
            #[allow(deprecated)]
            let _ = source.stop();
        }
    }

    /// Plays interleaved 16-bit samples.
    pub fn play(&mut self, sample_rate: u32, channels: u16, samples: &[i16]) -> Result<(), String> {
        self.stop();
        if channels == 0 || sample_rate == 0 {
            return Err("sound has no channels or no sample rate".into());
        }
        self.start(sample_rate, channels, samples)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn start(&mut self, sample_rate: u32, channels: u16, samples: &[i16]) -> Result<(), String> {
        let path =
            std::env::temp_dir().join(format!("glasnost-preview-{}.wav", std::process::id()));
        std::fs::write(&path, wav_bytes(sample_rate, channels, samples))
            .map_err(|e| format!("could not write the preview file: {}", e.kind()))?;
        self.wav = Some(path.clone());
        for player in ["pw-play", "paplay", "aplay", "afplay"] {
            let spawned = std::process::Command::new(player)
                .arg(&path)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn();
            if let Ok(child) = spawned {
                self.child = Some(child);
                return Ok(());
            }
        }
        Err("no command-line audio player found (pw-play, paplay, aplay or afplay)".into())
    }

    #[cfg(target_arch = "wasm32")]
    fn start(&mut self, sample_rate: u32, channels: u16, samples: &[i16]) -> Result<(), String> {
        let js = |e: wasm_bindgen::JsValue| format!("{e:?}");
        if self.context.is_none() {
            self.context = Some(web_sys::AudioContext::new().map_err(js)?);
        }
        let Some(ctx) = &self.context else {
            return Err("no audio context".into());
        };
        let ch = usize::from(channels);
        let frames = samples.len() / ch;
        if frames == 0 {
            return Err("sound is empty".into());
        }
        let buffer = ctx
            .create_buffer(u32::from(channels), frames as u32, sample_rate as f32)
            .map_err(js)?;
        for c in 0..ch {
            let data: Vec<f32> = samples
                .iter()
                .skip(c)
                .step_by(ch)
                .map(|&s| f32::from(s) / 32768.0)
                .collect();
            buffer.copy_to_channel(&data, c as i32).map_err(js)?;
        }
        let source = ctx.create_buffer_source().map_err(js)?;
        source.set_buffer(Some(&buffer));
        source
            .connect_with_audio_node(&ctx.destination())
            .map_err(js)?;
        // Browsers suspend contexts created before a user gesture.
        let _ = ctx.resume();
        #[allow(deprecated)]
        source.start().map_err(js)?;
        self.source = Some(source);
        Ok(())
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wav_header_is_canonical() {
        let wav = wav_bytes(22050, 2, &[1, -1, 2, -2]);
        assert_eq!(&wav[..4], b"RIFF");
        assert_eq!(u32::from_le_bytes(wav[4..8].try_into().unwrap()), 36 + 8);
        assert_eq!(&wav[8..16], b"WAVEfmt ");
        assert_eq!(u16::from_le_bytes([wav[22], wav[23]]), 2);
        assert_eq!(u32::from_le_bytes(wav[24..28].try_into().unwrap()), 22050);
        assert_eq!(
            u32::from_le_bytes(wav[28..32].try_into().unwrap()),
            22050 * 4
        );
        assert_eq!(wav.len(), 44 + 8);
        assert_eq!(&wav[44..46], &1i16.to_le_bytes());
    }
}
