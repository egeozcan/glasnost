//! A bounded single-producer, single-consumer ring of stereo frames: the
//! bridge between the client's frame loop, which renders the mix
//! ([`Ring::push`]), and a device's callback thread, which plays it
//! ([`Ring::pop_into`]).
//!
//! Lock-free: each frame is one `AtomicU32` (left and right 16-bit
//! samples), and the two positions are counters published with release
//! and read with acquire ordering, so the callback never waits for the
//! frame loop. A full ring refuses the excess; an empty one gives silence.

use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

/// The ring. One thread may push and one other thread may pop at a time.
pub struct Ring {
    frames: Box<[AtomicU32]>,
    /// Frames pushed so far (written by the producer only).
    head: AtomicUsize,
    /// Frames popped so far (written by the consumer only).
    tail: AtomicUsize,
}

fn pack(l: i16, r: i16) -> u32 {
    u32::from(l as u16) | (u32::from(r as u16) << 16)
}

fn unpack(f: u32) -> (i16, i16) {
    (f as u16 as i16, (f >> 16) as u16 as i16)
}

impl Ring {
    /// An empty ring holding at most `capacity` frames (at least 1).
    pub fn new(capacity: usize) -> Self {
        let capacity = capacity.max(1);
        Self {
            frames: (0..capacity).map(|_| AtomicU32::new(0)).collect(),
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
        }
    }

    /// The most frames the ring holds.
    pub fn capacity(&self) -> usize {
        self.frames.len()
    }

    /// Frames waiting to be played.
    pub fn len(&self) -> usize {
        let tail = self.tail.load(Ordering::Acquire);
        let head = self.head.load(Ordering::Acquire);
        head.wrapping_sub(tail)
    }

    /// Whether no frame waits.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Producer: queues the interleaved stereo frames of `samples` (an odd
    /// last sample is ignored) as far as they fit, and returns how many
    /// frames were queued. The rest is dropped, so the ring never holds
    /// more than its capacity.
    pub fn push(&self, samples: &[i16]) -> usize {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);
        let free = self.capacity() - head.wrapping_sub(tail);
        let n = free.min(samples.len() / 2);
        let cap = self.capacity();
        for (i, f) in samples.as_chunks::<2>().0.iter().take(n).enumerate() {
            self.frames[head.wrapping_add(i) % cap].store(pack(f[0], f[1]), Ordering::Relaxed);
        }
        self.head.store(head.wrapping_add(n), Ordering::Release);
        n
    }

    /// Consumer: fills `out` with interleaved stereo frames (an odd last
    /// sample is set to 0), silence where the ring runs dry, and returns
    /// how many frames came from the ring.
    pub fn pop_into(&self, out: &mut [i16]) -> usize {
        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Acquire);
        let avail = head.wrapping_sub(tail);
        let cap = self.capacity();
        let n = avail.min(out.len() / 2);
        let (played, rest) = out.split_at_mut(2 * n);
        for (i, f) in played.as_chunks_mut::<2>().0.iter_mut().enumerate() {
            let v = self.frames[tail.wrapping_add(i) % cap].load(Ordering::Relaxed);
            (f[0], f[1]) = unpack(v);
        }
        rest.fill(0);
        self.tail.store(tail.wrapping_add(n), Ordering::Release);
        n
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames_come_out_in_order() {
        let r = Ring::new(8);
        assert!(r.is_empty());
        assert_eq!(r.push(&[1, -1, 2, -2, i16::MIN, i16::MAX]), 3);
        assert_eq!(r.len(), 3);
        let mut out = [9; 4];
        assert_eq!(r.pop_into(&mut out), 2);
        assert_eq!(out, [1, -1, 2, -2]);
        let mut out = [9; 2];
        assert_eq!(r.pop_into(&mut out), 1);
        assert_eq!(out, [i16::MIN, i16::MAX]);
        assert!(r.is_empty());
    }

    #[test]
    fn an_underrun_plays_silence() {
        let r = Ring::new(8);
        r.push(&[5, 6]);
        let mut out = [9; 6];
        assert_eq!(r.pop_into(&mut out), 1);
        assert_eq!(out, [5, 6, 0, 0, 0, 0]);
        let mut out = [9; 4];
        assert_eq!(r.pop_into(&mut out), 0);
        assert_eq!(out, [0; 4]);
    }

    #[test]
    fn an_overrun_is_bounded() {
        let r = Ring::new(4);
        let samples: Vec<i16> = (0..20).collect();
        assert_eq!(r.push(&samples), 4);
        assert_eq!(r.len(), 4);
        assert_eq!(r.push(&[100, 101]), 0, "a full ring takes nothing");
        let mut out = [0; 8];
        assert_eq!(r.pop_into(&mut out), 4);
        assert_eq!(
            out,
            [0, 1, 2, 3, 4, 5, 6, 7],
            "the excess was dropped, not the start"
        );
    }

    #[test]
    fn it_wraps_around_many_times() {
        let r = Ring::new(3);
        let mut next = 0i16;
        let mut expect = 0i16;
        for _ in 0..50 {
            let batch = [next, next + 1, next + 2, next + 3];
            let n = r.push(&batch);
            next += 2 * n as i16;
            let mut out = [0; 2];
            if r.pop_into(&mut out) == 1 {
                assert_eq!(out, [expect, expect + 1]);
                expect += 2;
            }
        }
        assert!(r.len() <= r.capacity());
    }

    #[test]
    fn an_odd_tail_is_ignored_or_silenced() {
        let r = Ring::new(4);
        assert_eq!(r.push(&[1, 2, 3]), 1);
        let mut out = [9; 3];
        assert_eq!(r.pop_into(&mut out), 1);
        assert_eq!(out, [1, 2, 0]);
    }

    #[test]
    fn a_producer_and_a_consumer_thread_agree() {
        let r = std::sync::Arc::new(Ring::new(64));
        let total = 10_000i32;
        let consumer = {
            let r = r.clone();
            std::thread::spawn(move || {
                let mut got = Vec::new();
                let mut out = [0i16; 14];
                while got.len() < total as usize {
                    let n = r.pop_into(&mut out);
                    got.extend(out.chunks(2).take(n).map(|f| (f[0], f[1])));
                    std::thread::yield_now();
                }
                got
            })
        };
        let mut i = 0i32;
        while i < total {
            let batch: Vec<i16> = (i..(i + 10).min(total))
                .flat_map(|k| [k as i16, -(k as i16)])
                .collect();
            i += r.push(&batch) as i32;
            std::thread::yield_now();
        }
        let got = consumer.join().unwrap();
        for (k, f) in got.iter().enumerate() {
            assert_eq!(*f, (k as i16, -(k as i16)));
        }
    }
}
