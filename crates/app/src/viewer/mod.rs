//! The asset viewer (milestone M1): browse a mounted install, show sprites,
//! tiles, voxels, images and palettes with a chosen palette and house-colour
//! rows, and play sounds. One code path for native and web: winit for the
//! window and input, wgpu through `glasnost-render` for the device, egui for
//! the development UI.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use glasnost_render::gpu::{self, BackendChoice};
use glasnost_render::wgpu;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
use winit::window::{Window, WindowId};

use crate::Status;

mod browser;
mod input;
mod map_display;
mod map_pane;
pub mod preview;
mod rules;
pub(crate) mod tasks;
#[cfg(target_arch = "wasm32")]
pub(crate) mod web;

use browser::Browser;
use input::Input;
use tasks::{Spawner, Task, TaskResult};

/// Options for [`run_viewer`].
#[derive(Clone, Debug, Default)]
pub struct ViewerOptions {
    /// Backend policy.
    pub backend: BackendChoice,
    /// Install folder to mount at start (native).
    pub game_dir: Option<String>,
    /// Close after presenting this many frames (smoke tests).
    pub exit_after_frames: Option<u64>,
    /// Open the map viewer on the first map whose name contains this text
    /// (letter case ignored), once the install is mounted.
    pub open_map: Option<String>,
}

/// Events delivered to the viewer's loop.
enum ViewerEvent {
    Graphics(Box<Result<Gfx, String>>),
    Task(TaskResult),
}

/// Opens the viewer window. Native: blocks until it closes. Web: starts the
/// loop on the browser's and returns.
pub fn run_viewer(options: ViewerOptions) -> Result<(), winit::error::EventLoopError> {
    let event_loop = EventLoop::<ViewerEvent>::with_user_event().build()?;
    let proxy = event_loop.create_proxy();
    let app = ViewerApp {
        browser: {
            let mut b = Browser::new(options.game_dir.clone());
            b.pending_map = options.open_map.clone();
            b
        },
        options,
        spawner: Rc::new(LoopSpawner {
            proxy: proxy.clone(),
            queue: RefCell::new(Vec::new()),
        }),
        proxy: Some(proxy),
        window: None,
        gfx: None,
        egui: egui::Context::default(),
        input: None,
        started: false,
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

/// Runs tasks: natively to completion at once (reads are local and quick),
/// on the web on the page's event loop. Results arrive as loop events.
struct LoopSpawner {
    proxy: EventLoopProxy<ViewerEvent>,
    /// Native results waiting to be sent (sent after the current handler).
    queue: RefCell<Vec<TaskResult>>,
}

impl Spawner for LoopSpawner {
    fn spawn(&self, task: Task) {
        #[cfg(not(target_arch = "wasm32"))]
        self.queue.borrow_mut().push(pollster::block_on(task));
        #[cfg(target_arch = "wasm32")]
        {
            let proxy = self.proxy.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let _ = proxy.send_event(ViewerEvent::Task(task.await));
            });
        }
    }
}

impl LoopSpawner {
    fn flush(&self) {
        for r in self.queue.borrow_mut().drain(..) {
            let _ = self.proxy.send_event(ViewerEvent::Task(r));
        }
    }
}

struct ViewerApp {
    options: ViewerOptions,
    browser: Browser,
    spawner: Rc<LoopSpawner>,
    proxy: Option<EventLoopProxy<ViewerEvent>>,
    window: Option<Arc<Window>>,
    gfx: Option<Gfx>,
    egui: egui::Context,
    input: Option<Input>,
    started: bool,
}

