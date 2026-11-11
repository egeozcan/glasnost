//! What egui samples for the GPU map view: the texture view it is given.

use glasnost_render::wgpu;

/// The texture view egui samples for the GPU map view.
pub(super) struct MapDisplay {
    view: wgpu::TextureView,
}

impl MapDisplay {
    /// The display of the map pass's colour target `color`.
    pub(super) fn new(color: &wgpu::Texture) -> Self {
        // design: render (#77) egui-wgpu takes sampled values as they are (its own textures are not
        // sRGB-aware), so it samples the exact (non-sRGB) colour target itself: no sRGB-typed
        // copy, which the sampler would decode to linear, and no per-frame copy
        Self {
            view: color.create_view(&wgpu::TextureViewDescriptor::default()),
        }
    }

    /// The view egui samples.
    pub(super) fn view(&self) -> &wgpu::TextureView {
        &self.view
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glasnost_render::map_gpu::{COLOR_FORMAT, targets};

    /// Draws `display`'s view with egui-wgpu over a `w × h` screen and
    /// returns its RGBA bytes.
    fn egui_shows(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        display: &MapDisplay,
        (w, h): (u32, u32),
    ) -> Vec<u8> {
        let format = wgpu::TextureFormat::Rgba8Unorm;
        let mut renderer = egui_wgpu::Renderer::new(
            device,
            format,
            egui_wgpu::RendererOptions {
                msaa_samples: 1,
                depth_stencil_format: None,
                dithering: false,
                predictable_texture_filtering: true,
            },
        );
        let id =
            renderer.register_native_texture(device, display.view(), wgpu::FilterMode::Nearest);
        let ctx = egui::Context::default();
        let screen_rect =
            egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(w as f32, h as f32));
        let raw = egui::RawInput {
            screen_rect: Some(screen_rect),
            ..Default::default()
        };
        let out = ctx.run_ui(raw, |ui| {
            ui.painter().image(
                id,
                screen_rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
        });
        for (tid, deltas) in &out.textures_delta.set {
            for d in deltas {
                renderer.update_texture(device, queue, *tid, d);
            }
        }
        let jobs = ctx.tessellate(out.shapes, 1.0);
        let screen = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [w, h],
            pixels_per_point: 1.0,
        };
        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = target.create_view(&Default::default());
        let mut enc = device.create_command_encoder(&Default::default());
        let extra = renderer.update_buffers(device, queue, &mut enc, &jobs, &screen);
        {
            let pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            renderer.render(&mut pass.forget_lifetime(), &jobs, &screen);
        }
        queue.submit(extra.into_iter().chain([enc.finish()]));
        glasnost_render::offscreen::read_texture(device, queue, &target, w, h)
    }

    /// #77: egui-wgpu takes the bytes it samples as they are (its textures
    /// are not sRGB-aware), so the GPU map view shows the map pass's bytes,
    /// as the CPU reference view does. spec: S-0102 claims 9 and 10 (the GPU
    /// path draws as the CPU reference does)
    #[test]
    fn egui_shows_the_map_pass_bytes() {
        let Ok(glasnost_render::gpu::SharedDevice { device, queue, .. }) =
            glasnost_render::gpu::shared_device()
        else {
            eprintln!("skipping: no GPU adapter");
            return;
        };
        let size = (16, 16);
        let (color, _depth) = targets(&device, size, COLOR_FORMAT);
        let mut enc = device.create_command_encoder(&Default::default());
        {
            let view = color.create_view(&Default::default());
            let grey = 128.0 / 255.0;
            enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: grey,
                            g: grey,
                            b: grey,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
        }
        let display = MapDisplay::new(&color);
        queue.submit([enc.finish()]);
        let shown = egui_shows(&device, &queue, &display, size);
        let mid = ((8 * size.0 + 8) * 4) as usize;
        for k in 0..3 {
            assert!(
                shown[mid + k].abs_diff(128) <= 1,
                "egui shows {:?} for 128",
                &shown[mid..mid + 4]
            );
        }
    }
}
