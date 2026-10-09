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
//! halves the expanded 8-bit channels by blending, which can differ from the
//! CPU's 5-6-5 halving in the lowest bits (both provisional, Q-0015).

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::compose::{DepthTest, Paint, Profile, expand565};
use crate::scene::{Filter, ItemDepth, Scene, View};

/// WGSL source.
pub const SHADER_SOURCE: &str = include_str!("map_gpu.wgsl");

/// Depth format of the map pass.
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

/// Colour format of the map pass target.
pub const COLOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

const NO_IMAGE: u32 = u32::MAX;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
struct ViewUniform {
    origin: [i32; 2],
    size: [i32; 2],
    reference: i32,
    rows_per_column: i32,
    anchor_after_clip: i32,
    pad: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
struct Instance {
    image: [u32; 4],
    place: [i32; 4],
    depth: [u32; 4],
    paint: [u32; 4],
}

/// Where an image sits in the atlas.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Slot {
    x: u32,
    y: u32,
    layer: u32,
}

/// Packs rectangles into layers of `side × side` with shelves.
fn pack(sizes: &[(u32, u32)], side: u32) -> Option<(Vec<Slot>, u32)> {
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
    let (mut x, mut y, mut shelf, mut layer) = (0u32, 0u32, 0u32, 0u32);
    for i in order {
        let (w, h) = sizes[i];
        if w > side || h > side {
            return None;
        }
        if x + w > side {
            x = 0;
            y += shelf;
            shelf = 0;
        }
        if y + h > side {
            layer += 1;
            x = 0;
            y = 0;
            shelf = 0;
        }
        slots[i] = Slot { x, y, layer };
        x += w;
        shelf = shelf.max(h);
    }
    Some((slots, layer + 1))
}

/// Pipelines by role.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Role {
    Tile,
    Writer,
    Reader,
    Shadow,
}

