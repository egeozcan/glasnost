//! The map scene on the GPU (WebGL2 core path, PLAN.md 4.1).
//!
//! Scene images go into an `R8Uint` array texture atlas, palette rows into an
//! `Rgba8Unorm` texture, and each visible item is one instance. Depth uses a
//! `Depth32Float` buffer holding depth / 65536, which is exact for every
//! depth value 0 … 65535 of S-0102; the depth tests are the hardware tests
//! (≤ for tiles, < for everything else, claims 9 and 10), and writers are
//! the pipelines that store depth. Items are drawn in scene order, batched
//! into runs that share a pipeline, so the result matches
//! [`Scene::compose`] pixel for pixel. The one exception is shadows: the GPU
//! halves the expanded 8-bit channels by blending, rounded down (the blend
//! constant `SHADOW_FACTOR`), so a shadowed pixel packs to the CPU's 5-6-5
//! halving and a later translucent drawing reads the same backdrop, but its
//! low display bits can differ from the CPU's re-expanded channels (S-0102
//! claim 26, F7: an engine that keeps 8-bit channels packs to the same
//! hi-colour pixel; the display path here keeps the expanded channels).

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::compose::{ComposeQuirks, DepthTest, Paint, Profile, expand565};
use crate::scene::{Filter, ItemDepth, Scene, View};
use crate::spots::{SpotGroup, SpotKind};

/// WGSL source.
pub const SHADER_SOURCE: &str = include_str!("map_gpu.wgsl");

/// Depth format of the map pass.
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

/// Colour format of the map pass target.
pub const COLOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

/// The blend constant a shadow multiplies the target's 8-bit channels by.
/// With *c* = 127/255, round(*v*·*c*) = ⌊*v*/2⌋ for every *v* in 0 … 255
/// (*v*/2 − *v*/510 is within ½ − 1/510 of ⌊*v*/2⌋), and 127/255 is exact
/// in an 8-bit fixed-point blend unit. ⌊*v*/2⌋ keeps the top 5 (6) bits of
/// a halved expanded channel equal to the halved 5-bit (6-bit) channel, at
/// every repeat, so `packed` reads the CPU's pixel.
// spec: S-0102 claim 26, F7 (each channel halved, rounded down)
// design: render (#79) (a factor under one half, so the blend's rounding to nearest gives the floor)
const SHADOW_FACTOR: f64 = 127.0 / 255.0;

const NO_IMAGE: u32 = u32::MAX;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
struct ViewUniform {
    origin: [i32; 2],
    size: [i32; 2],
    reference: i32,
    rows_per_column: i32,
    pad: [i32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
struct Instance {
    image: [u32; 4],
    place: [i32; 4],
    depth: [u32; 4],
    paint: [u32; 4],
    /// The companion depth image's world x and y, width and height.
    companion: [i32; 4],
}

/// Where an image sits in the atlas.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Slot {
    x: u32,
    y: u32,
    layer: u32,
}

impl Slot {
    /// No place in the atlas: the image is wider or taller than a layer and
    /// its items are not drawn (design: render (#108)).
    const ABSENT: Slot = Slot {
        x: 0,
        y: 0,
        layer: NO_IMAGE,
    };

    /// Whether the image has a place in the atlas.
    fn present(self) -> bool {
        self.layer != NO_IMAGE
    }
}

/// A shelf packer over layers of `side × side`.
#[derive(Clone, Copy, Debug)]
struct Packer {
    side: u32,
    x: u32,
    y: u32,
    shelf: u32,
    layer: u32,
}

impl Packer {
    fn new(side: u32) -> Self {
        Self {
            side,
            x: 0,
            y: 0,
            shelf: 0,
            layer: 0,
        }
    }

    /// The slot of the next `w × h` rectangle, or `None` when it exceeds a
    /// layer.
    fn place(&mut self, w: u32, h: u32) -> Option<Slot> {
        let side = self.side;
        if w > side || h > side {
            return None;
        }
        if self.x + w > side {
            self.x = 0;
            self.y += self.shelf;
            self.shelf = 0;
        }
        if self.y + h > side {
            self.layer += 1;
            self.x = 0;
            self.y = 0;
            self.shelf = 0;
        }
        let slot = Slot {
            x: self.x,
            y: self.y,
            layer: self.layer,
        };
        self.x += w;
        self.shelf = self.shelf.max(h);
        Some(slot)
    }

    /// Layers in use.
    fn layers(&self) -> u32 {
        self.layer + 1
    }
}

/// Packs rectangles into layers of `side × side` with shelves, tallest
/// first. Returns the slots and the packer, which places later rectangles
/// after them. A rectangle wider or taller than a layer gets
/// [`Slot::ABSENT`] (design: render (#108)).
fn pack(sizes: &[(u32, u32)], side: u32) -> (Vec<Slot>, Packer) {
    let mut order: Vec<usize> = (0..sizes.len()).collect();
    order.sort_by_key(|&i| std::cmp::Reverse(sizes[i].1));
    let mut slots = vec![
        Slot {
            x: 0,
            y: 0,
            layer: 0
        };
        sizes.len()
    ];
    let mut packer = Packer::new(side);
    for i in order {
        let (w, h) = sizes[i];
        // design: render (#108): an image larger than a layer is left out, not the whole upload
        slots[i] = packer.place(w, h).unwrap_or(Slot::ABSENT);
    }
    (slots, packer)
}

/// Whether a `w` × `h` image is wider or taller than a layer of `side`;
/// such an image gets [`Slot::ABSENT`] (design: render (#108)).
fn too_large(w: u32, h: u32, side: u32) -> bool {
    w > side || h > side
}

/// The slot of a `w` × `h` image added after the upload, in an atlas of
/// `layers` layers: [`Slot::ABSENT`] for an image larger than a layer, an
/// error when the atlas is full.
fn place_later(packer: &mut Packer, w: u32, h: u32, layers: u32) -> Result<Slot, UploadError> {
    if too_large(w, h, packer.side) {
        // design: render (#108): left out with no place used, so no rebuild follows
        return Ok(Slot::ABSENT);
    }
    packer
        .place(w.max(1), h.max(1))
        .filter(|s| s.layer < layers)
        .ok_or_else(|| UploadError("the atlas is full".into()))
}

/// Warns about the images of `placed` (size and slot) that got no slot in
/// an atlas of `side`: their items are not drawn on the GPU.
fn warn_absent(placed: impl IntoIterator<Item = ((u32, u32), Slot)>, side: u32) {
    for ((w, h), slot) in placed {
        if !slot.present() {
            log::warn!(
                "a {w} × {h} image exceeds the GPU atlas of {side} × {side}: its items are not drawn"
            );
        }
    }
}

/// Atlas layers kept free at upload for images added later
/// ([`MapRenderer::sync`]); our own choice.
const SPARE_LAYERS: u32 = 1;

/// Pipelines by role.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Role {
    Tile,
    Writer,
    Reader,
    Shadow,
    /// Neither reads nor writes depth (S-0102 claim 25).
    Ignore,
    /// A shadow that stores its depth where it darkens (S-0102 claim 27).
    ShadowWriter,
    /// A shadow that neither reads nor writes depth (S-0102 claim 25).
    ShadowIgnore,
    /// Translucent drawings (S-0102 claim 32, F8) with the depth tests of
    /// a tile, a writer, a reader and a drawing that ignores depth: the
    /// shader blends with a copy of the target.
    BlendTile,
    BlendWriter,
    BlendReader,
    BlendIgnore,
    /// Single-pixel drawings ([`crate::spots`]): depth readers that compute
    /// their pixel from the copy of the target.
    Spot,
}

