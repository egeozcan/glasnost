//! The 2D interface batch: palette-indexed sprites with house-colour
//! palette rows, glyphs, solid and translucent rectangles and RGBA images,
//! drawn as instanced quads in screen pixels (PLAN.md 4.4, WebGL2 core
//! rules: vertices from the vertex index, base vertex 0, no first-instance
//! offsets: each run binds its own slice of the instance buffer).
//!
//! Clipping happens on the CPU ([`clip`]), so every drawn pixel is exact.
//! [`reference`] is the CPU model of the shader that the offscreen tests
//! compare against.

use std::collections::BTreeMap;

use bytemuck::{Pod, Zeroable};

/// WGSL source of the batch pipeline.
pub const SHADER_SOURCE: &str = include_str!("batch2d.wgsl");

/// Mode: a solid colour.
pub const MODE_SOLID: u32 = 0;
/// Mode: an atlas index through a palette row; index 0 is transparent.
pub const MODE_SPRITE: u32 = 1;
/// Mode: an atlas mask; index 0 is transparent, any other the colour.
pub const MODE_GLYPH: u32 = 2;
/// Mode: an RGBA image.
pub const MODE_IMAGE: u32 = 3;

/// The atlas's side in texels (the WebGL2 floor of the texture size).
pub const ATLAS_SIZE: u32 = 2048;

/// One quad: GPU instance data.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Pod, Zeroable)]
pub struct Quad {
    /// Destination x, y, width, height in screen pixels.
    pub dst: [i32; 4],
    /// Source x, y, width, height in the atlas or image.
    pub src: [i32; 4],
    /// Colour, `r | g << 8 | b << 16 | a << 24`.
    pub rgba: u32,
    /// Mode in the low byte, palette row above.
    pub mode_row: u32,
    /// The image (mode 3); not read by the shader.
    pub image: u32,
    /// Padding.
    pub pad: u32,
}

/// Packs an RGBA colour.
pub const fn pack(c: [u8; 4]) -> u32 {
    c[0] as u32 | (c[1] as u32) << 8 | (c[2] as u32) << 16 | (c[3] as u32) << 24
}

impl Quad {
    /// A solid rectangle.
    pub fn solid(dst: [i32; 4], rgba: [u8; 4]) -> Self {
        Self {
            dst,
            src: [0, 0, 1, 1],
            rgba: pack(rgba),
            mode_row: MODE_SOLID,
            ..Self::default()
        }
    }

    /// A sprite region of the atlas, 1:1, drawn with palette `row`.
    pub fn sprite(x: i32, y: i32, region: [i32; 4], row: u32) -> Self {
        Self {
            dst: [x, y, region[2], region[3]],
            src: region,
            mode_row: MODE_SPRITE | row << 8,
            ..Self::default()
        }
    }

    /// A glyph region of the atlas, 1:1, in `rgba`.
    pub fn glyph(x: i32, y: i32, region: [i32; 4], rgba: [u8; 4]) -> Self {
        Self {
            dst: [x, y, region[2], region[3]],
            src: region,
            rgba: pack(rgba),
            mode_row: MODE_GLYPH,
            ..Self::default()
        }
    }

    /// A whole image of `size` stretched to `dst`.
    pub fn image(image: u32, size: (i32, i32), dst: [i32; 4]) -> Self {
        Self {
            dst,
            src: [0, 0, size.0, size.1],
            mode_row: MODE_IMAGE,
            image,
            ..Self::default()
        }
    }

    fn mode(&self) -> u32 {
        self.mode_row & 255
    }
}

/// Clips a 1:1 quad (solid, sprite or glyph) to `clip` by moving its
/// source and destination together; `None` when nothing is left. Image
/// quads are returned unchanged when they meet the clip.
pub fn clip(q: Quad, clip: [i32; 4]) -> Option<Quad> {
    let [x, y, w, h] = q.dst;
    let x0 = x.max(clip[0]);
    let y0 = y.max(clip[1]);
    let x1 = (x + w).min(clip[0] + clip[2]);
    let y1 = (y + h).min(clip[1] + clip[3]);
    if x1 <= x0 || y1 <= y0 {
        return None;
    }
    if q.mode() == MODE_IMAGE || q.mode() == MODE_SOLID {
        let mut out = q;
        if q.mode() == MODE_SOLID {
            out.dst = [x0, y0, x1 - x0, y1 - y0];
        }
        return Some(out);
    }
    let mut out = q;
    out.dst = [x0, y0, x1 - x0, y1 - y0];
    out.src = [q.src[0] + x0 - x, q.src[1] + y0 - y, x1 - x0, y1 - y0];
    Some(out)
}

