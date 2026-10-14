//! Decoded audio: interleaved signed 16-bit PCM, with a WAV exporter.
//!
//! Every audio decoder in this crate ([`crate::aud`], [`crate::idx`])
//! produces a [`Pcm`]. [`Pcm::to_wav`] writes a canonical 44-byte-header
//! RIFF/WAVE file for playback and export.

use crate::bytes::{PutLe, Reader};
use crate::error::{Error, Result};
use crate::limits::MAX_DECOMPRESSED;
use crate::synth::SynthRng;

/// Interleaved signed 16-bit PCM audio.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Pcm {
    /// Frames per second.
    pub sample_rate: u32,
    /// Interleaved channels (1 mono, 2 stereo).
    pub channels: u16,
    /// Samples, `channels` per frame, in frame order.
    pub samples: Vec<i16>,
}

/// Length of the header [`Pcm::to_wav`] writes.
pub const WAV_HEADER_LEN: usize = 44;

impl Pcm {
    /// Number of whole frames (0 if `channels` is 0).
    pub fn frames(&self) -> usize {
        self.samples
            .len()
            .checked_div(usize::from(self.channels))
            .unwrap_or(0)
    }

    /// A canonical 16-bit PCM RIFF/WAVE file holding these samples' whole
    /// frames.
    ///
    /// Size fields saturate at `u32::MAX`; the decoders in this crate never
    /// produce audio that large (see [`crate::limits::MAX_DECOMPRESSED`]).
    pub fn to_wav(&self) -> Vec<u8> {
        let sat = |n: u64| u32::try_from(n).unwrap_or(u32::MAX);
        // Only whole frames: a trailing partial frame is dropped so the data
        // chunk is a multiple of the block alignment.
        let samples = &self.samples[..self.frames() * usize::from(self.channels)];
        let data_len = (samples.len() as u64).saturating_mul(2);
        let block_align = u64::from(self.channels).saturating_mul(2);
        let mut out =
            Vec::with_capacity(WAV_HEADER_LEN.saturating_add(samples.len().saturating_mul(2)));
        out.extend_from_slice(b"RIFF");
        out.put_u32(sat(data_len.saturating_add(WAV_HEADER_LEN as u64 - 8)));
        out.extend_from_slice(b"WAVE");
        out.extend_from_slice(b"fmt ");
        out.put_u32(16);
        out.put_u16(1); // PCM
        out.put_u16(self.channels);
        out.put_u32(self.sample_rate);
        out.put_u32(sat(u64::from(self.sample_rate).saturating_mul(block_align)));
        out.put_u16(u16::try_from(block_align).unwrap_or(u16::MAX));
        out.put_u16(16);
        out.extend_from_slice(b"data");
        out.put_u32(sat(data_len));
        for &s in samples {
            out.put_i16(s);
        }
        out
    }

    /// Reads a plain PCM RIFF/WAVE file: format code 1, 8-bit unsigned or
    /// 16-bit signed samples, one or two channels. Chunks other than `fmt `
    /// and `data` are skipped (each is padded to an even length, as RIFF
    /// requires); a `data` chunk that claims more bytes than the file holds
    /// is cut to what is there, and a partial last frame is dropped.
    /// Compressed WAVE formats are [`Error::Unsupported`].
    pub fn from_wav(data: &[u8]) -> Result<Self> {
        let mut r = Reader::new(data);
        if r.array::<4>()? != *b"RIFF" {
            return Err(Error::Invalid("not a RIFF file"));
        }
        r.skip(4)?;
        if r.array::<4>()? != *b"WAVE" {
            return Err(Error::Invalid("not a WAVE file"));
        }
        let mut format: Option<(u16, u32, u16)> = None;
        while r.remaining() >= 8 {
            let id = r.array::<4>()?;
            let len = r.u32()? as usize;
            let avail = len.min(r.remaining());
            let body = r.bytes(avail)?;
            match &id {
                b"fmt " => {
                    let mut f = Reader::new(body);
                    let code = f.u16()?;
                    let channels = f.u16()?;
                    let rate = f.u32()?;
                    f.skip(6)?;
                    let bits = f.u16()?;
                    if code != 1 {
                        return Err(Error::Unsupported("compressed WAVE format"));
                    }
                    if !(1..=2).contains(&channels) || rate == 0 {
                        return Err(Error::Invalid("WAVE channel count or sample rate"));
                    }
                    if bits != 8 && bits != 16 {
                        return Err(Error::Unsupported("WAVE sample width"));
                    }
                    format = Some((channels, rate, bits));
                }
                b"data" => {
                    let (channels, sample_rate, bits) =
                        format.ok_or(Error::Invalid("WAVE data before its format"))?;
                    if body.len() > MAX_DECOMPRESSED {
                        return Err(Error::TooLarge("WAVE data"));
                    }
                    let mut samples: Vec<i16> = if bits == 8 {
                        body.iter().map(|&b| (i16::from(b) - 128) << 8).collect()
                    } else {
                        body.chunks_exact(2)
                            .map(|c| i16::from_le_bytes([c[0], c[1]]))
                            .collect()
                    };
                    let whole = samples.len() - samples.len() % usize::from(channels);
                    samples.truncate(whole);
                    return Ok(Self {
                        sample_rate,
                        channels,
                        samples,
                    });
                }
                _ => {}
            }
            if len % 2 == 1 && r.remaining() > 0 {
                r.skip(1)?;
            }
        }
        Err(Error::Invalid("WAVE file without a data chunk"))
    }

