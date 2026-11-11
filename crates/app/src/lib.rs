//! Application shell shared by `platform-native` and `platform-web`: one
//! winit event loop, the same rendering code on both targets.
//!
//! **Milestone M0**: the palette-indexed quad demo ([`run_demo`]).
//! **M1**: the asset viewer ([`run_viewer`], module [`viewer`]).
//! **M5b**: the game client ([`run_game`], module [`game`]): menu, skirmish
//! setup, the game against the AI and the end screen.

use std::sync::Arc;

use glasnost_fmt::image::IndexedImage;
use glasnost_fmt::synth::{checkerboard, ramp_palette};
use glasnost_render::gpu::{self, BackendChoice};
use glasnost_render::palette_quad::{PaletteQuad, QuadParams};
use glasnost_render::wgpu;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

pub mod game;
pub mod mapview;
mod redraw;
pub mod settings;
mod status;
pub mod viewer;
pub use game::{GameOptions, Perturb, run_game};
pub use status::Status;
pub use viewer::{ViewerOptions, run_viewer};

/// Options for [`run_demo`].
#[derive(Clone, Debug, Default)]
pub struct DemoOptions {
    /// Backend policy (see [`BackendChoice`]).
    pub backend: BackendChoice,
    /// Close after presenting this many frames (native smoke tests).
    pub exit_after_frames: Option<u64>,
}

/// Frames between palette-cycling steps.
const CYCLE_EVERY_FRAMES: u64 = 6;
/// Side of the demo image in pixels, and of one checker square.
const IMAGE_SIZE: u32 = 128;
const CHECKER_CELL: u32 = 16;

/// Opens a window and runs the palette-indexed quad demo.
///
/// Native: blocks until the window closes. Web: starts the event loop on the
/// browser's and returns immediately.
pub fn run_demo(options: DemoOptions) -> Result<(), winit::error::EventLoopError> {
    let event_loop = EventLoop::<Result<Graphics, String>>::with_user_event().build()?;
    let app = App {
        options,
        proxy: Some(event_loop.create_proxy()),
        window: None,
        graphics: None,
        hidden: redraw::Hidden::default(),
    };
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut app = app;
        event_loop.run_app(&mut app)
    }
    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::EventLoopExtWebSys;
        event_loop.spawn_app(app);
        Ok(())
    }
}

struct App {
    options: DemoOptions,
    /// Taken when graphics initialisation starts, so it runs once.
    proxy: Option<EventLoopProxy<Result<Graphics, String>>>,
    window: Option<Arc<Window>>,
    graphics: Option<Graphics>,
    /// Whether the window is hidden (#99).
    hidden: redraw::Hidden,
}

impl App {
    /// Asks a hidden window for a redraw once it is shown, resized or
    /// focused (#99).
    fn wake(&mut self) {
        if self.hidden.wake()
            && let Some(g) = &self.graphics
        {
            g.window.request_redraw();
        }
    }
}

impl ApplicationHandler<Result<Graphics, String>> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let Some(proxy) = self.proxy.take() else {
            return;
        };
        let attributes = Window::default_attributes().with_title("Glasnost: palette quad");
        #[cfg(target_arch = "wasm32")]
        let attributes = {
            use winit::platform::web::WindowAttributesExtWebSys;
            attributes.with_append(true)
        };
        let window = match event_loop.create_window(attributes) {
            Ok(w) => Arc::new(w),
            Err(e) => {
                Status::Error(format!("could not open a window: {e}")).publish();
                event_loop.exit();
                return;
            }
        };
        self.window = Some(window.clone());
        Status::Starting.publish();

        let backend = self.options.backend;
        #[cfg(not(target_arch = "wasm32"))]
        {
            let display: Box<dyn wgpu::wgt::WgpuHasDisplayHandle> =
                Box::new(event_loop.owned_display_handle());
            let result = pollster::block_on(Graphics::new(window, backend, Some(display)));
            // The loop is still running, so this cannot fail.
            let _ = proxy.send_event(result);
        }
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(async move {
            let result = Graphics::new(window, backend, None).await;
            let _ = proxy.send_event(result);
        });
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: Result<Graphics, String>) {
        match event {
            Ok(graphics) => {
                log::info!("rendering with {}", graphics.backend);
                graphics.window.request_redraw();
                self.graphics = Some(graphics);
            }
            Err(e) => {
                Status::Error(e).publish();
                event_loop.exit();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(g) = &mut self.graphics {
                    g.resize(size.width, size.height);
                }
                self.wake();
            }
            WindowEvent::Occluded(false) | WindowEvent::Focused(true) => self.wake(),
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Named(NamedKey::Space),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                if let Some(g) = &mut self.graphics {
                    g.palette_row = (g.palette_row + 1) % g.palette_rows;
                }
            }
            WindowEvent::RedrawRequested => {
                let Some(g) = &mut self.graphics else {
                    return;
                };
                let drawn = match g.render() {
                    Ok(d) => d,
                    Err(e) => {
                        Status::Error(e).publish();
                        event_loop.exit();
                        return;
                    }
                };
                if g.frames_presented == 1 {
                    Status::Presented {
                        backend: g.backend.to_owned(),
                        webgpu_extras: g.webgpu_extras,
                    }
                    .publish();
                }
                if self
                    .options
                    .exit_after_frames
                    .is_some_and(|n| g.frames_presented >= n)
                {
                    event_loop.exit();
                    return;
                }
                // A hidden window waits (#99).
                if self.hidden.after(drawn, redraw::now_ms()) {
                    g.window.request_redraw();
                }
            }
            _ => {}
        }
    }

    /// While the window is hidden, the loop wakes every `redraw::RETRY_MS`
    /// to try again (#99), and waits for events otherwise.
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let now = redraw::now_ms();
        if self.hidden.due(now)
            && let Some(g) = &self.graphics
        {
            g.window.request_redraw();
        }
        event_loop.set_control_flow(match self.hidden.wait_ms(now) {
            Some(ms) => ControlFlow::wait_duration(std::time::Duration::from_millis(ms)),
            None => ControlFlow::Wait,
        });
    }
}

