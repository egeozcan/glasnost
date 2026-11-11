//! What egui samples for the GPU map view: the texture view it is given and
//! what brings that view up to the map pass's colour target each frame.

use glasnost_render::wgpu;

/// The texture view egui samples for the GPU map view, with the copy of the
/// colour target it needs, if any.
pub(super) struct MapDisplay {
    copy: Option<wgpu::Texture>,
    view: wgpu::TextureView,
}

impl MapDisplay {
    /// The display of the map pass's colour target `color`.
    pub(super) fn new(device: &wgpu::Device, color: &wgpu::Texture) -> Self {
        let display = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("map display"),
            size: color.size(),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = display.create_view(&wgpu::TextureViewDescriptor::default());
        Self {
            copy: Some(display),
            view,
        }
    }

    /// The view egui samples.
    pub(super) fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    /// Records what brings the display up to `color`'s picture, after the
    /// map pass.
    pub(super) fn record(&self, encoder: &mut wgpu::CommandEncoder, color: &wgpu::Texture) {
        // The pass renders into an exact (non-sRGB) target; egui samples an
        // sRGB-typed copy of the same bytes.
        if let Some(copy) = &self.copy {
            encoder.copy_texture_to_texture(
                color.as_image_copy(),
                copy.as_image_copy(),
                color.size(),
            );
        }
    }
}
