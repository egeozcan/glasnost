//! The game client's window: winit events into [`Client`], its frames
//! through the 2D batch (`glasnost_render::batch2d`) onto the surface, and
//! its jobs run natively at once (after the frame that shows the work
//! starting) or on the web on the page's event loop.

use std::sync::Arc;

use glasnost_render::batch2d::Batch2d;
use glasnost_render::gpu::{self, BackendChoice};
use glasnost_render::map_gpu::{COLOR_FORMAT, MapRenderer, targets};
use glasnost_render::wgpu;
use glasnost_sim::targeting::Modifiers;
use glasnost_ui::geom::Point;
use glasnost_vfs::PickedFile;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
use winit::keyboard::{self, ModifiersState, NamedKey};
use winit::window::{Window, WindowId};

use super::client::{Client, Done, Key, run_job};
use super::play::Button;
use super::session::{MAP_IMAGE, Session};
use crate::Status;
use glasnost_ui::shell::Screen;

/// Options for [`run_game`].
#[derive(Clone, Debug, Default)]
pub struct GameOptions {
    /// Backend policy.
    pub backend: BackendChoice,
    /// Install folder to read at start (native).
    pub game_dir: Option<String>,
    /// Close after presenting this many frames (smoke tests).
    pub exit_after_frames: Option<u64>,
    /// The relay address the multiplayer screen starts with (`--relay`,
    /// `?relay=`); `None`: [`super::link::DEFAULT_RELAY`].
    pub relay: Option<String>,
    /// How the client opens a link to a relay, when the platform gives one
    /// (the web shell's WebSocket); `None`: the built-in native link.
    pub connect: Option<super::link::Connect>,
    /// How the client stores a desync report, when the platform gives one
    /// (the web shell's download); `None`: the built-in native file.
    pub report: Option<super::report::SaveReport>,
    /// A test client ([`Client::set_test`]) with this perturbation, if any
    /// (the native client in a debug build with `GLASNOST_PERTURB` set).
    pub test: Option<Option<glasnost_net::desync::Perturb>>,
}

/// Events delivered to the game's loop.
enum GameEvent {
    Graphics(Box<Result<Gfx, String>>),
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    Done(Box<Done>),
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    Picked(Vec<PickedFile>),
}

/// Opens the game window. Native: blocks until it closes. Web: starts the
/// loop on the browser's and returns.
pub fn run_game(options: GameOptions) -> Result<(), winit::error::EventLoopError> {
    let event_loop = EventLoop::<GameEvent>::with_user_event().build()?;
    let proxy = event_loop.create_proxy();
    let mut client = Client::new();
    if let Some(store) = crate::settings::platform_store() {
        client.attach_store(store);
    }
    client.can_quit = !cfg!(target_arch = "wasm32");
    if let Some(r) = options.relay.as_deref().filter(|r| !r.is_empty()) {
        client.netform.relay = r.to_owned();
    }
    if let Some(c) = options.connect {
        client.connect = Some(c);
    }
    if let Some(r) = options.report {
        client.save_report = Some(r);
        client.save_path_report = Some(r);
    }
    if let Some(p) = options.test {
        client.set_test(p);
    }
    if cfg!(not(target_arch = "wasm32")) && options.game_dir.is_none() {
        client.status = "Start with --game-dir DIR to read an install.".into();
    }
    let app = GameApp {
        options,
        client,
        proxy,
        window: None,
        gfx: None,
        started: false,
        mods: ModifiersState::empty(),
        pointer: Point::new(0, 0),
        cursor_hidden: false,
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

/// Host milliseconds since the program started (pacing only; never game
/// state).
fn now_ms() -> i64 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::sync::OnceLock;
        static START: OnceLock<std::time::Instant> = OnceLock::new();
        START
            .get_or_init(std::time::Instant::now)
            .elapsed()
            .as_millis() as i64
    }
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .and_then(|w| w.performance())
            .map_or(0, |p| p.now() as i64)
    }
}

/// A seed for the next game from the host clock (S-0202 claim 8: the game
/// starts from one seed; how a skirmish picks it is the client's).
fn host_seed() -> u32 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(1, |d| d.as_nanos() as u32)
            | 1
    }
    #[cfg(target_arch = "wasm32")]
    {
        (js_sys::Date::now() as u64 as u32) | 1
    }
}