/// GPU resources for one scene.
pub struct MapRenderer {
    pipelines: [wgpu::RenderPipeline; 4],
    bind_group: wgpu::BindGroup,
    view_buffer: wgpu::Buffer,
    instances: wgpu::Buffer,
    instance_capacity: usize,
    slots: Vec<Slot>,
    rows_per_column: u32,
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
        let sizes: Vec<(u32, u32)> = scene
            .images
            .iter()
            .map(|i| (i.width().max(1), i.height().max(1)))
            .collect();
        let (slots, layers) =
            pack(&sizes, side).ok_or_else(|| UploadError("an image exceeds the atlas".into()))?;
        if layers > limits.max_texture_array_layers {
            return Err(UploadError(format!(
                "the scene needs {layers} atlas layers; the device allows {}",
                limits.max_texture_array_layers
            )));
        }
        // On the GL backends a one-layer texture is a plain 2D texture and
        // cannot be viewed as an array, so the atlas always has two layers.
        let layers = layers.max(2);
        let mut atlas = vec![0u8; side as usize * side as usize * layers as usize];
        for (img, slot) in scene.images.iter().zip(&slots) {
            let base = slot.layer as usize * side as usize * side as usize;
            for row in 0..img.height() as usize {
                let src = &img.pixels()[row * img.width() as usize..][..img.width() as usize];
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
        let rows_texture = device.create_texture_with_data(
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
            ],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("map"),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: view_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&atlas_texture.create_view(
                        &wgpu::TextureViewDescriptor {
                            dimension: Some(wgpu::TextureViewDimension::D2Array),
                            ..Default::default()
                        },
                    )),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(
                        &rows_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
            ],
        });
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
            0 => Uint32x4, 1 => Sint32x4, 2 => Uint32x4, 3 => Uint32x4
        ];
        let make = |role: Role| {
            let (compare, write) = match role {
                Role::Tile => (wgpu::CompareFunction::LessEqual, true),
                Role::Writer => (wgpu::CompareFunction::Less, true),
                Role::Reader | Role::Shadow => (wgpu::CompareFunction::Less, false),
            };
            let blend = (role == Role::Shadow).then_some(wgpu::BlendState {
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
                    entry_point: Some("fs_main"),
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
        let pipelines = [
            make(Role::Tile),
            make(Role::Writer),
            make(Role::Reader),
            make(Role::Shadow),
        ];
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
            view_buffer,
            instances,
            instance_capacity,
            slots,
            rows_per_column,
        })
    }

    /// Records the scene for `view` into `encoder`, rendering into `target`
    /// (a [`COLOR_FORMAT`] view of the view's size) with `depth` (a
    /// [`DEPTH_FORMAT`] view of the same size). Returns the number of items
    /// drawn.
    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        depth: &wgpu::TextureView,
        scene: &Scene,
        view: &View,
        filter: &Filter,
        anchor_after_clip: bool,
    ) -> usize {
        let uniform = ViewUniform {
            origin: [view.origin.0, view.origin.1],
            size: [view.size.0 as i32, view.size.1 as i32],
            reference: view.reference,
            rows_per_column: self.rows_per_column as i32,
            anchor_after_clip: i32::from(anchor_after_clip),
            pad: 0,
        };
        queue.write_buffer(&self.view_buffer, 0, bytemuck::bytes_of(&uniform));
        let (ox, oy) = view.origin;
        let (w, h) = (view.size.0 as i32, view.size.1 as i32);
        let mut list: Vec<Instance> = Vec::new();
        let mut batches: Vec<(Role, std::ops::Range<u32>)> = Vec::new();
        for item in &scene.items {
            if !filter.shows(item) {
                continue;
            }
            let (l, t, r, b) = scene.bounds(item);
            if r <= ox || b <= oy || l >= ox + w || t >= oy + h {
                continue;
            }
            let img = &scene.images[item.image as usize];
            let slot = self.slots[item.image as usize];
            let (role, flags, bias, reference_row, depth_slot) = match item.depth {
                ItemDepth::Tile {
                    depth,
                    reference_row,
                } => (
                    Role::Tile,
                    0u32,
                    0,
                    reference_row,
                    depth.map(|d| self.slots[d as usize]),
                ),
                ItemDepth::Profile { profile, bias, .. } => {
                    let kind = match profile {
                        Profile::Flat => 1,
                        Profile::Slope => 2,
                        Profile::Upright => 3,
                    };
                    let role = match (item.paint, item.test) {
                        (Paint::Shadow, _) => Role::Shadow,
                        (_, DepthTest::Reader) => Role::Reader,
                        (_, DepthTest::Tile) => Role::Tile,
                        (_, DepthTest::Writer) => Role::Writer,
                    };
                    (role, kind, bias, 0, None)
                }
            };
            let row = match item.paint {
                Paint::Row(r) => r as u32,
                Paint::Shadow => 0,
            };
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
                paint: [row, 0, 0, 0],
            });
            let n = list.len() as u32;
            match batches.last_mut() {
                Some((r, range)) if *r == role => range.end = n,
                _ => batches.push((role, n - 1..n)),
            }
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
        }
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("map"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth,
                // spec: S-0102 claim 8 (the depth buffer is filled with 65535 on a full redraw)
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(f32::from(crate::compose::DEPTH_CLEAR) / 65536.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            ..Default::default()
        });
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.instances.slice(..));
        pass.set_blend_constant(wgpu::Color {
            r: 0.5,
            g: 0.5,
            b: 0.5,
            a: 1.0,
        });
        for (role, range) in batches {
            let i = match role {
                Role::Tile => 0,
                Role::Writer => 1,
                Role::Reader => 2,
                Role::Shadow => 3,
            };
            pass.set_pipeline(&self.pipelines[i]);
            pass.draw(0..6, range);
        }
        list.len()
    }
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

    #[test]
    fn shelf_packing_fits() {
        let (slots, layers) = pack(&[(10, 10), (5, 20), (10, 10)], 16).unwrap();
        assert_eq!(slots.len(), 3);
        assert!(layers >= 2);
        assert!(pack(&[(17, 1)], 16).is_none());
    }

    #[test]
    fn instance_is_64_bytes() {
        assert_eq!(std::mem::size_of::<Instance>(), 64);
    }
}