/// Every role, in pipeline order.
const ROLES: [Role; 12] = [
    Role::Tile,
    Role::Writer,
    Role::Reader,
    Role::Shadow,
    Role::Ignore,
    Role::ShadowWriter,
    Role::ShadowIgnore,
    Role::BlendTile,
    Role::BlendWriter,
    Role::BlendReader,
    Role::BlendIgnore,
    Role::Spot,
];

impl Role {
    /// The role's pipeline.
    fn index(self) -> usize {
        ROLES.iter().position(|&r| r == self).unwrap_or(0)
    }

    /// Whether the role blends with the target's earlier pixels in the
    /// shader, so that it needs a copy of them.
    fn translucent(self) -> bool {
        matches!(
            self,
            Role::BlendTile
                | Role::BlendWriter
                | Role::BlendReader
                | Role::BlendIgnore
                | Role::Spot
        )
    }
}

/// What the map pass drew and uploaded: the counters of the performance
/// benchmark (M8 step C4). [`MapRenderer::take_stats`] returns them and
/// starts counting again.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MapStats {
    /// Scene items (stack copies counted) looked at by the last render.
    pub items: usize,
    /// Instances drawn (each a six-vertex quad).
    pub instances: usize,
    /// Draw calls.
    pub draws: usize,
    /// Render passes begun.
    pub passes: usize,
    /// Copies of the target into the backdrop (one per translucent run).
    pub copies: usize,
    /// Pixels those copies moved.
    pub copied_px: u64,
    /// Texture and buffer writes.
    pub uploads: usize,
    /// Bytes those writes moved.
    pub upload_bytes: u64,
}

/// GPU resources for one scene.
pub struct MapRenderer {
    pipelines: [wgpu::RenderPipeline; ROLES.len()],
    bind_group: wgpu::BindGroup,
    layout: wgpu::BindGroupLayout,
    view_buffer: wgpu::Buffer,
    instances: wgpu::Buffer,
    instance_capacity: usize,
    slots: Vec<Slot>,
    /// Atlas slots of [`Scene::signed`].
    signed_slots: Vec<Slot>,
    rows_per_column: u32,
    atlas: wgpu::Texture,
    atlas_layers: u32,
    packer: Packer,
    row_count: usize,
    /// The scene's [`Scene::rewrites`] when last uploaded or synced.
    rewrites_seen: u64,
    target_format: wgpu::TextureFormat,
    rows: wgpu::Texture,
    /// The light-alpha texture and its size.
    alpha: (wgpu::Texture, (u32, u32)),
    /// Whether the light-alpha texture holds 127 everywhere.
    alpha_default: bool,
    /// The values the light-alpha texture holds when not `alpha_default`:
    /// a frame whose values are the same is not uploaded again (M8 step
    /// C4).
    alpha_held: Vec<u8>,
    /// The copy of the target's pixels that a translucent drawing blends
    /// with (S-0102 claim 32), of the light-alpha texture's size.
    backdrop: wgpu::Texture,
    /// The counters since the last [`MapRenderer::take_stats`].
    stats: MapStats,
    /// The last frame's instance list and their view rectangles, kept for
    /// their room.
    list: Vec<Instance>,
    rects: Vec<[u32; 4]>,
}

/// Why the scene could not be uploaded.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UploadError(pub String);

impl std::fmt::Display for UploadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for UploadError {}

