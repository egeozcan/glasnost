//! A palette-indexed textured quad: the smallest piece of the core sprite
//! path (PLAN.md 4.4). Indices live in an R8Uint texture, colours in a
//! 256×N palette texture; the fragment shader resolves one through the other.

use bytemuck::{Pod, Zeroable};
use glasnost_fmt::image::{IndexedImage, PALETTE_LEN, Palette};
use wgpu::util::DeviceExt;

/// WGSL source of the quad pipeline.
pub const SHADER_SOURCE: &str = include_str!("palette_quad.wgsl");

/// Per-draw parameters (one uniform block, 16 bytes).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct QuadParams {
    /// Half-extent of the quad in clip space; `[1.0, 1.0]` fills the target.
    pub scale: [f32; 2],
    /// Palette row to use.
    pub palette_row: u32,
    /// Shade offset within each 16-entry ramp.
    pub cycle: u32,
}

/// CPU reference for the shader's palette-cycling rule: the low four bits
/// (shade) rotate by `cycle`, the high four bits (ramp) stay.
pub fn cycled_index(index: u8, cycle: u32) -> u8 {
    (index & 0xF0) | ((u32::from(index) + cycle) & 0x0F) as u8
}

/// Why a [`PaletteQuad`] could not be built.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QuadError {
    /// No palettes were supplied.
    NoPalettes,
    /// A texture would exceed the device's 2D texture limit.
    TooLarge {
        /// Requested width or height.
        size: u32,
        /// `max_texture_dimension_2d` of the device.
        limit: u32,
    },
}

impl std::fmt::Display for QuadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoPalettes => write!(f, "at least one palette is required"),
            Self::TooLarge { size, limit } => {
                write!(f, "texture size {size} exceeds the device limit {limit}")
            }
        }
    }
}

impl std::error::Error for QuadError {}

/// GPU resources for one palette-indexed quad.
pub struct PaletteQuad {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    params: wgpu::Buffer,
}

impl PaletteQuad {
    /// Uploads `image` and `palettes` (one texture row each) and builds a
    /// pipeline that renders into `target_format`.
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target_format: wgpu::TextureFormat,
        image: &IndexedImage,
        palettes: &[Palette],
    ) -> Result<Self, QuadError> {
        if palettes.is_empty() {
            return Err(QuadError::NoPalettes);
        }
        let limit = device.limits().max_texture_dimension_2d;
        let rows = u32::try_from(palettes.len()).unwrap_or(u32::MAX);
        for size in [image.width(), image.height(), rows] {
            if size > limit {
                return Err(QuadError::TooLarge { size, limit });
            }
        }

        let index_texture = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some("palette-quad indices"),
                size: wgpu::Extent3d {
                    width: image.width(),
                    height: image.height(),
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
            image.pixels(),
        );

        // Palette colours are sRGB-encoded. On an sRGB target, store them in an
        // sRGB texture so decode-on-load and encode-on-store cancel out.
        let palette_format = if target_format.is_srgb() {
            wgpu::TextureFormat::Rgba8UnormSrgb
        } else {
            wgpu::TextureFormat::Rgba8Unorm
        };
        let palette_bytes: Vec<u8> = palettes.iter().flat_map(|p| p.to_rgba8()).collect();
        let palette_texture = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some("palette-quad palettes"),
                size: wgpu::Extent3d {
                    width: PALETTE_LEN as u32,
                    height: rows,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: palette_format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            &palette_bytes,
        );

        let params = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("palette-quad params"),
            contents: bytemuck::bytes_of(&QuadParams::default()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let texture_entry = |binding, sample_type| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type,
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("palette-quad"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<QuadParams>() as u64
                        ),
                    },
                    count: None,
                },
                texture_entry(1, wgpu::TextureSampleType::Uint),
                texture_entry(2, wgpu::TextureSampleType::Float { filterable: false }),
            ],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("palette-quad"),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &index_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(
                        &palette_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
            ],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("palette-quad"),
            source: wgpu::ShaderSource::Wgsl(SHADER_SOURCE.into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("palette-quad"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("palette-quad"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });

        Ok(Self {
            pipeline,
            bind_group,
            params,
        })
    }

    /// Updates the per-draw parameters for the next submission.
    pub fn set_params(&self, queue: &wgpu::Queue, params: &QuadParams) {
        queue.write_buffer(&self.params, 0, bytemuck::bytes_of(params));
    }

    /// Records the draw into `pass`.
    pub fn draw(&self, pass: &mut wgpu::RenderPass<'_>) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.draw(0..6, 0..1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycling_keeps_the_ramp() {
        assert_eq!(cycled_index(0x37, 0), 0x37);
        assert_eq!(cycled_index(0x37, 2), 0x39);
        assert_eq!(cycled_index(0x3F, 1), 0x30);
        assert_eq!(cycled_index(0xFF, 17), 0xF0);
    }

    #[test]
    fn params_fit_one_uniform_block() {
        assert_eq!(std::mem::size_of::<QuadParams>(), 16);
    }
}