struct GameApp {
    options: GameOptions,
    client: Client,
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    proxy: EventLoopProxy<GameEvent>,
    window: Option<Arc<Window>>,
    gfx: Option<Gfx>,
    started: bool,
    mods: ModifiersState,
    pointer: Point,
    /// The system pointer is hidden for the game's own cursor.
    cursor_hidden: bool,
}

impl GameApp {
    fn modifiers(&self) -> Modifiers {
        // spec: S-0214 claim 23 (Alt force-move, Ctrl force-fire, Shift force-select, either side; no Cmd binding)
        Modifiers {
            force_select: self.mods.shift_key(),
            force_fire: self.mods.control_key(),
            force_move: self.mods.alt_key(),
            attack_move_mode: false,
        }
    }

    /// Hands the client's pending job to the runner: on the web at once,
    /// natively when `now` (after a presented frame, so the status shows).
    fn run_jobs(&mut self, now: bool) {
        #[cfg(target_arch = "wasm32")]
        {
            let _ = now;
            if let Some(job) = self.client.job.take() {
                let proxy = self.proxy.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let done = run_job(job).await;
                    let _ = proxy.send_event(GameEvent::Done(Box::new(done)));
                });
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        if now && let Some(job) = self.client.job.take() {
            let done = pollster::block_on(run_job(job));
            self.client.done(done);
        }
    }