impl ApplicationHandler<ViewerEvent> for ViewerApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let Some(proxy) = self.proxy.take() else {
            return;
        };
        let attributes = Window::default_attributes().with_title("Glasnost: asset viewer");
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
        configure_fonts(&self.egui);
        #[cfg(target_arch = "wasm32")]
        web::install_folder_picker(proxy.clone());
        let backend = self.options.backend;
        #[cfg(not(target_arch = "wasm32"))]
        {
            let display: Box<dyn wgpu::wgt::WgpuHasDisplayHandle> =
                Box::new(event_loop.owned_display_handle());
            let result = pollster::block_on(Gfx::new(window, backend, Some(display)));
            let _ = proxy.send_event(ViewerEvent::Graphics(Box::new(result)));
        }
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(async move {
            let result = Gfx::new(window, backend, None).await;
            let _ = proxy.send_event(ViewerEvent::Graphics(Box::new(result)));
        });
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: ViewerEvent) {
        match event {
            ViewerEvent::Graphics(result) => match *result {
                Ok(gfx) => {
                    log::info!("viewer rendering with {}", gfx.backend);
                    let size = gfx.window.inner_size();
                    let mut input =
                        Input::new((size.width, size.height), gfx.window.scale_factor() as f32);
                    input.set_max_texture_side(
                        gfx.device.limits().max_texture_dimension_2d as usize,
                    );
                    self.input = Some(input);
                    gfx.window.request_redraw();
                    self.gfx = Some(gfx);
                    if !self.started {
                        self.started = true;
                        #[cfg(not(target_arch = "wasm32"))]
                        if let Some(dir) = self.options.game_dir.clone() {
                            match tasks::native_files(std::path::Path::new(&dir)) {
                                Ok(files) => self.browser.mount(self.spawner.as_ref(), files),
                                Err(e) => log::warn!("{e}"),
                            }
                        }
                    }
                }
                Err(e) => {
                    Status::Error(e).publish();
                    event_loop.exit();
                }
            },
            ViewerEvent::Task(result) => {
                self.browser.on_task(result, self.spawner.as_ref());
                self.request_redraw();
            }
        }
        self.spawner.flush();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if let Some(input) = &mut self.input
            && input.on_event(&event)
        {
            self.request_redraw();
        }
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(g) = &mut self.gfx {
                    g.resize(size.width, size.height);
                }
            }
            WindowEvent::RedrawRequested => {
                if let Err(e) = self.frame() {
                    Status::Error(e).publish();
                    event_loop.exit();
                    return;
                }
                if let Some(g) = &self.gfx {
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
                    }
                }
            }
            _ => {}
        }
        self.spawner.flush();
    }
}

impl ViewerApp {
    fn request_redraw(&self) {
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }

    fn frame(&mut self) -> Result<(), String> {
        let (Some(gfx), Some(input)) = (&mut self.gfx, &mut self.input) else {
            return Ok(());
        };
        let raw = input.take(input::now_seconds());
        let backend = gfx.backend;
        let request = if self.browser.map_mode {
            self.browser.map.gpu_request()
        } else {
            None
        };
        let texture = request.and_then(|r| match gfx.render_map(&r) {
            Ok(id) => Some(id),
            Err(e) => {
                // Not built again on the next frame (#110).
                self.browser.map.gpu_failed(&e);
                None
            }
        });
        self.browser.map.set_gpu_texture(texture);
        let browser = &mut self.browser;
        let spawner = self.spawner.as_ref();
        let output = self.egui.run_ui(raw, |ui| browser.ui(ui, spawner, backend));
        let repaint_now = output
            .viewport_output
            .get(&egui::ViewportId::ROOT)
            .is_some_and(|v| v.repaint_delay.is_zero());
        gfx.paint(&self.egui, output)?;
        if repaint_now || self.options.exit_after_frames.is_some() {
            gfx.window.request_redraw();
        }
        Ok(())
    }
}

/// The UI font: Droid Sans (Apache-2.0, see `assets/fonts/NOTICE-DroidSans.txt`).
/// egui's default fonts are not used: their licences are outside the
/// project's dependency policy.
fn configure_fonts(ctx: &egui::Context) {
    const DROID_SANS: &[u8] = include_bytes!("../../assets/fonts/DroidSans.ttf");
    let mut fonts = egui::FontDefinitions::empty();
    fonts.font_data.insert(
        "droid-sans".into(),
        Arc::new(egui::FontData::from_static(DROID_SANS)),
    );
    for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
        fonts
            .families
            .entry(family)
            .or_default()
            .push("droid-sans".into());
    }
    ctx.set_fonts(fonts);
}

/// The window's GPU state and egui renderer.
struct Gfx {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    renderer: egui_wgpu::Renderer,
    backend: &'static str,
    webgpu_extras: bool,
    frames_presented: u64,
    map: Option<MapGpu>,
}

/// The GPU map pass's resources: the uploaded scene, its targets and what
/// egui samples.
struct MapGpu {
    generation: u64,
    renderer: glasnost_render::map_gpu::MapRenderer,
    size: (u32, u32),
    targets: Option<(wgpu::Texture, wgpu::Texture, map_display::MapDisplay)>,
    texture: Option<egui::TextureId>,
}