/// The quads of a line between two pixels (both included): one 1-pixel
/// rectangle per run of a Bresenham walk.
pub fn line(from: (i32, i32), to: (i32, i32), rgba: [u8; 4]) -> Vec<Quad> {
    let (mut x, mut y) = from;
    let dx = (to.0 - x).abs();
    let dy = -(to.1 - y).abs();
    let sx = if x < to.0 { 1 } else { -1 };
    let sy = if y < to.1 { 1 } else { -1 };
    let mut err = dx + dy;
    let mut out = Vec::new();
    loop {
        out.push(Quad::solid([x, y, 1, 1], rgba));
        if (x, y) == to {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
    out
}

/// The CPU side of the index atlas: a shelf packer over an R8 image.
#[derive(Clone, Debug)]
pub struct Atlas {
    size: u32,
    pixels: Vec<u8>,
    x: u32,
    y: u32,
    shelf: u32,
    dirty: bool,
}

impl Default for Atlas {
    fn default() -> Self {
        Self::new(ATLAS_SIZE)
    }
}

impl Atlas {
    /// An empty atlas of `size` × `size`.
    pub fn new(size: u32) -> Self {
        Self {
            size,
            pixels: vec![0; (size * size) as usize],
            x: 0,
            y: 0,
            shelf: 0,
            dirty: true,
        }
    }

    /// The side in texels.
    pub fn size(&self) -> u32 {
        self.size
    }

    /// The index at a texel.
    pub fn get(&self, x: i32, y: i32) -> u8 {
        if x < 0 || y < 0 || x as u32 >= self.size || y as u32 >= self.size {
            return 0;
        }
        self.pixels[(y as u32 * self.size + x as u32) as usize]
    }

    /// Adds a `w` × `h` image of indices; its region, or `None` when the
    /// atlas is full.
    pub fn add(&mut self, w: u32, h: u32, pixels: &[u8]) -> Option<[i32; 4]> {
        if w == 0 || h == 0 || w > self.size || pixels.len() < (w * h) as usize {
            return None;
        }
        if self.x + w > self.size {
            self.x = 0;
            self.y += self.shelf + 1;
            self.shelf = 0;
        }
        if self.y + h > self.size {
            return None;
        }
        for row in 0..h {
            let at = ((self.y + row) * self.size + self.x) as usize;
            let from = (row * w) as usize;
            self.pixels[at..at + w as usize].copy_from_slice(&pixels[from..from + w as usize]);
        }
        let r = [self.x as i32, self.y as i32, w as i32, h as i32];
        self.x += w + 1;
        self.shelf = self.shelf.max(h);
        self.dirty = true;
        Some(r)
    }
}

/// An RGBA image for [`reference`].
#[derive(Clone, Debug)]
pub struct CpuImage {
    /// Width.
    pub w: i32,
    /// Height.
    pub h: i32,
    /// Tightly packed RGBA, top row first.
    pub rgba: Vec<u8>,
}

fn texel(q: &Quad, lx: i32, ly: i32) -> (i32, i32) {
    let [sx, sy, sw, sh] = q.src;
    let dw = q.dst[2].max(1);
    let dh = q.dst[3].max(1);
    let tx = ((2 * lx + 1) * sw / (2 * dw)).clamp(0, sw - 1);
    let ty = ((2 * ly + 1) * sh / (2 * dh)).clamp(0, sh - 1);
    (sx + tx, sy + ty)
}

fn blend(dst: &mut [u8], c: [u8; 4]) {
    let a = u32::from(c[3]);
    for k in 0..3 {
        let s = u32::from(c[k]);
        let d = u32::from(dst[k]);
        dst[k] = ((s * a + d * (255 - a) + 127) / 255) as u8;
    }
    let da = u32::from(dst[3]);
    dst[3] = ((a * 255 + da * (255 - a) + 127) / 255) as u8;
}

/// The CPU model of the shader: draws `quads` in order over `width` ×
/// `height` opaque black, with source-alpha blending.
pub fn reference(
    quads: &[Quad],
    atlas: &Atlas,
    palettes: &[[u8; 1024]],
    images: &BTreeMap<u32, CpuImage>,
    width: i32,
    height: i32,
) -> Vec<u8> {
    let mut out: Vec<u8> = std::iter::repeat_n([0u8, 0, 0, 255], (width * height) as usize)
        .flatten()
        .collect();
    for q in quads {
        let [x, y, w, h] = q.dst;
        for py in y.max(0)..(y + h).min(height) {
            for px in x.max(0)..(x + w).min(width) {
                let (tx, ty) = texel(q, px - x, py - y);
                let c = match q.mode() {
                    MODE_SOLID => Some(q.rgba.to_le_bytes()),
                    MODE_IMAGE => images.get(&q.image).map(|im| {
                        let at = ((ty * im.w + tx) * 4) as usize;
                        [
                            im.rgba[at],
                            im.rgba[at + 1],
                            im.rgba[at + 2],
                            im.rgba[at + 3],
                        ]
                    }),
                    m => {
                        let i = atlas.get(tx, ty);
                        if i == 0 {
                            None
                        } else if m == MODE_GLYPH {
                            Some(q.rgba.to_le_bytes())
                        } else {
                            let row = (q.mode_row >> 8) as usize;
                            palettes.get(row).map(|p| {
                                let at = usize::from(i) * 4;
                                [p[at], p[at + 1], p[at + 2], p[at + 3]]
                            })
                        }
                    }
                };
                if let Some(c) = c {
                    let at = ((py * width + px) * 4) as usize;
                    blend(&mut out[at..at + 4], c);
                }
            }
        }
    }
    out
}

struct GpuImage {
    /// The batch's own texture; `None` for a view the caller owns.
    texture: Option<wgpu::Texture>,
    size: (u32, u32),
    group: wgpu::BindGroup,
}

/// GPU resources of the batch.
pub struct Batch2d {
    pipeline: wgpu::RenderPipeline,
    layout0: wgpu::BindGroupLayout,
    layout1: wgpu::BindGroupLayout,
    screen: wgpu::Buffer,
    atlas: wgpu::Texture,
    palettes: wgpu::Texture,
    group0: wgpu::BindGroup,
    dummy: wgpu::BindGroup,
    images: BTreeMap<u32, GpuImage>,
    instances: wgpu::Buffer,
    capacity: u64,
    runs: Vec<(Option<u32>, u32, u32)>,
    stats: BatchStats,
}

/// What the batch uploaded and drew: the counters of the performance
/// benchmark (M8 step C4). [`Batch2d::take_stats`] returns them and starts
/// counting again.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BatchStats {
    /// Quads prepared.
    pub quads: usize,
    /// Draw calls of the prepared quads.
    pub draws: usize,
    /// Texture and buffer writes.
    pub uploads: usize,
    /// Bytes those writes moved.
    pub upload_bytes: u64,
}

fn texture(
    device: &wgpu::Device,
    label: &str,
    w: u32,
    h: u32,
    format: wgpu::TextureFormat,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    })
}