    fn after_input(&mut self) {
        if std::mem::take(&mut self.client.pick_install) {
            #[cfg(target_arch = "wasm32")]
            crate::viewer::web::open_folder_picker();
            #[cfg(not(target_arch = "wasm32"))]
            self.native_mount();
        }
        self.run_jobs(false);
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn native_mount(&mut self) {
        let Some(dir) = self.options.game_dir.clone() else {
            self.client.status = "Start with --game-dir DIR to read an install.".into();
            return;
        };
        match crate::viewer::tasks::native_files(std::path::Path::new(&dir)) {
            Ok(files) => self.client.mount(files),
            Err(e) => self.client.status = e,
        }
    }
}

impl ApplicationHandler<GameEvent> for GameApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attributes = Window::default_attributes().with_title("Glasnost");
        // The page's style sizes the canvas on the web.
        #[cfg(not(target_arch = "wasm32"))]
        let attributes = attributes.with_inner_size(winit::dpi::LogicalSize::new(1024.0, 768.0));
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
        let proxy = self.proxy.clone();
        #[cfg(target_arch = "wasm32")]
        {
            let picked = proxy.clone();
            crate::viewer::web::on_folder_picked(move |files| {
                let _ = picked.send_event(GameEvent::Picked(files));
            });
        }
        let backend = self.options.backend;
        #[cfg(not(target_arch = "wasm32"))]
        {
            let display: Box<dyn wgpu::wgt::WgpuHasDisplayHandle> =
                Box::new(event_loop.owned_display_handle());
            let result = pollster::block_on(Gfx::new(window, backend, Some(display)));
            let _ = proxy.send_event(GameEvent::Graphics(Box::new(result)));
        }
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(async move {
            let result = Gfx::new(window, backend, None).await;
            let _ = proxy.send_event(GameEvent::Graphics(Box::new(result)));
        });
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: GameEvent) {
        match event {
            GameEvent::Graphics(result) => match *result {
                Ok(gfx) => {
                    log::info!("game rendering with {}", gfx.backend);
                    // The battlefield is drawn on the GPU from here on.
                    self.client.cpu_map = false;
                    self.client
                        .resize((gfx.config.width as i32, gfx.config.height as i32));
                    gfx.window.request_redraw();
                    self.gfx = Some(gfx);
                    if !self.started {
                        self.started = true;
                        #[cfg(not(target_arch = "wasm32"))]
                        if self.options.game_dir.is_some() {
                            self.native_mount();
                        }
                    }
                }
                Err(e) => {
                    Status::Error(e).publish();
                    event_loop.exit();
                }
            },
            GameEvent::Done(d) => self.client.done(*d),
            GameEvent::Picked(files) => self.client.mount(files),
        }
        self.run_jobs(false);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(g) = &mut self.gfx {
                    g.resize(size.width, size.height);
                    self.client
                        .resize((g.config.width as i32, g.config.height as i32));
                }
            }
            WindowEvent::Focused(on) => self.client.focus(on),
            WindowEvent::ModifiersChanged(m) => {
                self.mods = m.state();
                let mods = self.modifiers();
                self.client.modifiers(mods);
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.pointer = Point::new(position.x as i32, position.y as i32);
                self.client.pointer_moved(self.pointer);
            }
            WindowEvent::CursorLeft { .. } => self.client.pointer_left(),
            WindowEvent::MouseWheel { delta, .. } => {
                let dy = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => y as f64,
                    winit::event::MouseScrollDelta::PixelDelta(p) => p.y,
                };
                // Host input only: the wheel's direction (S-0411 claim 18).
                if dy != 0.0 {
                    self.client.wheel(if dy > 0.0 { 1 } else { -1 });
                }
                self.after_input();
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let b = match button {
                    MouseButton::Left => Button::Left,
                    MouseButton::Right => Button::Right,
                    _ => return,
                };
                match state {
                    ElementState::Pressed => self.client.pointer_down(b, self.pointer),
                    ElementState::Released => {
                        self.client.next_seed = host_seed();
                        let mods = self.modifiers();
                        self.client.pointer_up(b, self.pointer, mods);
                    }
                }
                self.after_input();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let down = event.state == ElementState::Pressed;
                // Auto-repeated key-downs are not new presses: holding an arrow
                // first pressed during an order block must not become an
                // accepted press afterwards (S-0213 claim 23).
                let key = match &event.logical_key {
                    // Backspace repeats while held, as in any text field
                    // (and only there: in the game it holds or cancels).
                    keyboard::Key::Named(NamedKey::Backspace)
                        if !event.repeat || self.client.typing_text() =>
                    {
                        Some(Key::Backspace)
                    }
                    _ if event.repeat => None,
                    keyboard::Key::Named(NamedKey::Enter) => Some(Key::Enter),
                    keyboard::Key::Named(NamedKey::Tab) => Some(Key::Tab),
                    keyboard::Key::Named(NamedKey::Space) => Some(Key::Space),
                    keyboard::Key::Named(NamedKey::PageUp) => Some(Key::PageUp),
                    keyboard::Key::Named(NamedKey::PageDown) => Some(Key::PageDown),
                    keyboard::Key::Named(NamedKey::Escape) => Some(Key::Escape),
                    keyboard::Key::Named(NamedKey::ArrowLeft) => Some(Key::Left),
                    keyboard::Key::Named(NamedKey::ArrowRight) => Some(Key::Right),
                    keyboard::Key::Named(NamedKey::ArrowUp) => Some(Key::Up),
                    keyboard::Key::Named(NamedKey::ArrowDown) => Some(Key::Down),
                    keyboard::Key::Character(s) => {
                        s.chars().next().map(|c| Key::Char(c.to_ascii_lowercase()))
                    }
                    _ => None,
                };
                if let Some(k) = key {
                    self.client.key(k, down);
                    self.after_input();
                }
                // The text a press types, for the network screens' fields
                // and the chat.
                if down && let Some(text) = event.text.as_ref() {
                    self.client.typed(text);
                }
            }
            WindowEvent::RedrawRequested => {
                self.client.update(now_ms());
                let Some(g) = &mut self.gfx else {
                    return;
                };
                if let Err(e) = g.render(&mut self.client) {
                    Status::Error(e).publish();
                    event_loop.exit();
                    return;
                }
                let hide = self.client.draws_cursor();
                if hide != self.cursor_hidden
                    && let Some(w) = &self.window
                {
                    w.set_cursor_visible(!hide);
                    self.cursor_hidden = hide;
                }
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
                    || self.client.quit
                {
                    event_loop.exit();
                    return;
                }
                g.window.request_redraw();
                self.run_jobs(true);
            }
            _ => {}
        }
        if self.client.quit {
            event_loop.exit();
        }
    }
}

/// The battlefield drawn on the GPU (`glasnost_render::map_gpu`) into a
/// target the batch shows as its image `MAP_IMAGE`.
struct MapGpu {
    serial: u64,
    renderer: MapRenderer,
    size: (u32, u32),
    targets: Option<(wgpu::Texture, wgpu::Texture)>,
}

/// The client's frames on a device: the 2D batch, and the battlefield on
/// the GPU unless the client composes it on the CPU. The window draws it to
/// its surface; the local lane draws it offscreen.
pub struct GpuDraw {
    batch: Batch2d,
    map: Option<MapGpu>,
}