impl Gfx {
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
        // egui blends in gamma space; a non-sRGB target keeps its colours
        // exact. Fall back to whatever the surface offers.
        if let Some(linear) = caps.formats.iter().copied().find(|f| !f.is_srgb()) {
            config.format = linear;
        }
        config.present_mode = wgpu::PresentMode::AutoVsync;
        let renderer = egui_wgpu::Renderer::new(
            &device,
            config.format,
            egui_wgpu::RendererOptions {
                msaa_samples: 1,
                depth_stencil_format: None,
                dithering: false,
                predictable_texture_filtering: true,
            },
        );
        let mut gfx = Self {
            window,
            surface,
            device,
            queue,
            config,
            renderer,
            backend: gpu::backend_label(info.backend),
            webgpu_extras,
            frames_presented: 0,
            map: None,
        };
        gfx.resize(size.width, size.height);
        Ok(gfx)
    }

    fn resize(&mut self, width: u32, height: u32) {
        let max = self.device.limits().max_texture_dimension_2d;
        self.config.width = width.clamp(1, max);
        self.config.height = height.clamp(1, max);
        self.surface.configure(&self.device, &self.config);
    }

    /// Draws the map view on the GPU and returns the egui texture holding
    /// it.
    fn render_map(&mut self, r: &map_pane::GpuRequest) -> Result<egui::TextureId, String> {
        use glasnost_render::map_gpu::{COLOR_FORMAT, MapRenderer, targets};
        // A scene of the same generation only grows (images and palette
        // rows added for the live objects): upload what it gained, or the
        // whole scene again when the atlas is full.
        let synced = match self.map.as_mut() {
            Some(m) if m.generation == r.generation => {
                match m.renderer.sync(&self.device, &self.queue, &r.scene) {
                    Ok(()) => true,
                    Err(e) => {
                        log::info!("map scene uploaded again: {e}");
                        false
                    }
                }
            }
            _ => false,
        };
        if !synced {
            if let Some(old) = self.map.take()
                && let Some(id) = old.texture
            {
                self.renderer.free_texture(&id);
            }
            let renderer = MapRenderer::new(&self.device, &self.queue, &r.scene, COLOR_FORMAT)
                .map_err(|e| e.to_string())?;
            log::info!(
                "map scene uploaded: {} items, {} images",
                r.scene.items.len(),
                r.scene.images.len()
            );
            self.map = Some(MapGpu {
                generation: r.generation,
                renderer,
                size: (0, 0),
                targets: None,
                texture: None,
            });
        }
        let device = &self.device;
        let m = self.map.as_mut().ok_or("no map")?;
        let size = r.view.size;
        if m.size != size || m.targets.is_none() {
            let (color, depth) = targets(device, size, COLOR_FORMAT);
            let display = map_display::MapDisplay::new(device, &color);
            match m.texture {
                Some(id) => self.renderer.update_egui_texture_from_wgpu_texture(
                    device,
                    display.view(),
                    wgpu::FilterMode::Nearest,
                    id,
                ),
                None => {
                    m.texture = Some(self.renderer.register_native_texture(
                        device,
                        display.view(),
                        wgpu::FilterMode::Nearest,
                    ))
                }
            }
            m.targets = Some((color, depth, display));
            m.size = size;
        }
        let (color, depth, display) = m.targets.as_ref().ok_or("no targets")?;
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("map") });
        m.renderer.render_lit(
            device,
            &self.queue,
            &mut encoder,
            color,
            &depth.create_view(&Default::default()),
            &r.scene,
            &r.view,
            &r.filter,
            &glasnost_render::compose::ComposeQuirks::default(),
            r.alpha.as_deref(),
        );
        display.record(&mut encoder, color);
        self.queue.submit([encoder.finish()]);
        if self.frames_presented.is_multiple_of(120) {
            log::info!(
                "map view drawn on the GPU at {:?}, {}×{}",
                r.view.origin,
                size.0,
                size.1
            );
        }
        m.texture.ok_or_else(|| "no texture".into())
    }

    fn free_textures<'a>(&mut self, ids: impl IntoIterator<Item = &'a egui::TextureId>) {
        for id in ids {
            self.renderer.free_texture(id);
        }
    }

    fn paint(&mut self, ctx: &egui::Context, mut output: egui::FullOutput) -> Result<(), String> {
        // Texture changes apply even when no frame can be drawn this time:
        // egui sends each texture once.
        for (id, deltas) in &output.textures_delta.set {
            for delta in deltas {
                self.renderer
                    .update_texture(&self.device, &self.queue, *id, delta);
            }
        }
        // Frees apply after the frame that may still draw them.
        let free = std::mem::take(&mut output.textures_delta.free);
        output.textures_delta.clear();
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t)
            | wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                self.free_textures(&free);
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.config);
                self.free_textures(&free);
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err("the surface texture failed validation".into());
            }
        };
        let ppp = output.pixels_per_point;
        let screen = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point: ppp,
        };
        let jobs = ctx.tessellate(output.shapes, ppp);
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("viewer"),
            });
        let extra =
            self.renderer
                .update_buffers(&self.device, &self.queue, &mut encoder, &jobs, &screen);
        {
            let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("viewer"),
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
            let mut pass = pass.forget_lifetime();
            self.renderer.render(&mut pass, &jobs, &screen);
        }
        self.queue
            .submit(extra.into_iter().chain(std::iter::once(encoder.finish())));
        self.free_textures(&free);
        self.window.pre_present_notify();
        self.queue.present(frame);
        self.frames_presented += 1;
        Ok(())
    }
}
