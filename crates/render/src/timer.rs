//! GPU frame time from timestamp queries, for the performance benchmark
//! (M8 step C4). Two timestamps bracket a frame's commands in its encoder;
//! the result is read back a frame or more later, without blocking. Only
//! devices opened with [`TIMER_FEATURES`] can make one: the native Vulkan,
//! Metal and DX12 backends and OpenGL with `GL_ARB_timer_query`, never
//! WebGL2. Our own tooling; nothing here changes what is drawn.

use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

/// The features a [`GpuTimer`] needs.
pub const TIMER_FEATURES: wgpu::Features =
    wgpu::Features::TIMESTAMP_QUERY.union(wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS);

/// Readback buffers in flight; a frame finds none free only when the GPU
/// is this many frames behind, and is then not timed.
// nocite: our own tooling
const SLOTS: usize = 4;

const FREE: u8 = 0;
const PENDING: u8 = 1;
const READY: u8 = 2;
const FAILED: u8 = 3;

struct Slot {
    buffer: wgpu::Buffer,
    state: Arc<AtomicU8>,
    frame: u64,
}

/// Times frames on the GPU.
pub struct GpuTimer {
    set: wgpu::QuerySet,
    resolve: wgpu::Buffer,
    slots: Vec<Slot>,
    /// The slot the frame being recorded writes, if any was free.
    active: Option<usize>,
    /// Nanoseconds per timestamp unit.
    period: f64,
}

impl GpuTimer {
    /// A timer on `device`, when it was opened with [`TIMER_FEATURES`].
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Option<Self> {
        if !device.features().contains(TIMER_FEATURES) {
            return None;
        }
        let set = device.create_query_set(&wgpu::QuerySetDescriptor {
            label: Some("frame timer"),
            ty: wgpu::QueryType::Timestamp,
            count: 2,
        });
        let resolve = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("frame timer resolve"),
            size: 16,
            usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let slots = (0..SLOTS)
            .map(|_| Slot {
                buffer: device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("frame timer readback"),
                    size: 16,
                    usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
                state: Arc::new(AtomicU8::new(FREE)),
                frame: 0,
            })
            .collect();
        Some(Self {
            set,
            resolve,
            slots,
            active: None,
            period: f64::from(queue.get_timestamp_period()),
        })
    }

    /// Writes the frame's first timestamp, when a readback slot is free.
    pub fn begin(&mut self, encoder: &mut wgpu::CommandEncoder) {
        self.active = self
            .slots
            .iter()
            .position(|s| s.state.load(Ordering::Acquire) == FREE);
        if self.active.is_some() {
            encoder.write_timestamp(&self.set, 0);
        }
    }

    /// Writes the frame's last timestamp and copies both to the readback
    /// slot.
    pub fn end(&mut self, encoder: &mut wgpu::CommandEncoder) {
        let Some(i) = self.active else {
            return;
        };
        encoder.write_timestamp(&self.set, 1);
        encoder.resolve_query_set(&self.set, 0..2, &self.resolve, 0);
        encoder.copy_buffer_to_buffer(&self.resolve, 0, &self.slots[i].buffer, 0, 16);
    }

    /// After the frame's submission: asks for the readback of frame
    /// `frame`.
    pub fn submitted(&mut self, frame: u64) {
        let Some(i) = self.active.take() else {
            return;
        };
        let slot = &mut self.slots[i];
        slot.frame = frame;
        slot.state.store(PENDING, Ordering::Release);
        let state = slot.state.clone();
        slot.buffer.map_async(wgpu::MapMode::Read, .., move |r| {
            state.store(if r.is_ok() { READY } else { FAILED }, Ordering::Release);
        });
    }

    /// The frames whose times have come back since the last call, with
    /// their GPU milliseconds (polls `device` without blocking).
    pub fn collect(&mut self, device: &wgpu::Device) -> Vec<(u64, f64)> {
        let _ = device.poll(wgpu::PollType::Poll);
        let mut out = Vec::new();
        for s in &mut self.slots {
            match s.state.load(Ordering::Acquire) {
                READY => {
                    let ticks = s.buffer.get_mapped_range(..).ok().map(|v| {
                        let t: &[u64] = bytemuck::cast_slice(&v);
                        t[1].saturating_sub(t[0])
                    });
                    s.buffer.unmap();
                    if let Some(t) = ticks {
                        out.push((s.frame, t as f64 * self.period / 1.0e6));
                    }
                    s.state.store(FREE, Ordering::Release);
                }
                FAILED => s.state.store(FREE, Ordering::Release),
                _ => {}
            }
        }
        out
    }
}