impl MapRenderer {
    /// Uploads `scene` for drawing into targets of `target_format`
    /// ([`COLOR_FORMAT`] or its sRGB form; with an sRGB target the palette
    /// rows are stored sRGB too, so decode on fetch and encode on store
    /// cancel out and the bytes stay exact).
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &Scene,
        target_format: wgpu::TextureFormat,
    ) -> Result<Self, UploadError> {
        let limits = device.limits();
        let side = limits.max_texture_dimension_2d.min(4096);
        // Indexed images, then the signed companion depth images (their bytes
        // are the two's complement values).
        let sizes: Vec<(u32, u32)> = scene
            .images
            .iter()
            .map(|i| (i.width().max(1), i.height().max(1)))
            .chain(
                scene
                    .signed
                    .iter()
                    .map(|i| (i.width.max(1), i.height.max(1))),
            )
            .collect();
        let (mut slots, packer) = pack(&sizes, side);
        warn_absent(sizes.iter().copied().zip(slots.iter().copied()), side);
        let signed_slots = slots.split_off(scene.images.len());
        let layers = packer.layers();
        if layers > limits.max_texture_array_layers {
            return Err(UploadError(format!(
                "the scene needs {layers} atlas layers; the device allows {}",
                limits.max_texture_array_layers
            )));
        }
        // On the GL backends a one-layer texture is a plain 2D texture and
        // cannot be viewed as an array, so the atlas always has two layers.
        // Spare layers take images added later.
        let layers = (layers + SPARE_LAYERS)
            .min(limits.max_texture_array_layers)
            .max(2);
        let mut atlas = vec![0u8; side as usize * side as usize * layers as usize];
        let images = scene.images.iter().map(|i| (i.width(), i.pixels())).chain(
            scene
                .signed
                .iter()
                .map(|i| (i.width, bytemuck::cast_slice::<i8, u8>(&i.values))),
        );
        for ((width, pixels), slot) in images.zip(slots.iter().chain(&signed_slots)) {
            let base = slot.layer as usize * side as usize * side as usize;
            let width = width as usize;
            if width == 0 || !slot.present() {
                continue;
            }
            for (row, src) in pixels.chunks_exact(width).enumerate() {
                let dst = base + (slot.y as usize + row) * side as usize + slot.x as usize;
                atlas[dst..dst + src.len()].copy_from_slice(src);
            }
        }
        let atlas_texture = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some("map atlas"),
                size: wgpu::Extent3d {
                    width: side,
                    height: side,
                    depth_or_array_layers: layers,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::R8Uint,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            &atlas,
        );
        let (rows_texture, rows_per_column) =
            rows_texture(device, queue, scene, side, target_format)?;
        let view_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("map view"),
            size: std::mem::size_of::<ViewUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("map"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Uint,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Uint,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });
        let alpha = alpha_texture(device, queue, (1, 1));
        let backdrop = backdrop_texture(device, (1, 1));
        let bind_group = bind_group(
            device,
            &layout,
            &view_buffer,
            &atlas_texture,
            &rows_texture,
            &alpha,
            &backdrop,
        );
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("map"),
            source: wgpu::ShaderSource::Wgsl(SHADER_SOURCE.into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("map"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let attributes = wgpu::vertex_attr_array![
            0 => Uint32x4, 1 => Sint32x4, 2 => Uint32x4, 3 => Uint32x4, 4 => Sint32x4
        ];
        let make = |role: Role| {
            let (compare, write) = match role {
                Role::Tile => (wgpu::CompareFunction::LessEqual, true),
                Role::Writer => (wgpu::CompareFunction::Less, true),
                Role::Reader | Role::Shadow => (wgpu::CompareFunction::Less, false),
                Role::Ignore => (wgpu::CompareFunction::Always, false),
                Role::ShadowWriter => (wgpu::CompareFunction::Less, true),
                // spec: S-0102 claim 25 (a drawing with no depth test neither reads nor writes depth)
                Role::ShadowIgnore => (wgpu::CompareFunction::Always, false),
                Role::BlendTile => (wgpu::CompareFunction::LessEqual, true),
                Role::BlendWriter => (wgpu::CompareFunction::Less, true),
                Role::BlendReader => (wgpu::CompareFunction::Less, false),
                Role::BlendIgnore => (wgpu::CompareFunction::Always, false),
                // spec: S-0518 claim 20 (a beam reads depth and never writes it; strictly less passes)
                Role::Spot => (wgpu::CompareFunction::Less, false),
            };
            let blend = matches!(role, Role::Shadow | Role::ShadowWriter | Role::ShadowIgnore)
                .then_some(wgpu::BlendState {
                    color: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::Zero,
                        dst_factor: wgpu::BlendFactor::Constant,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha: wgpu::BlendComponent::OVER,
                });
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("map"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    compilation_options: Default::default(),
                    buffers: &[Some(wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Instance>() as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &attributes,
                    })],
                },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: DEPTH_FORMAT,
                    depth_write_enabled: Some(write),
                    depth_compare: Some(compare),
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some(if role == Role::Spot {
                        "fs_spot"
                    } else {
                        "fs_main"
                    }),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: target_format,
                        blend,
                        write_mask: wgpu::ColorWrites::COLOR,
                    })],
                }),
                multiview_mask: None,
                cache: None,
            })
        };
        let pipelines = ROLES.map(make);
        let instance_capacity = 1024;
        let instances = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("map instances"),
            size: (instance_capacity * std::mem::size_of::<Instance>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Ok(Self {
            pipelines,
            bind_group,
            layout,
            view_buffer,
            instances,
            instance_capacity,
            slots,
            signed_slots,
            rows_per_column,
            atlas: atlas_texture,
            atlas_layers: layers,
            packer,
            row_count: scene.rows.len(),
            rewrites_seen: scene.rewrites,
            target_format,
            rows: rows_texture,
            alpha: (alpha, (1, 1)),
            alpha_default: true,
            alpha_held: Vec::new(),
            backdrop,
            stats: MapStats::default(),
            list: Vec::new(),
            rects: Vec::new(),
        })
    }

    /// The counters since the last call (or the upload), and a fresh start.
    pub fn take_stats(&mut self) -> MapStats {
        std::mem::take(&mut self.stats)
    }

    /// Uploads what `scene` gained since the upload or the last sync: images
    /// appended to [`Scene::images`] go into the atlas's free space, images
    /// rewritten in place ([`Scene::replace_image`]) are written again into
    /// their own slots, and palette rows appended to [`Scene::rows`]
    /// replace the row texture. Images and rows already uploaded must
    /// otherwise be unchanged. Fails when the atlas is full (or the scene
    /// lost images); upload the scene anew then.
    pub fn sync(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &Scene,
    ) -> Result<(), UploadError> {
        if scene.images.len() < self.slots.len() || scene.signed.len() < self.signed_slots.len() {
            return Err(UploadError("the scene lost images".into()));
        }
        if scene.rewrites != self.rewrites_seen {
            for (&id, &at) in &scene.rewritten {
                if at <= self.rewrites_seen {
                    continue;
                }
                let (Some(&slot), Some(img)) =
                    (self.slots.get(id as usize), scene.images.get(id as usize))
                else {
                    continue;
                };
                self.write(queue, slot, img.width(), img.height(), img.pixels());
            }
            self.rewrites_seen = scene.rewrites;
        }
        for img in &scene.images[self.slots.len()..] {
            let slot = self.upload(queue, img.width(), img.height(), img.pixels())?;
            self.slots.push(slot);
        }
        for img in &scene.signed[self.signed_slots.len()..] {
            let bytes = bytemuck::cast_slice::<i8, u8>(&img.values);
            let slot = self.upload(queue, img.width, img.height, bytes)?;
            self.signed_slots.push(slot);
        }
        if scene.rows.len() != self.row_count {
            let (rows, per_column) =
                rows_texture(device, queue, scene, self.packer.side, self.target_format)?;
            self.stats.uploads += 1;
            self.stats.upload_bytes += u64::from(rows.width()) * u64::from(rows.height()) * 4;
            self.bind_group = bind_group(
                device,
                &self.layout,
                &self.view_buffer,
                &self.atlas,
                &rows,
                &self.alpha.0,
                &self.backdrop,
            );
            self.rows = rows;
            self.rows_per_column = per_column;
            self.row_count = scene.rows.len();
        }
        Ok(())
    }

    /// Places a `w` × `h` image of bytes `pixels` in the atlas's free space.
    fn upload(
        &mut self,
        queue: &wgpu::Queue,
        w: u32,
        h: u32,
        pixels: &[u8],
    ) -> Result<Slot, UploadError> {
        let slot = place_later(&mut self.packer, w, h, self.atlas_layers)?;
        warn_absent([((w, h), slot)], self.packer.side);
        self.write(queue, slot, w, h, pixels);
        Ok(slot)
    }

    /// Writes a `w` × `h` image of bytes `pixels` into the atlas at `slot`.
    fn write(&mut self, queue: &wgpu::Queue, slot: Slot, w: u32, h: u32, pixels: &[u8]) {
        if w > 0 && h > 0 && slot.present() {
            self.stats.uploads += 1;
            self.stats.upload_bytes += u64::from(w) * u64::from(h);
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &self.atlas,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: slot.x,
                        y: slot.y,
                        z: slot.layer,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                pixels,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(w),
                    rows_per_image: Some(h),
                },
                wgpu::Extent3d {
                    width: w,
                    height: h,
                    depth_or_array_layers: 1,
                },
            );
        }
    }

    /// Records the scene for `view` into `encoder`, rendering into `target`
    /// (a [`COLOR_FORMAT`] texture of the view's size that can be copied
    /// from, as [`targets`] makes it) with `depth` (a [`DEPTH_FORMAT`] view
    /// of the same size). Returns the number of items drawn.
    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::Texture,
        depth: &wgpu::TextureView,
        scene: &Scene,
        view: &View,
        filter: &Filter,
        q: &ComposeQuirks,
    ) -> usize {
        self.render_lit(
            device, queue, encoder, target, depth, scene, view, filter, q, None,
        )
    }

    /// [`Self::render`] with the light-alpha values `alpha` of the view's
    /// pixels, row by row (S-0110 claims 5 and 6; `None`: 127 everywhere).
    ///
    /// A translucent item (S-0102 claim 32) blends with the pixels drawn
    /// before it: the pass ends before it, the target's pixels under it
    /// are copied to the backdrop texture, and the pass resumes, so the
    /// shader applies F8 to the packed channels exactly, as the CPU
    /// reference does. This needs a target of [`COLOR_FORMAT`].
    #[allow(clippy::too_many_arguments)]
    pub fn render_lit(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::Texture,
        depth: &wgpu::TextureView,
        scene: &Scene,
        view: &View,
        filter: &Filter,
        q: &ComposeQuirks,
        alpha: Option<&[u8]>,
    ) -> usize {
        self.set_alpha(device, queue, view.size, alpha);
        let uniform = ViewUniform {
            origin: [view.origin.0, view.origin.1],
            size: [view.size.0 as i32, view.size.1 as i32],
            reference: view.reference,
            rows_per_column: self.rows_per_column as i32,
            pad: [0; 2],
        };
        queue.write_buffer(&self.view_buffer, 0, bytemuck::bytes_of(&uniform));
        let (ox, oy) = view.origin;
        let (w, h) = (view.size.0 as i32, view.size.1 as i32);
        // The buffers of the last frame, emptied, so that a frame does not
        // grow them again.
        let mut list: Vec<Instance> = std::mem::take(&mut self.list);
        list.clear();
        // The view rectangle each instance draws into, for placing the
        // backdrop copies (`passes`).
        let mut rects: Vec<[u32; 4]> = std::mem::take(&mut self.rects);
        rects.clear();
        // A spot instance draws its own pixel.
        let spot_rects = |list: &[Instance], rects: &mut Vec<[u32; 4]>| {
            for i in &list[rects.len()..] {
                rects.push([(i.place[0] - ox) as u32, (i.place[1] - oy) as u32, 1, 1]);
            }
        };
        // Runs of instances that share a pipeline; a translucent item is a
        // run of its own, with the view rectangle it covers.
        let mut batches: Vec<Batch> = Vec::new();
        let mut spot_groups = scene.spots.iter().peekable();
        // spec: S-0109 claim 27 (a stack's copies, lowest first, as many as the view's top row allows)
        let copies = scene.items.iter().enumerate().flat_map(|(n, i)| {
            let count = i.stack.map_or(1, |s| s.copies(oy));
            (0..count).map(move |j| (n, i, j))
        });
        for (n, base, j) in copies {
            self.stats.items += 1;
            while let Some(g) = spot_groups.next_if(|g| g.at <= n) {
                if filter.shows_spots(g) {
                    spot_instances(g, view, &mut list, &mut batches);
                    spot_rects(&list, &mut rects);
                }
            }
            // Copy 0 is the item itself; a higher copy moves only its row and bias.
            let moved;
            let item = if j == 0 {
                base
            } else {
                moved = base.copies(oy).nth(j as usize).unwrap_or(*base);
                &moved
            };
            if !filter.shows(item) {
                continue;
            }
            let (l, t, r, b) = scene.bounds(item);
            // spec: S-0101 F4b (an image is drawn iff its drawn extent meets the view; a bounds miss draws no pixel)
            if r <= ox || b <= oy || l >= ox + w || t >= oy + h {
                continue;
            }
            let img = &scene.images[item.image as usize];
            let slot = self.slots[item.image as usize];
            if !slot.present() {
                // design: render (#108): an image larger than the atlas is not drawn
                continue;
            }
            let mut companion_rect = [0; 4];
            let (role, flags, bias, reference_row, depth_slot) = match item.depth {
                ItemDepth::Tile {
                    depth,
                    reference_row,
                } => (
                    // spec: S-0102 claim 9 (a tile without depth data does not touch the buffer: it neither reads nor writes depth)
                    if depth.is_some() {
                        Role::Tile
                    } else {
                        Role::Ignore
                    },
                    0u32,
                    0,
                    reference_row,
                    depth.map(|d| self.slots[d as usize]),
                ),
                ItemDepth::Profile {
                    profile,
                    bias,
                    companion,
                } => {
                    let mut kind = match profile {
                        Profile::Flat => 1,
                        Profile::Slope => 2,
                        Profile::Upright => 3,
                    };
                    // spec: S-0102 F2, F5 (a companion depth image: bit 2 of the flags, its atlas
                    // slot in the depth fields, its world rectangle in `companion`)
                    let companion_slot = companion.map(|(i, cx, cy)| {
                        let img = &scene.signed[i as usize];
                        kind |= 4;
                        companion_rect = [cx, cy, img.width as i32, img.height as i32];
                        self.signed_slots[i as usize]
                    });
                    let role = match (item.paint, item.test) {
                        // spec: S-0102 claim 27 (a cell-pass shadow stores its depth, quirks.cell_pass_shadows_write_depth)
                        (Paint::Shadow, DepthTest::Writer) if q.cell_pass_shadows_write_depth => {
                            Role::ShadowWriter
                        }
                        // spec: S-0102 claim 25 (no depth test: drawn whatever the stored depth), S-0111 claim 16 (the self-heal marker)
                        (Paint::Shadow, DepthTest::Ignore) => Role::ShadowIgnore,
                        (Paint::Shadow, _) => Role::Shadow,
                        (Paint::Translucent(..), DepthTest::Reader) => Role::BlendReader,
                        (Paint::Translucent(..), DepthTest::Tile) => Role::BlendTile,
                        (Paint::Translucent(..), DepthTest::Writer) => Role::BlendWriter,
                        (Paint::Translucent(..), DepthTest::Ignore) => Role::BlendIgnore,
                        (_, DepthTest::Reader) => Role::Reader,
                        (_, DepthTest::Tile) => Role::Tile,
                        (_, DepthTest::Writer) => Role::Writer,
                        (_, DepthTest::Ignore) => Role::Ignore,
                    };
                    (role, kind, bias, 0, companion_slot)
                }
            };
            // `diamond` holds the diamond flag in bit 0, the translucency
            // code in bits 1 and 2 and the protection colour mask in bits 16
            // to 31.
            let (row, diamond, shade_rows, rho) = match item.paint {
                Paint::Row(r) => (r as u32, 0, 0, 0),
                // spec: S-0010 claim 22 (a diamond has no transparent index)
                Paint::Diamond(r) => (r as u32, 1, 0, 0),
                // spec: S-0102 claim 34, F9 (the opaque drawing's mask K)
                Paint::Lit(s) => (
                    s.base,
                    u32::from(s.mask) << 16,
                    u32::from(s.rows),
                    u32::from(s.rho),
                ),
                Paint::LitDiamond(s) => (
                    s.base,
                    1 | (u32::from(s.mask) << 16),
                    u32::from(s.rows),
                    u32::from(s.rho),
                ),
                // spec: S-0102 claim 32 (codes 1 to 3 blend by F8)
                Paint::Translucent(s, t) => {
                    (s.base, t.code() << 1, u32::from(s.rows), u32::from(s.rho))
                }
                Paint::Shadow => (0, 0, 0, 0),
                // spec: S-0616 claim 28 (a pixel particle's own colour, flagged by bit 3)
                Paint::Pixel(c) => (
                    (u32::from(c.r) << 16) | (u32::from(c.g) << 8) | u32::from(c.b),
                    1 << 3,
                    0,
                    0,
                ),
            };
            if depth_slot.is_some_and(|s| !s.present()) {
                // design: render (#108): nor is an item whose depth image is larger than the atlas
                continue;
            }
            let ds = depth_slot.unwrap_or(Slot {
                x: 0,
                y: 0,
                layer: NO_IMAGE,
            });
            list.push(Instance {
                image: [
                    slot.x,
                    slot.y,
                    slot.layer,
                    img.width() | (img.height() << 16),
                ],
                place: [item.x, item.y, bias, reference_row],
                depth: [ds.x, ds.y, ds.layer, flags],
                paint: [row, diamond, shade_rows, rho],
                companion: companion_rect,
            });
            let n = list.len() as u32;
            // The view pixels the item covers (a translucent item blends
            // with them).
            let x0 = (l - ox).max(0) as u32;
            let y0 = (t - oy).max(0) as u32;
            let x1 = (r - ox).min(w) as u32;
            let y1 = (b - oy).min(h) as u32;
            let rect = [x0, y0, x1 - x0, y1 - y0];
            rects.push(rect);
            if role.translucent() {
                batches.push((role, n - 1..n, Some(rect)));
                continue;
            }
            match batches.last_mut() {
                Some((r, range, None)) if *r == role => range.end = n,
                _ => batches.push((role, n - 1..n, None)),
            }
        }
        for g in spot_groups.filter(|g| filter.shows_spots(g)) {
            spot_instances(g, view, &mut list, &mut batches);
            spot_rects(&list, &mut rects);
        }
        if list.len() > self.instance_capacity {
            self.instance_capacity = list.len().next_power_of_two();
            self.instances = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("map instances"),
                size: (self.instance_capacity * std::mem::size_of::<Instance>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        if !list.is_empty() {
            queue.write_buffer(&self.instances, 0, bytemuck::cast_slice(&list));
            self.stats.uploads += 1;
            self.stats.upload_bytes += std::mem::size_of_val(list.as_slice()) as u64;
        }
        self.stats.instances += list.len();
        let target_view = target.create_view(&Default::default());
        let begin = |encoder: &mut wgpu::CommandEncoder, first: bool| {
            let mut pass = encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("map"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &target_view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: if first {
                                wgpu::LoadOp::Clear(wgpu::Color::BLACK)
                            } else {
                                wgpu::LoadOp::Load
                            },
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: depth,
                        // spec: S-0102 claim 8 (the depth buffer is filled with 65535 on a full redraw)
                        depth_ops: Some(wgpu::Operations {
                            load: if first {
                                wgpu::LoadOp::Clear(
                                    f32::from(crate::compose::DEPTH_CLEAR) / 65536.0,
                                )
                            } else {
                                wgpu::LoadOp::Load
                            },
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    ..Default::default()
                })
                .forget_lifetime();
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.set_vertex_buffer(0, self.instances.slice(..));
            pass.set_blend_constant(wgpu::Color {
                r: SHADOW_FACTOR,
                g: SHADOW_FACTOR,
                b: SHADOW_FACTOR,
                a: 1.0,
            });
            pass
        };
        self.stats.draws += batches.len();
        let mut first = true;
        for (copies, run) in passes(batches, &rects) {
            for [x, y, cw, ch] in copies {
                // spec: S-0102 claim 32 (a translucent pixel blends with the already-painted
                // destination: the pixels drawn so far are copied before the item is drawn)
                encoder.copy_texture_to_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: target,
                        mip_level: 0,
                        origin: wgpu::Origin3d { x, y, z: 0 },
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::TexelCopyTextureInfo {
                        texture: &self.backdrop,
                        mip_level: 0,
                        origin: wgpu::Origin3d { x, y, z: 0 },
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::Extent3d {
                        width: cw,
                        height: ch,
                        depth_or_array_layers: 1,
                    },
                );
                self.stats.copies += 1;
                self.stats.copied_px += u64::from(cw) * u64::from(ch);
            }
            let mut pass = begin(encoder, first);
            first = false;
            self.stats.passes += 1;
            for (role, range, _) in run {
                pass.set_pipeline(&self.pipelines[role.index()]);
                pass.draw(0..6, range);
            }
        }
        let drawn = list.len();
        self.list = list;
        self.rects = rects;
        drawn
    }
}

/// A run of instances sharing a pipeline, with the view rectangle a
/// blending run covers.
type Batch = (Role, std::ops::Range<u32>, Option<[u32; 4]>);

/// The render passes of a frame's `batches` (`rects`: the view rectangle
/// each instance draws into), each with the backdrop copies taken before
/// it begins.
///
/// A blending run reads the copy of the pixels under it, which must hold
/// what was drawn before it. The copy is taken where the last pass began
/// when nothing drawn since then meets the run's rectangle (the pixels
/// there are the same at both moments), else a new pass begins before the
/// run. So runs that do not meet what was drawn since share one pass break
/// instead of one each (M8 step C4); the first pass, which clears the
/// target, takes no copy. There is always at least one pass.
fn passes(batches: Vec<Batch>, rects: &[[u32; 4]]) -> Vec<(Vec<[u32; 4]>, Vec<Batch>)> {
    let meets = |a: &[u32; 4], b: &[u32; 4]| {
        a[0] < b[0] + b[2] && b[0] < a[0] + a[2] && a[1] < b[1] + b[3] && b[1] < a[1] + a[3]
    };
    let mut out: Vec<(Vec<[u32; 4]>, Vec<Batch>)> = vec![(Vec::new(), Vec::new())];
    // The rectangles drawn since the last pass began.
    let mut drawn: Vec<[u32; 4]> = Vec::new();
    for b in batches {
        if let Some(c) = b.2 {
            if out.len() > 1 && !drawn.iter().any(|d| meets(d, &c)) {
                if let Some(last) = out.last_mut() {
                    last.0.push(c);
                }
            } else {
                out.push((vec![c], Vec::new()));
                drawn.clear();
            }
        }
        drawn.extend_from_slice(&rects[b.1.start as usize..b.1.end as usize]);
        if let Some(last) = out.last_mut() {
            last.1.push(b);
        }
    }
    out
}

/// Appends the instances of spot group `g` (one 1 × 1 instance per spot in
/// the view) and their batches. A batch reads the copy of the target taken
/// before it, so a batch ends before a spot that hits a pixel the batch
/// already holds: overlapping spots then apply in order, as on the CPU
/// ([`crate::spots::apply`]). A magnetic wave's spots never repeat a pixel
/// and read the copy around them (S-0607 claim 25).
fn spot_instances(g: &SpotGroup, view: &View, list: &mut Vec<Instance>, batches: &mut Vec<Batch>) {
    let (ox, oy) = view.origin;
    let (w, h) = (view.size.0 as i32, view.size.1 as i32);
    let mut held: std::collections::HashSet<(i32, i32)> = std::collections::HashSet::new();
    let mut rect: Option<[i32; 4]> = None;
    let mut start = list.len() as u32;
    let close =
        |list: &Vec<Instance>, start: u32, rect: Option<[i32; 4]>, batches: &mut Vec<Batch>| {
            let n = list.len() as u32;
            if let Some([x0, y0, x1, y1]) = rect
                && n > start
            {
                let (x0, y0) = (x0.max(0), y0.max(0));
                let (x1, y1) = (x1.min(w), y1.min(h));
                batches.push((
                    Role::Spot,
                    start..n,
                    Some([x0 as u32, y0 as u32, (x1 - x0) as u32, (y1 - y0) as u32]),
                ));
            }
        };
    for s in &g.spots {
        let (x, y) = (s.x - ox, s.y - oy);
        if x < 0 || y < 0 || x >= w || y >= h {
            continue;
        }
        let (kind, colour, weights, shift) = match s.kind {
            SpotKind::Opaque { colour } => (0u32, u32::from(colour), 0u32, (0, 0)),
            SpotKind::Additive { rgb } => (1, rgb_word(rgb), 0, (0, 0)),
            SpotKind::Blended {
                rgb,
                line,
                screen,
                alpha_from,
            } => (
                2,
                rgb_word(rgb),
                u32::from(line) | (u32::from(screen) << 16),
                alpha_from,
            ),
            SpotKind::Flash { rgb } => (3, rgb_word(rgb), 0, (0, 0)),
            SpotKind::Wave { shift, k } => {
                let m = crate::spots::WAVE_MARGIN;
                if x < m || y < m || x >= w - m || y >= h - m {
                    continue;
                }
                (4, 0, u32::from(k), shift)
            }
        };
        if !held.insert((x, y)) {
            close(list, start, rect, batches);
            start = list.len() as u32;
            held.clear();
            held.insert((x, y));
            rect = None;
        }
        // The rectangle of target pixels the batch reads: its own pixels and,
        // for a wave, the source pixels.
        let (sx, sy) = (x + i32::from(shift.0), y + i32::from(shift.1));
        let r = rect.get_or_insert([x, y, x + 1, y + 1]);
        r[0] = r[0].min(x).min(sx);
        r[1] = r[1].min(y).min(sy);
        r[2] = r[2].max(x + 1).max(sx + 1);
        r[3] = r[3].max(y + 1).max(sy + 1);
        list.push(Instance {
            image: [0, 0, NO_IMAGE, 1 | (1 << 16)],
            place: [s.x, s.y, s.bias, 0],
            depth: [0, 0, NO_IMAGE, 0],
            paint: [kind, colour, weights, 0],
            companion: [i32::from(shift.0), i32::from(shift.1), 0, 0],
        });
    }
    close(list, start, rect, batches);
}

/// Three 8-bit channels in one word, red lowest.
fn rgb_word(c: [u8; 3]) -> u32 {
    u32::from(c[0]) | (u32::from(c[1]) << 8) | (u32::from(c[2]) << 16)
}

impl MapRenderer {
    /// Makes the light-alpha texture hold `alpha` (or 127 everywhere) for
    /// a view of `size`.
    fn set_alpha(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        size: (u32, u32),
        alpha: Option<&[u8]>,
    ) {
        let size = (size.0.max(1), size.1.max(1));
        let n = size.0 as usize * size.1 as usize;
        let alpha = alpha.filter(|a| a.len() == n);
        if self.alpha.1 != size {
            let t = alpha_texture(device, queue, size);
            let backdrop = backdrop_texture(device, size);
            self.bind_group = bind_group(
                device,
                &self.layout,
                &self.view_buffer,
                &self.atlas,
                &self.rows,
                &t,
                &backdrop,
            );
            self.alpha = (t, size);
            self.backdrop = backdrop;
            self.alpha_default = true;
        }
        let bytes = match alpha {
            Some(a) if !self.alpha_default && self.alpha_held == a => return,
            Some(a) => {
                self.alpha_default = false;
                self.alpha_held.clear();
                self.alpha_held.extend_from_slice(a);
                std::borrow::Cow::Borrowed(a)
            }
            None if self.alpha_default => return,
            None => {
                self.alpha_default = true;
                self.alpha_held.clear();
                std::borrow::Cow::Owned(vec![crate::light::DEFAULT_LIGHT_ALPHA as u8; n])
            }
        };
        self.stats.uploads += 1;
        self.stats.upload_bytes += bytes.len() as u64;
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.alpha.0,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &bytes,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(size.0),
                rows_per_image: Some(size.1),
            },
            wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
        );
    }
}