fn write(queue: &wgpu::Queue, t: &wgpu::Texture, w: u32, h: u32, bpp: u32, data: &[u8]) {
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: t,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(w * bpp),
            rows_per_image: Some(h),
        },
        wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
    );
}

fn view(t: &wgpu::Texture) -> wgpu::TextureView {
    t.create_view(&wgpu::TextureViewDescriptor::default())
}

const INSTANCE_ATTRIBUTES: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![
    0 => Sint32x4,
    1 => Sint32x4,
    2 => Uint32,
    3 => Uint32,
];

impl Batch2d {
    /// Builds the pipeline for a non-sRGB `format` (colours and palettes
    /// are stored bytes, written unchanged).
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let size = ATLAS_SIZE.min(device.limits().max_texture_dimension_2d);
        let tex_entry = |binding, sample_type| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type,
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let float = wgpu::TextureSampleType::Float { filterable: false };
        let layout0 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("batch2d"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(16),
                    },
                    count: None,
                },
                tex_entry(1, wgpu::TextureSampleType::Uint),
                tex_entry(2, float),
            ],
        });
        let layout1 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("batch2d image"),
            entries: &[tex_entry(0, float)],
        });
        let screen = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("batch2d screen"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let atlas = texture(
            device,
            "batch2d atlas",
            size,
            size,
            wgpu::TextureFormat::R8Uint,
        );
        let palettes = texture(
            device,
            "batch2d palettes",
            256,
            1,
            wgpu::TextureFormat::Rgba8Unorm,
        );
        let group0 = Self::group0(device, &layout0, &screen, &atlas, &palettes);
        let dummy_t = texture(
            device,
            "batch2d dummy",
            1,
            1,
            wgpu::TextureFormat::Rgba8Unorm,
        );
        let dummy = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("batch2d dummy"),
            layout: &layout1,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view(&dummy_t)),
            }],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("batch2d"),
            source: wgpu::ShaderSource::Wgsl(SHADER_SOURCE.into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("batch2d"),
            bind_group_layouts: &[Some(&layout0), Some(&layout1)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("batch2d"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Quad>() as u64,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &INSTANCE_ATTRIBUTES,
                })],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let capacity = 1024;
        let instances = Self::instance_buffer(device, capacity);
        // The dummy texture stays alive through its bind group.
        Self {
            pipeline,
            layout0,
            layout1,
            screen,
            atlas,
            palettes,
            group0,
            dummy,
            images: BTreeMap::new(),
            instances,
            capacity,
            runs: Vec::new(),
            stats: BatchStats::default(),
        }
    }

    fn instance_buffer(device: &wgpu::Device, n: u64) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("batch2d instances"),
            size: n * std::mem::size_of::<Quad>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    fn group0(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        screen: &wgpu::Buffer,
        atlas: &wgpu::Texture,
        palettes: &wgpu::Texture,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("batch2d"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: screen.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&view(atlas)),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&view(palettes)),
                },
            ],
        })
    }

    /// The counters since the last call, and a fresh start.
    pub fn take_stats(&mut self) -> BatchStats {
        std::mem::take(&mut self.stats)
    }

    /// Uploads the atlas when it changed since the last upload.
    pub fn upload_atlas(&mut self, queue: &wgpu::Queue, atlas: &mut Atlas) {
        if atlas.dirty {
            let s = atlas.size.min(self.atlas.width());
            if s == atlas.size {
                self.stats.uploads += 1;
                self.stats.upload_bytes += atlas.pixels.len() as u64;
                write(queue, &self.atlas, s, s, 1, &atlas.pixels);
            }
            atlas.dirty = false;
        }
    }

    /// Replaces the palette rows (256 RGBA entries each).
    pub fn set_palettes(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        rows: &[[u8; 1024]],
    ) {
        let n = rows.len().max(1) as u32;
        if self.palettes.height() != n {
            self.palettes = texture(
                device,
                "batch2d palettes",
                256,
                n,
                wgpu::TextureFormat::Rgba8Unorm,
            );
            self.group0 = Self::group0(
                device,
                &self.layout0,
                &self.screen,
                &self.atlas,
                &self.palettes,
            );
        }
        if !rows.is_empty() {
            let bytes: Vec<u8> = rows.iter().flatten().copied().collect();
            self.stats.uploads += 1;
            self.stats.upload_bytes += bytes.len() as u64;
            write(queue, &self.palettes, 256, n, 4, &bytes);
        }
    }

    /// Creates or updates an RGBA image (tightly packed, top row first).
    pub fn set_image(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        id: u32,
        w: u32,
        h: u32,
        rgba: &[u8],
    ) {
        if w == 0 || h == 0 {
            return;
        }
        let fresh = self
            .images
            .get(&id)
            .is_none_or(|i| i.size != (w, h) || i.texture.is_none());
        if fresh {
            let t = texture(
                device,
                "batch2d image",
                w,
                h,
                wgpu::TextureFormat::Rgba8Unorm,
            );
            let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("batch2d image"),
                layout: &self.layout1,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view(&t)),
                }],
            });
            self.images.insert(
                id,
                GpuImage {
                    texture: Some(t),
                    size: (w, h),
                    group,
                },
            );
        }
        if let Some(t) = self.images.get(&id).and_then(|i| i.texture.as_ref()) {
            self.stats.uploads += 1;
            self.stats.upload_bytes += rgba.len() as u64;
            write(queue, t, w, h, 4, rgba);
        }
    }

    /// Makes image `id` draw from `view`, a filterable RGBA texture of
    /// `size` the caller owns and renders into (a render target), in place
    /// of any image of that id.
    pub fn set_image_view(
        &mut self,
        device: &wgpu::Device,
        id: u32,
        view: &wgpu::TextureView,
        size: (u32, u32),
    ) {
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("batch2d image view"),
            layout: &self.layout1,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(view),
            }],
        });
        self.images.insert(
            id,
            GpuImage {
                texture: None,
                size,
                group,
            },
        );
    }

    /// Writes the quads of the next [`draw`](Self::draw) for a target of
    /// `size` pixels, and splits them into runs by image.
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        size: (u32, u32),
        quads: &[Quad],
    ) {
        let s: [f32; 4] = [size.0 as f32, size.1 as f32, 0.0, 0.0];
        queue.write_buffer(&self.screen, 0, bytemuck::cast_slice(&s));
        let n = quads.len() as u64;
        if n > self.capacity {
            self.capacity = n.next_power_of_two();
            self.instances = Self::instance_buffer(device, self.capacity);
        }
        if n > 0 {
            queue.write_buffer(&self.instances, 0, bytemuck::cast_slice(quads));
            self.stats.uploads += 1;
            self.stats.upload_bytes += std::mem::size_of_val(quads) as u64;
        }
        self.stats.quads += quads.len();
        self.runs.clear();
        for (i, q) in quads.iter().enumerate() {
            let img = (q.mode() == MODE_IMAGE).then_some(q.image);
            match self.runs.last_mut() {
                Some((last, _, len)) if *last == img => *len += 1,
                _ => self.runs.push((img, i as u32, 1)),
            }
        }
        self.stats.draws += self.runs.len();
    }

    /// Records the prepared quads into `pass`.
    pub fn draw(&self, pass: &mut wgpu::RenderPass<'_>) {
        if self.runs.is_empty() {
            return;
        }
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.group0, &[]);
        let stride = std::mem::size_of::<Quad>() as u64;
        for &(img, first, len) in &self.runs {
            let group = img
                .and_then(|i| self.images.get(&i))
                .map_or(&self.dummy, |i| &i.group);
            if img.is_some() && !img.is_some_and(|i| self.images.contains_key(&i)) {
                continue;
            }
            pass.set_bind_group(1, group, &[]);
            let from = u64::from(first) * stride;
            let to = from + u64::from(len) * stride;
            pass.set_vertex_buffer(0, self.instances.slice(from..to));
            pass.draw(0..6, 0..len);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quad_is_48_bytes_and_packs_colours() {
        assert_eq!(std::mem::size_of::<Quad>(), 48);
        assert_eq!(pack([1, 2, 3, 4]).to_le_bytes(), [1, 2, 3, 4]);
    }

    #[test]
    fn clipping_moves_source_with_destination() {
        let q = Quad::sprite(10, 10, [100, 200, 8, 8], 0);
        let c = clip(q, [12, 0, 100, 15]).unwrap();
        assert_eq!(c.dst, [12, 10, 6, 5]);
        assert_eq!(c.src, [102, 200, 6, 5]);
        assert!(clip(q, [0, 0, 10, 10]).is_none());
        let s = clip(Quad::solid([0, 0, 10, 10], [1, 1, 1, 1]), [5, 5, 10, 10]).unwrap();
        assert_eq!(s.dst, [5, 5, 5, 5]);
    }

    #[test]
    fn lines_cover_both_ends() {
        let l = line((0, 0), (4, 2), [9, 9, 9, 255]);
        assert_eq!(l.first().unwrap().dst, [0, 0, 1, 1]);
        assert_eq!(l.last().unwrap().dst, [4, 2, 1, 1]);
        assert_eq!(l.len(), 5);
        assert_eq!(line((3, 3), (3, 3), [0; 4]).len(), 1);
    }

    #[test]
    fn atlas_packs_shelves_and_reports_full() {
        let mut a = Atlas::new(16);
        let r = a.add(10, 4, &[7; 40]).unwrap();
        assert_eq!(r, [0, 0, 10, 4]);
        let r2 = a.add(10, 4, &[8; 40]).unwrap();
        assert_eq!(r2, [0, 5, 10, 4]);
        assert_eq!(a.get(0, 5), 8);
        assert!(a.add(16, 16, &[1; 256]).is_none());
    }

    /// design: render (#103): an image that does not fit changes nothing,
    /// so the rest of the shelf still takes smaller images.
    #[test]
    fn a_failed_add_changes_nothing() {
        let mut a = Atlas::new(16);
        assert_eq!(a.add(10, 4, &[7; 40]), Some([0, 0, 10, 4]));
        // Too wide for the shelf, too tall for a new one.
        assert_eq!(a.add(10, 12, &[8; 120]), None);
        assert_eq!(a.add(4, 4, &[9; 16]), Some([11, 0, 4, 4]));
        assert_eq!(a.get(11, 0), 9);
        assert_eq!(a.get(0, 5), 0, "nothing written below the shelf");
    }

    #[test]
    fn reference_draws_modes_in_order() {
        let mut a = Atlas::new(8);
        let r = a.add(2, 1, &[0, 3]).unwrap();
        let mut pal = [0u8; 1024];
        pal[12..16].copy_from_slice(&[10, 20, 30, 255]);
        let q = [
            Quad::solid([0, 0, 2, 1], [200, 0, 0, 255]),
            Quad::sprite(0, 0, r, 0),
            Quad::solid([0, 0, 1, 1], [0, 0, 255, 128]),
        ];
        let px = reference(&q, &a, &[pal], &BTreeMap::new(), 2, 1);
        assert_eq!(&px[4..8], &[10, 20, 30, 255]);
        // Index 0 is transparent: the red stays under the translucent blue.
        assert_eq!(&px[0..4], &[100, 0, 128, 255]);
    }
}