impl GpuDraw {
    /// A drawer for targets of `format`.
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        Self {
            batch: Batch2d::new(device, format),
            map: None,
        }
    }

    /// Takes the client's frame for a target of `size`: records the
    /// battlefield into `encoder` (falling back to the CPU compose for good
    /// when that fails) and uploads and prepares the batch. Draw it with
    /// [`GpuDraw::draw`] after `encoder`'s work.
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        client: &mut Client,
        size: (u32, u32),
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let (list, images) = client.frame();
        if !client.cpu_map
            && client.shell.screen == Screen::Game
            && let Some(s) = client.session.as_ref()
            && let Err(e) = self.render_map(device, queue, s, client.game_serial, encoder)
        {
            log::warn!("the battlefield is drawn on the CPU from now on: {e}");
            client.cpu_map = true;
        }
        let painter = &mut client.painter;
        if painter.dirty {
            self.batch.set_palettes(device, queue, &painter.palettes);
            painter.dirty = false;
        }
        self.batch.upload_atlas(queue, &mut painter.atlas);
        for (id, w, h, rgba) in &images.changed {
            self.batch.set_image(device, queue, *id, *w, *h, rgba);
        }
        let quads = painter.quads(&list);
        self.batch.prepare(device, queue, size, &quads);
    }

    /// Records the prepared frame into `pass`.
    pub fn draw(&self, pass: &mut wgpu::RenderPass<'_>) {
        self.batch.draw(pass);
    }

    /// Draws the session's battlefield into the map target.
    fn render_map(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        s: &Session,
        serial: u64,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Result<(), String> {
        // A game's scene only grows: upload what it gained, or all of it for
        // a new game or when the atlas is full.
        let synced = match self.map.as_mut() {
            Some(m) if m.serial == serial => m.renderer.sync(device, queue, &s.scene).is_ok(),
            _ => false,
        };
        if !synced {
            let renderer = MapRenderer::new(device, queue, &s.scene, COLOR_FORMAT)
                .map_err(|e| e.to_string())?;
            self.map = Some(MapGpu {
                serial,
                renderer,
                size: (0, 0),
                targets: None,
            });
        }
        let m = self.map.as_mut().ok_or("no map renderer")?;
        let v = s.view();
        if m.size != v.size || m.targets.is_none() {
            let (color, depth) = targets(device, v.size, COLOR_FORMAT);
            self.batch.set_image_view(
                device,
                MAP_IMAGE,
                &color.create_view(&Default::default()),
                v.size,
            );
            m.targets = Some((color, depth));
            m.size = v.size;
        }
        let (color, depth) = m.targets.as_ref().ok_or("no map targets")?;
        let alpha = s.light_alpha();
        m.renderer.render_lit(
            device,
            queue,
            encoder,
            color,
            &depth.create_view(&Default::default()),
            &s.scene,
            &v,
            s.filter(),
            &glasnost_render::compose::ComposeQuirks::default(),
            Some(&alpha),
        );
        Ok(())
    }
}

struct Gfx {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    draw: GpuDraw,
    backend: &'static str,
    webgpu_extras: bool,
    frames_presented: u64,
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
        // The batch blends in gamma space, as the CPU reference does; a
        // non-sRGB target keeps its colours exact.
        if let Some(linear) = caps.formats.iter().copied().find(|f| !f.is_srgb()) {
            config.format = linear;
        }
        config.present_mode = wgpu::PresentMode::AutoVsync;
        let draw = GpuDraw::new(&device, config.format);
        let mut gfx = Self {
            window,
            surface,
            device,
            queue,
            config,
            draw,
            backend: gpu::backend_label(info.backend),
            webgpu_extras,
            frames_presented: 0,
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

    fn render(&mut self, client: &mut Client) -> Result<(), String> {
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t)
            | wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.config);
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err("the surface texture failed validation".into());
            }
        };
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("game frame"),
            });
        self.draw.prepare(
            &self.device,
            &self.queue,
            client,
            (self.config.width, self.config.height),
            &mut encoder,
        );
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("game frame"),
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
            self.draw.draw(&mut pass);
        }
        self.queue.submit([encoder.finish()]);
        self.window.pre_present_notify();
        self.queue.present(frame);
        self.frames_presented += 1;
        Ok(())
    }
}