/// A light-alpha texture of `size`, 127 everywhere (S-0110 claim 5).
fn alpha_texture(device: &wgpu::Device, queue: &wgpu::Queue, size: (u32, u32)) -> wgpu::Texture {
    device.create_texture_with_data(
        queue,
        &wgpu::TextureDescriptor {
            label: Some("map light-alpha"),
            size: wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Uint,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        },
        wgpu::util::TextureDataOrder::LayerMajor,
        &vec![crate::light::DEFAULT_LIGHT_ALPHA as u8; size.0 as usize * size.1 as usize],
    )
}

/// The backdrop texture of a view of `size`: a copy of the target's pixels
/// under a translucent item (S-0102 claim 32).
fn backdrop_texture(device: &wgpu::Device, size: (u32, u32)) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("map backdrop"),
        size: wgpu::Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: COLOR_FORMAT,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    })
}

/// The palette-row texture of `scene` and its rows per column.
fn rows_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    scene: &Scene,
    side: u32,
    target_format: wgpu::TextureFormat,
) -> Result<(wgpu::Texture, u32), UploadError> {
    let rows_per_column = scene.rows.len().clamp(1, side as usize) as u32;
    let columns = (scene.rows.len().max(1) as u32).div_ceil(rows_per_column);
    if columns * 256 > side {
        return Err(UploadError("too many palette rows".into()));
    }
    let mut row_bytes = vec![0u8; (columns * 256 * rows_per_column * 4) as usize];
    for (r, row) in scene.rows.iter().enumerate() {
        let (col, y) = (r as u32 / rows_per_column, r as u32 % rows_per_column);
        for (i, &c) in row.iter().enumerate() {
            let (cr, cg, cb) = expand565(c);
            let at = ((y * columns * 256 + col * 256 + i as u32) * 4) as usize;
            row_bytes[at..at + 4].copy_from_slice(&[cr, cg, cb, 255]);
        }
    }
    let texture = device.create_texture_with_data(
        queue,
        &wgpu::TextureDescriptor {
            label: Some("map rows"),
            size: wgpu::Extent3d {
                width: columns * 256,
                height: rows_per_column,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: if target_format.is_srgb() {
                wgpu::TextureFormat::Rgba8UnormSrgb
            } else {
                wgpu::TextureFormat::Rgba8Unorm
            },
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        },
        wgpu::util::TextureDataOrder::LayerMajor,
        &row_bytes,
    );
    Ok((texture, rows_per_column))
}

/// The map pass's bind group.
fn bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    view_buffer: &wgpu::Buffer,
    atlas: &wgpu::Texture,
    rows: &wgpu::Texture,
    alpha: &wgpu::Texture,
    backdrop: &wgpu::Texture,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("map"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: view_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&atlas.create_view(
                    &wgpu::TextureViewDescriptor {
                        dimension: Some(wgpu::TextureViewDimension::D2Array),
                        ..Default::default()
                    },
                )),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(
                    &rows.create_view(&wgpu::TextureViewDescriptor::default()),
                ),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(
                    &alpha.create_view(&wgpu::TextureViewDescriptor::default()),
                ),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::TextureView(
                    &backdrop.create_view(&wgpu::TextureViewDescriptor::default()),
                ),
            },
        ],
    })
}