    /// A synthetic test signal of our own: a triangle wave of `period`
    /// frames and peak `amplitude`, with a little seeded noise added. In
    /// stereo the right channel runs at half the period. Used by fixture
    /// writers and tests; it imitates nothing.
    ///
    /// # Panics
    /// If `period` is below 4 or `channels` is 0.
    pub fn synthetic_tone(
        sample_rate: u32,
        channels: u16,
        frames: usize,
        period: u32,
        amplitude: i16,
        seed: u64,
    ) -> Self {
        assert!(period >= 4 && channels > 0, "bad synthetic tone parameters");
        let mut rng = SynthRng::new(seed);
        let amp = i64::from(amplitude);
        let tri = |t: u32, p: u32| -> i64 {
            // Rises from -amp to amp over half a period, then falls back.
            let half = p / 2;
            let ph = t % p;
            let up = if ph < half { ph } else { p - ph };
            -amp + (2 * amp * i64::from(up)) / i64::from(half)
        };
        let mut samples = Vec::with_capacity(frames * usize::from(channels));
        for f in 0..frames {
            let t = (f % (1 << 20)) as u32;
            for c in 0..channels {
                let p = if c % 2 == 0 {
                    period
                } else {
                    (period / 2).max(4)
                };
                let noise = (rng.next_u64() % 65) as i64 - 32;
                let v = tri(t, p) + noise;
                samples.push(v.clamp(i64::from(i16::MIN), i64::from(i16::MAX)) as i16);
            }
        }
        Self {
            sample_rate,
            channels,
            samples,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wav_header_is_canonical() {
        let pcm = Pcm {
            sample_rate: 22050,
            channels: 2,
            samples: vec![1, -1, 256, -256],
        };
        let wav = pcm.to_wav();
        assert_eq!(wav.len(), WAV_HEADER_LEN + 8);
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(u32::from_le_bytes(wav[4..8].try_into().unwrap()), 36 + 8);
        assert_eq!(&wav[8..16], b"WAVEfmt ");
        assert_eq!(u16::from_le_bytes([wav[22], wav[23]]), 2);
        // Byte rate = 22050 frames × 2 channels × 2 bytes.
        assert_eq!(
            u32::from_le_bytes(wav[28..32].try_into().unwrap()),
            22050 * 4
        );
        assert_eq!(u16::from_le_bytes([wav[32], wav[33]]), 4);
        assert_eq!(&wav[36..40], b"data");
        assert_eq!(u32::from_le_bytes(wav[40..44].try_into().unwrap()), 8);
        assert_eq!(&wav[44..], &[1, 0, 0xFF, 0xFF, 0, 1, 0, 0xFF]);
    }

    #[test]
    fn frames_handle_odd_shapes() {
        let mut pcm = Pcm {
            sample_rate: 8000,
            channels: 2,
            samples: vec![0; 5],
        };
        assert_eq!(pcm.frames(), 2);
        pcm.channels = 0;
        assert_eq!(pcm.frames(), 0);
        // Hostile header values do not overflow the exporter.
        pcm.sample_rate = u32::MAX;
        pcm.channels = u16::MAX;
        // u16::MAX channels: zero whole frames, header only.
        assert_eq!(pcm.to_wav().len(), WAV_HEADER_LEN);
        // A partial last frame is not exported.
        pcm.channels = 2;
        assert_eq!(pcm.to_wav().len(), WAV_HEADER_LEN + 8);
    }

    #[test]
    fn wav_round_trips() {
        for channels in [1, 2] {
            let pcm = Pcm::synthetic_tone(22050, channels, 300, 40, 9000, 5);
            assert_eq!(Pcm::from_wav(&pcm.to_wav()).unwrap(), pcm);
        }
    }

    #[test]
    fn wav_reader_handles_8_bit_extra_chunks_and_truncation() {
        let mut wav = Vec::new();
        wav.extend_from_slice(b"RIFF\0\0\0\0WAVE");
        // An odd-length chunk of our own before the format, padded.
        wav.extend_from_slice(b"junk");
        wav.put_u32(3);
        wav.extend_from_slice(&[1, 2, 3, 0]);
        wav.extend_from_slice(b"fmt ");
        wav.put_u32(16);
        wav.put_u16(1);
        wav.put_u16(1);
        wav.put_u32(8000);
        wav.put_u32(8000);
        wav.put_u16(1);
        wav.put_u16(8);
        wav.extend_from_slice(b"data");
        wav.put_u32(100); // claims more than is there
        wav.extend_from_slice(&[128, 255, 0]);
        let pcm = Pcm::from_wav(&wav).unwrap();
        assert_eq!(pcm.sample_rate, 8000);
        assert_eq!(pcm.samples, vec![0, 127 << 8, -128 << 8]);
        // Compressed formats and garbage are errors, not panics.
        let mut adpcm = wav.clone();
        adpcm[32] = 0x11;
        assert!(matches!(
            Pcm::from_wav(&adpcm),
            Err(Error::Unsupported(_))
        ));
        assert!(Pcm::from_wav(b"RIFF").is_err());
        assert!(Pcm::from_wav(&wav[..30]).is_err());
    }

    #[test]
    fn synthetic_tone_is_deterministic() {
        let a = Pcm::synthetic_tone(11025, 2, 100, 40, 8000, 3);
        assert_eq!(a, Pcm::synthetic_tone(11025, 2, 100, 40, 8000, 3));
        assert_eq!(a.frames(), 100);
        assert!(a.samples.iter().any(|&s| s > 7000));
        assert!(a.samples.iter().any(|&s| s < -7000));
    }
}