struct Graphics {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    quad: PaletteQuad,
    image: IndexedImage,
    backend: &'static str,
    webgpu_extras: bool,
    palette_row: u32,
    palette_rows: u32,
    frame: u64,
    frames_presented: u64,
}

impl Graphics {
    async fn new(
        window: Arc<Window>,
        backend: BackendChoice,
        display: Option<Box<dyn wgpu::wgt::WgpuHasDisplayHandle>>,
    ) -> Result<Self, String> {
        let instance = gpu::create_instance(backend, display).await;
        let surface = instance
            .create_surface(window.clone())
            .map_err(|e| format!("could not create a drawing surface: {e}"))?;
        let adapter = gpu::request_adapter(&instance, Some(&surface))
            .await
            .map_err(|e| e.to_string())?;
        let info = adapter.get_info();
        let webgpu_extras = gpu::extras_available(&adapter);
        let (device, queue) = gpu::request_device(&adapter)
            .await
            .map_err(|e| e.to_string())?;

        let size = window.inner_size();
        let mut config = surface
            .get_default_config(&adapter, size.width.max(1), size.height.max(1))
            .ok_or("the surface is not supported by this adapter")?;
        let caps = surface.get_capabilities(&adapter);
        if let Some(srgb) = caps.formats.iter().copied().find(|f| f.is_srgb()) {
            config.format = srgb;
        }
        config.present_mode = wgpu::PresentMode::AutoVsync;

        let image = checkerboard(IMAGE_SIZE, IMAGE_SIZE, CHECKER_CELL);
        let palettes = [ramp_palette(0), ramp_palette(7)];
        let quad = PaletteQuad::new(&device, &queue, config.format, &image, &palettes)
            .map_err(|e| e.to_string())?;

        let mut graphics = Self {
            window,
            surface,
            device,
            queue,
            config,
            quad,
            image,
            backend: gpu::backend_label(info.backend),
            webgpu_extras,
            palette_row: 0,
            palette_rows: palettes.len() as u32,
            frame: 0,
            frames_presented: 0,
        };
        graphics.resize(size.width, size.height);
        Ok(graphics)
    }

    /// Reconfigures the surface, clamped to at least 1×1 and to the device's
    /// texture limit (a HiDPI canvas can exceed the WebGL2 floor of 2048).
    fn resize(&mut self, width: u32, height: u32) {
        let max = self.device.limits().max_texture_dimension_2d;
        self.config.width = width.clamp(1, max);
        self.config.height = height.clamp(1, max);
        self.surface.configure(&self.device, &self.config);
    }

    fn quad_params(&self) -> QuadParams {
        // Largest whole-number zoom that keeps the image within 80 % of the
        // shorter side, so pixels stay square and crisp.
        let (w, h) = (self.config.width as f32, self.config.height as f32);
        let (iw, ih) = (self.image.width() as f32, self.image.height() as f32);
        let fit = 0.8 * (w / iw).min(h / ih);
        let zoom = if fit >= 1.0 { fit.floor() } else { fit };
        QuadParams {
            scale: [zoom * iw / w, zoom * ih / h],
            palette_row: self.palette_row,
            cycle: (self.frame / CYCLE_EVERY_FRAMES) as u32,
        }
    }

    /// Draws a frame, and says what came of the try (#99).
    fn render(&mut self) -> Result<redraw::Drawn, String> {
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t)
            | wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
            wgpu::CurrentSurfaceTexture::Timeout => return Ok(redraw::Drawn::Again),
            wgpu::CurrentSurfaceTexture::Occluded => return Ok(redraw::Drawn::Hidden),
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.config);
                return Ok(redraw::Drawn::Again);
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err("the surface texture failed validation".into());
            }
        };
        self.quad.set_params(&self.queue, &self.quad_params());
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("frame"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.02,
                            g: 0.02,
                            b: 0.03,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            self.quad.draw(&mut pass);
        }
        self.queue.submit([encoder.finish()]);
        self.window.pre_present_notify();
        self.queue.present(frame);
        self.frame += 1;
        self.frames_presented += 1;
        Ok(redraw::Drawn::Yes)
    }
}