/// Creates a colour target of `format` and a depth buffer for a view of
/// `size`.
pub fn targets(
    device: &wgpu::Device,
    size: (u32, u32),
    format: wgpu::TextureFormat,
) -> (wgpu::Texture, wgpu::Texture) {
    let make = |format, usage, label| {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: size.0.max(1),
                height: size.1.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        })
    };
    (
        make(
            format,
            wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            "map colour",
        ),
        make(
            DEPTH_FORMAT,
            wgpu::TextureUsages::RENDER_ATTACHMENT,
            "map depth",
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The backdrop copies share pass breaks only where that copies the
    /// same pixels (M8 step C4): a blending run that meets nothing drawn
    /// since the last break takes its copy there; one that meets it, or
    /// any in the first pass, begins a pass of its own.
    #[test]
    fn backdrop_copies_share_breaks_only_where_nothing_changed() {
        let r = |x, y, w, h| [x, y, w, h];
        // Instances: 0 opaque over the left, 1 blending on the right, 2
        // blending further right, 3 blending over 0, 4 opaque over 1.
        let rects = [
            r(0, 0, 10, 10),
            r(20, 0, 5, 5),
            r(30, 0, 5, 5),
            r(2, 2, 4, 4),
            r(20, 0, 2, 2),
        ];
        let batches: Vec<Batch> = vec![
            (Role::Writer, 0..1, None),
            (Role::BlendReader, 1..2, Some(rects[1])),
            (Role::BlendReader, 2..3, Some(rects[2])),
            (Role::BlendReader, 3..4, Some(rects[3])),
            (Role::Reader, 4..5, None),
        ];
        let p = passes(batches, &rects);
        // The first pass (it clears) has no copy; 1 begins a pass; 2 shares
        // its break (it does not meet 1); 3 shares it too (it meets only 0,
        // drawn before the break, so the copy holds it); 4 joins the pass.
        assert_eq!(p.len(), 2);
        assert!(p[0].0.is_empty());
        assert_eq!(p[0].1.len(), 1);
        assert_eq!(p[1].0, vec![rects[1], rects[2], rects[3]]);
        assert_eq!(p[1].1.len(), 4);

        // A blending run over a blending run drawn since the break begins
        // a new pass with its own copy.
        let rects = [r(0, 0, 4, 4), r(10, 0, 4, 4), r(12, 2, 4, 4), r(0, 0, 1, 1)];
        let batches: Vec<Batch> = vec![
            (Role::Tile, 0..1, None),
            (Role::BlendWriter, 1..2, Some(rects[1])),
            (Role::BlendIgnore, 2..3, Some(rects[2])),
            (Role::Spot, 3..4, Some(rects[3])),
        ];
        let p = passes(batches, &rects);
        assert_eq!(
            p.iter().map(|(c, _)| c.len()).collect::<Vec<_>>(),
            vec![0, 1, 2]
        );
        assert_eq!(p[2].0, vec![rects[2], rects[3]]);
        // Every run is kept, in order.
        let order: Vec<u32> = p
            .iter()
            .flat_map(|(_, b)| b.iter().map(|b| b.1.start))
            .collect();
        assert_eq!(order, vec![0, 1, 2, 3]);

        // A frame that starts with a blending run, and one with no runs.
        let rects = [r(0, 0, 1, 1)];
        let p = passes(vec![(Role::Spot, 0..1, Some(rects[0]))], &rects);
        assert_eq!(p.len(), 2);
        assert!(p[0].0.is_empty() && p[0].1.is_empty());
        assert_eq!(p[1].0, vec![rects[0]]);
        assert_eq!(passes(Vec::new(), &[]).len(), 1);
    }

    #[test]
    fn shelf_packing_fits() {
        let (slots, packer) = pack(&[(10, 10), (5, 12), (10, 10)], 16);
        assert_eq!(slots.len(), 3);
        assert!(slots.iter().all(|s| s.present()));
        assert!(packer.layers() >= 2);
    }

    #[test]
    fn later_images_go_after_the_packed_ones() {
        let (slots, mut packer) = pack(&[(10, 10), (5, 12)], 16);
        let next = packer.place(6, 4).unwrap();
        // The new slot overlaps none of the first ones.
        for (s, (w, h)) in slots.iter().zip([(10, 10), (5, 12)]) {
            let apart = s.layer != next.layer
                || next.x >= s.x + w
                || s.x >= next.x + 6
                || next.y >= s.y + h
                || s.y >= next.y + 4;
            assert!(apart, "{s:?} and {next:?} overlap");
        }
    }

    /// design: render (#108): an image wider or taller than an atlas layer
    /// gets no slot and fails nothing, at the upload as later; the others
    /// are placed as before, and a full atlas is still an error.
    #[test]
    fn an_image_larger_than_a_layer_gets_no_slot() {
        let (slots, mut packer) = pack(&[(10, 10), (17, 1), (5, 12), (4, 20)], 16);
        assert!(slots[0].present() && slots[2].present());
        assert_eq!(slots[1], Slot::ABSENT);
        assert_eq!(slots[3], Slot::ABSENT);
        let mut unchanged = packer;
        assert_eq!(place_later(&mut packer, 17, 1, 8), Ok(Slot::ABSENT));
        assert_eq!(place_later(&mut packer, 1, 17, 8), Ok(Slot::ABSENT));
        assert_eq!(
            place_later(&mut packer, 6, 4, 8),
            Ok(unchanged.place(6, 4).unwrap())
        );
        let mut full = Packer::new(16);
        assert!(place_later(&mut full, 16, 16, 1).is_ok());
        assert!(place_later(&mut full, 16, 16, 1).is_err());
    }

    #[test]
    fn instance_is_80_bytes() {
        assert_eq!(std::mem::size_of::<Instance>(), 80);
    }
}
