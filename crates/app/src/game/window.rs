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
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy};
use winit::keyboard::{self, ModifiersState, NamedKey};
use winit::window::{Window, WindowId};

use super::bench::{self, Bench, FrameEnd};
use super::client::{Client, Done, Key, run_job};
use super::play::Button;
use super::pump::Pump;
use super::session::{MAP_IMAGE, Session};
use super::touch;
use crate::Status;
use crate::redraw::{Drawn, Hidden, now_ms};
use glasnost_ui::shell::Screen;

/// The digit a digit key or a keypad digit key stands for.
fn digit_of(k: keyboard::PhysicalKey) -> Option<char> {
    use keyboard::KeyCode as C;
    let keyboard::PhysicalKey::Code(c) = k else {
        return None;
    };
    let n = match c {
        C::Digit0 | C::Numpad0 => 0,
        C::Digit1 | C::Numpad1 => 1,
        C::Digit2 | C::Numpad2 => 2,
        C::Digit3 | C::Numpad3 => 3,
        C::Digit4 | C::Numpad4 => 4,
        C::Digit5 | C::Numpad5 => 5,
        C::Digit6 | C::Numpad6 => 6,
        C::Digit7 | C::Numpad7 => 7,
        C::Digit8 | C::Numpad8 => 8,
        C::Digit9 | C::Numpad9 => 9,
        _ => return None,
    };
    char::from_digit(n, 10)
}

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
    /// How the client opens direct links to its game's peers, when the
    /// platform gives one (the web shell's RTCPeerConnection, M9 N7);
    /// `None`: the built-in one ([`super::direct::platform_direct`]).
    pub direct: Option<super::direct::OpenPairFn>,
    /// No direct links in this run (`--relay-only`, `?direct=off`): the
    /// relay carries everything.
    pub relay_only: bool,
    /// The direct links' STUN or TURN servers, space-separated URLs
    /// (`--ice`, `?ice=`); `None`: the settings' (none by default).
    pub ice: Option<String>,
    /// How the client stores a desync report, when the platform gives one
    /// (the web shell's download); `None`: the built-in native file.
    pub report: Option<super::report::SaveReport>,
    /// A test client ([`Client::set_test`]) with this perturbation, if any
    /// (the native client in a debug build with `GLASNOST_PERTURB` set).
    pub test: Option<Option<glasnost_net::desync::Perturb>>,
    /// Run the performance benchmark (`--bench`, `?bench=`, M8 step C4)
    /// instead of the menus, and close (natively) when it is done.
    pub bench: Option<super::bench::BenchOptions>,
    /// Mirror the game's view centre and selection count into the page
    /// (`?probe=1`, web only) for the browser checks of touch and touchpad
    /// scrolling: `data-glasnost-view` holds `x,y` and
    /// `data-glasnost-selected` the count, in a game.
    pub probe: bool,
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
    if let Some(d) = options.direct {
        client.direct = Some(std::rc::Rc::new(d));
    }
    if options.relay_only {
        client.direct = None;
    }
    if let Some(ice) = options.ice.as_deref() {
        client.ice = super::direct::Ice::from_urls(ice);
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
    let host = if cfg!(target_arch = "wasm32") {
        "web"
    } else {
        "native-window"
    };
    let bench = options.bench.clone().map(|o| Bench::new(o, host));
    if bench.is_some() {
        client.status = "Benchmark: pick the game folder.".into();
    }
    let app = GameApp {
        bench,
        options,
        client,
        proxy,
        window: None,
        gfx: None,
        started: false,
        mods: ModifiersState::empty(),
        pointer: Point::new(0, 0),
        cursor_hidden: false,
        pump: Pump::default(),
        probed: None,
        display_probed: String::new(),
        hidden: Hidden::default(),
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
    /// The performance benchmark's run, when one was asked for.
    bench: Option<Bench>,
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
    /// Keeps a network game going while no frames are drawn (the M9 exit
    /// run).
    pump: Pump,
    /// The probe last published ([`GameOptions::probe`]).
    probed: Option<((i32, i32), usize)>,
    /// The display probe last published ([`publish_display`]).
    display_probed: String,
    /// Whether the window is hidden, so that it asks for no redraw (#99).
    hidden: Hidden,
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

    /// Asks a hidden window for a redraw once it is shown, resized or
    /// focused (#99).
    fn wake(&mut self) {
        if self.hidden.wake()
            && let Some(g) = &self.gfx
        {
            g.window.request_redraw();
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

    /// Gives the client the window's drawable size and scale factor, after
    /// the start, a resize or a change of the scale factor (a window moved
    /// to another monitor, a browser zoom). The benchmark draws at its size
    /// in physical pixels, one view pixel each, whatever the display.
    fn fit_window(&mut self) {
        let (Some(g), Some(w)) = (&self.gfx, &self.window) else {
            return;
        };
        let physical = (g.config.width as i32, g.config.height as i32);
        if self.bench.is_some() {
            self.client.resize(physical);
        } else {
            self.client.set_window(physical, w.scale_factor());
        }
    }

    /// The view pixel under a host position in physical pixels: the one
    /// place mouse and touch positions are converted
    /// (`super::display::Display::point`).
    fn view_point(&self, p: winit::dpi::PhysicalPosition<f64>) -> Point {
        self.client.display().point(p.x, p.y)
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
        let attributes = match self.options.bench.as_ref() {
            // The benchmark draws at its size in physical pixels.
            Some(b) => {
                attributes.with_inner_size(winit::dpi::PhysicalSize::new(b.size.0, b.size.1))
            }
            None => attributes.with_inner_size(winit::dpi::LogicalSize::new(1024.0, 768.0)),
        };
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
        #[cfg(target_arch = "wasm32")]
        if let Some(b) = self.options.bench.as_ref() {
            let _ = window.request_inner_size(winit::dpi::PhysicalSize::new(b.size.0, b.size.1));
        }
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
        let timing = self.options.bench.as_ref().map(|b| b.vsync);
        #[cfg(not(target_arch = "wasm32"))]
        {
            let display: Box<dyn wgpu::wgt::WgpuHasDisplayHandle> =
                Box::new(event_loop.owned_display_handle());
            let result = pollster::block_on(Gfx::new(window, backend, Some(display), timing));
            let _ = proxy.send_event(GameEvent::Graphics(Box::new(result)));
        }
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(async move {
            let result = Gfx::new(window, backend, None, timing).await;
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
                    if let Some(b) = self.bench.as_mut() {
                        b.device = gfx.device_label.clone();
                        b.gpu_timer = gfx.timer.is_some();
                        b.set_size((gfx.config.width, gfx.config.height));
                    }
                    gfx.window.request_redraw();
                    self.gfx = Some(gfx);
                    self.fit_window();
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
                    if let Some(b) = self.bench.as_mut() {
                        b.set_size((g.config.width, g.config.height));
                    }
                }
                self.fit_window();
                self.wake();
            }
            WindowEvent::Occluded(false) => self.wake(),
            // design: readable text and display scale (lead's request 2026-11-10): a new scale factor (another monitor, a browser zoom) picks the factor again at once; the resize that follows sets the new size
            WindowEvent::ScaleFactorChanged { .. } => {
                if let (Some(g), Some(w)) = (&mut self.gfx, &self.window) {
                    let size = w.inner_size();
                    g.resize(size.width, size.height);
                }
                self.fit_window();
            }
            WindowEvent::Focused(on) => {
                self.client.focus(on);
                if on {
                    self.wake();
                }
            }
            WindowEvent::ModifiersChanged(m) => {
                self.mods = m.state();
                let mods = self.modifiers();
                self.client.modifiers(mods);
            }
            // design: touch and touchpad scrolling (lead's request 2026-11-10): mouse events made of touches are dropped while touches are handled
            WindowEvent::CursorMoved { .. } | WindowEvent::MouseInput { .. }
                if self.client.mouse_quiet(now_ms()) => {}
            WindowEvent::CursorMoved { position, .. } => {
                self.pointer = self.view_point(position);
                self.client.pointer_moved(self.pointer);
            }
            WindowEvent::CursorLeft { .. } => self.client.pointer_left(),
            WindowEvent::MouseWheel { delta, .. } => {
                // The client picks line steps or a swipe (S-0411 claim 18; touch scrolling).
                let d = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => {
                        touch::WheelDelta::Lines(f64::from(y))
                    }
                    winit::event::MouseScrollDelta::PixelDelta(p) => {
                        touch::WheelDelta::from_pixels(
                            (p.x, p.y),
                            self.window.as_ref().map_or(1.0, |w| w.scale_factor()),
                            cfg!(target_arch = "wasm32"),
                        )
                    }
                };
                // A swipe's pixels in view pixels (`super::display`).
                let d = self.client.display().wheel(d);
                self.client.wheel_event(d);
                self.after_input();
            }
            // design: touch and touchpad scrolling (lead's request 2026-11-10): touches work whatever the touchpad setting
            WindowEvent::Touch(t) => {
                use winit::event::TouchPhase as T;
                let phase = match t.phase {
                    T::Started => touch::Phase::Start,
                    T::Moved => touch::Phase::Move,
                    T::Ended => touch::Phase::End,
                    T::Cancelled => touch::Phase::Cancel,
                };
                let p = self.view_point(t.location);
                // A tap's click is a release, as the mouse's (S-0202 claim 8: the client picks the next game's seed).
                self.client.next_seed = host_seed();
                let mods = self.modifiers();
                self.client.touch(phase, t.id, p, now_ms(), mods);
                self.after_input();
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let b = match button {
                    MouseButton::Left => Button::Left,
                    MouseButton::Right => Button::Right,
                    MouseButton::Middle => {
                        self.client.middle_button(self.pointer);
                        self.after_input();
                        return;
                    }
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
                // A digit key is its digit whatever Shift or the layout
                // makes of it, for the control-group keys (Shift+digit).
                let digit = digit_of(event.physical_key);
                let key = match &event.logical_key {
                    _ if event.repeat && digit.is_some() => None,
                    _ if digit.is_some() => digit.map(Key::Char),
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
                    keyboard::Key::Named(NamedKey::Delete) => Some(Key::Delete),
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
                let timing = match self.bench.as_mut() {
                    Some(b) if !b.finished() => {
                        b.before_frame(&mut self.client);
                        b.running().then(|| b.frame())
                    }
                    Some(_) => None,
                    None => {
                        let now = now_ms();
                        self.client.update(now);
                        self.pump.drew(now);
                        None
                    }
                };
                let Some(g) = &mut self.gfx else {
                    return;
                };
                let (end, drawn) = match g.render(&mut self.client, timing) {
                    Ok(r) => r,
                    Err(e) => {
                        Status::Error(e).publish();
                        event_loop.exit();
                        return;
                    }
                };
                if let Some(b) = self.bench.as_mut() {
                    if !b.finished() {
                        b.after_frame(&self.client, end, drawn == Drawn::Yes);
                    }
                    // A run that failed before its frames ends here too.
                    if b.take_report() {
                        g.finish_timer(b);
                        finish_bench(b, self.options.report);
                        if cfg!(not(target_arch = "wasm32")) {
                            event_loop.exit();
                            return;
                        }
                    }
                }
                if self.options.probe {
                    publish_probe(&self.client, &mut self.probed);
                    publish_display(&self.client, &mut self.display_probed);
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
                // A hidden window asks for no redraw; it waits (#99).
                if self.hidden.after(drawn, now_ms()) {
                    g.window.request_redraw();
                }
                self.run_jobs(true);
            }
            _ => {}
        }
        if self.client.quit {
            event_loop.exit();
        }
    }

    /// Between events: while a network link is open, the client is
    /// updated even when the window draws no frames (covered, minimised, a
    /// background tab), and the loop wakes on a timer for it; otherwise the
    /// loop waits for events as before. While the window is hidden, it also
    /// wakes every `redraw::RETRY_MS` to try to draw again (#99). The
    /// benchmark's frames are otherwise left alone.
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let now = now_ms();
        if self.hidden.due(now)
            && let Some(g) = &self.gfx
        {
            g.window.request_redraw();
        }
        let hidden_wait = self.hidden.wait_ms(now);
        if self.bench.is_some() {
            event_loop.set_control_flow(match hidden_wait {
                Some(ms) => ControlFlow::wait_duration(std::time::Duration::from_millis(ms)),
                None => ControlFlow::Wait,
            });
            return;
        }
        if self.pump.between_frames(&mut self.client, now) {
            self.run_jobs(true);
            if self.client.quit {
                event_loop.exit();
                return;
            }
        }
        let wait = match (self.pump.wait_ms(&self.client, now), hidden_wait) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        };
        event_loop.set_control_flow(match wait {
            Some(ms) => ControlFlow::wait_duration(std::time::Duration::from_millis(ms)),
            None => ControlFlow::Wait,
        });
    }
}

/// The battlefield drawn on the GPU (`glasnost_render::map_gpu`) into a
/// target the batch shows as its image `MAP_IMAGE`.
struct MapGpu {
    serial: u64,
    renderer: MapRenderer,
    size: (u32, u32),
    targets: Option<(wgpu::Texture, wgpu::Texture)>,
    /// The last frame's light alpha with the covers and view it came from:
    /// the values depend on nothing else in a game, so a frame with the
    /// same covers and view keeps them (M8 step C4).
    light: Option<LightKept>,
}

/// Light-alpha values kept between frames, with what they came from.
struct LightKept {
    covers: Vec<glasnost_render::shroud::Cover>,
    origin: (i32, i32),
    size: (u32, u32),
    alpha: Vec<u8>,
}

/// The client's frames on a device: the 2D batch, and the battlefield on
/// the GPU unless the client composes it on the CPU. The window draws it to
/// its surface; the local lane draws it offscreen.
pub struct GpuDraw {
    batch: Batch2d,
    map: Option<MapGpu>,
    /// A host clock in milliseconds, set only by the performance benchmark
    /// (M8 step C4): each [`GpuDraw::prepare`] then records its
    /// [`DrawLaps`].
    pub lap_clock: Option<fn() -> f64>,
    /// The last prepare's laps, when [`GpuDraw::lap_clock`] is set.
    pub laps: DrawLaps,
    rebuilds: usize,
    /// The last prepare's size in view pixels.
    size: (u32, u32),
    /// Screen pixels per view pixel and the target's size in screen
    /// pixels ([`GpuDraw::set_screen`]).
    screen: (u32, (u32, u32)),
}

/// Host-clock times of one [`GpuDraw::prepare`]'s parts, in milliseconds,
/// for the performance benchmark (M8 step C4; our own tooling).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DrawLaps {
    /// The client's draw list and changed images ([`Client::frame`]).
    pub hud: f64,
    /// The battlefield's scene uploads ([`MapRenderer::sync`], or a new
    /// renderer).
    pub sync: f64,
    /// The view's light alpha after the shroud covers.
    pub light: f64,
    /// The battlefield's draw recorded ([`MapRenderer::render_lit`]).
    pub map: f64,
    /// The batch's uploads, quads and buffers.
    pub batch: f64,
}

/// The renderer's counts since the last [`GpuDraw::take_stats`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DrawStats {
    /// The battlefield's.
    pub map: glasnost_render::map_gpu::MapStats,
    /// The 2D batch's.
    pub batch: glasnost_render::batch2d::BatchStats,
    /// Battlefield renderers made anew (a new game, or a full atlas), each
    /// uploading the whole scene.
    pub rebuilds: usize,
}

impl GpuDraw {
    /// A drawer for targets of `format`.
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        Self {
            batch: Batch2d::new(device, format),
            map: None,
            lap_clock: None,
            laps: DrawLaps::default(),
            rebuilds: 0,
            size: (1, 1),
            screen: (1, (1, 1)),
        }
    }

    /// Draws each view pixel as `factor` × `factor` screen pixels on a
    /// target of `target` screen pixels, from its top left corner
    /// (`super::display`; design: readable text and display scale (lead's
    /// request 2026-11-10)). The default, factor 1, draws view pixels as
    /// they are.
    pub fn set_screen(&mut self, factor: u32, target: (u32, u32)) {
        self.screen = (factor.max(1), target);
    }

    /// The counts since the last call (draws, instances, uploads).
    pub fn take_stats(&mut self) -> DrawStats {
        DrawStats {
            map: self
                .map
                .as_mut()
                .map(|m| m.renderer.take_stats())
                .unwrap_or_default(),
            batch: self.batch.take_stats(),
            rebuilds: std::mem::take(&mut self.rebuilds),
        }
    }

    fn lap(&self) -> f64 {
        self.lap_clock.map_or(0.0, |c| c())
    }

    /// Takes the client's frame for a view of `size` view pixels: records the
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
        let start = self.lap();
        let (list, images) = client.frame();
        self.laps = DrawLaps {
            hud: self.lap() - start,
            ..DrawLaps::default()
        };
        if !client.cpu_map
            && client.shell.screen == Screen::Game
            && let Some(s) = client.session.as_ref()
            && let Err(e) = self.render_map(device, queue, s, client.game_serial, encoder)
        {
            log::warn!("the battlefield is drawn on the CPU from now on: {e}");
            client.cpu_map = true;
        }
        let batched = self.lap();
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
        self.size = size;
        self.batch.prepare(device, queue, size, &quads);
        self.laps.batch = self.lap() - batched;
    }

    /// Records the prepared frame into `pass`: the batch maps view pixels
    /// onto a viewport of the view's size times the factor, so each view
    /// pixel covers a whole square of screen pixels and the batch's texel
    /// reads (`batch2d.wgsl`, pixel centres floored in view pixels) give
    /// the nearest-neighbour enlargement of the 1x picture.
    pub fn draw(&self, pass: &mut wgpu::RenderPass<'_>) {
        let (f, target) = self.screen;
        if f > 1 {
            // design: readable text and display scale (lead's request 2026-11-10): whole-number factor, nearest neighbour, from the top left corner
            let w = (self.size.0 * f).min(target.0).max(1);
            let h = (self.size.1 * f).min(target.1).max(1);
            pass.set_viewport(0.0, 0.0, w as f32, h as f32, 0.0, 1.0);
        }
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
        let start = self.lap();
        let synced = match self.map.as_mut() {
            Some(m) if m.serial == serial => m.renderer.sync(device, queue, &s.scene).is_ok(),
            _ => false,
        };
        if !synced {
            self.rebuilds += 1;
            let renderer = MapRenderer::new(device, queue, &s.scene, COLOR_FORMAT)
                .map_err(|e| e.to_string())?;
            self.map = Some(MapGpu {
                serial,
                renderer,
                size: (0, 0),
                targets: None,
                light: None,
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
        let clock = self.lap_clock;
        let lap = || clock.map_or(0.0, |c| c());
        let lit = lap();
        self.laps.sync = lit - start;
        let covers = s.shroud_covers();
        let kept = m
            .light
            .as_ref()
            .is_some_and(|l| l.origin == v.origin && l.size == v.size && l.covers == covers);
        if !kept {
            let alpha = s.light_alpha_of(&covers);
            m.light = Some(LightKept {
                covers,
                origin: v.origin,
                size: v.size,
                alpha,
            });
        }
        let alpha = m.light.as_ref().map(|l| l.alpha.as_slice());
        let drawn = lap();
        self.laps.light = drawn - lit;
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
            alpha,
        );
        self.laps.map = lap() - drawn;
        Ok(())
    }
}

struct Gfx {
    /// GPU frame times (the benchmark, where the device has timestamps).
    timer: Option<glasnost_render::timer::GpuTimer>,
    /// The backend and adapter, for the benchmark's summary.
    device_label: String,
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
    /// `timing`: the benchmark's frames, with or without vsync.
    async fn new(
        window: Arc<Window>,
        backend: BackendChoice,
        display: Option<Box<dyn wgpu::wgt::WgpuHasDisplayHandle>>,
        timing: Option<bool>,
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
        let (device, queue) = match timing {
            Some(_) => {
                gpu::request_device_with(&adapter, glasnost_render::timer::TIMER_FEATURES).await
            }
            None => gpu::request_device(&adapter).await,
        }
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
        config.present_mode = match timing {
            Some(false) => wgpu::PresentMode::AutoNoVsync,
            _ => wgpu::PresentMode::AutoVsync,
        };
        let mut draw = GpuDraw::new(&device, config.format);
        let timer = timing.and_then(|_| glasnost_render::timer::GpuTimer::new(&device, &queue));
        if timing.is_some() {
            draw.lap_clock = Some(bench::now);
        }
        let device_label = format!(
            "{} on {} ({})",
            gpu::backend_label(info.backend),
            info.name,
            info.driver
        );
        let mut gfx = Self {
            timer,
            device_label,
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

    /// Draws the client's frame; `timing` numbers a benchmark frame, whose
    /// times come back, with what came of the try (#99).
    fn render(
        &mut self,
        client: &mut Client,
        timing: Option<u64>,
    ) -> Result<(FrameEnd, Drawn), String> {
        let mut end = FrameEnd::default();
        let t0 = bench::now();
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t)
            | wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
            wgpu::CurrentSurfaceTexture::Timeout => return Ok((end, Drawn::Again)),
            wgpu::CurrentSurfaceTexture::Occluded => return Ok((end, Drawn::Hidden)),
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.config);
                return Ok((end, Drawn::Again));
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err("the surface texture failed validation".into());
            }
        };
        end.acquire = bench::now() - t0;
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("game frame"),
            });
        if let Some(t) = self.timer.as_mut().filter(|_| timing.is_some()) {
            t.begin(&mut encoder);
        }
        // The picture is drawn in view pixels and shown at the display
        // factor (`super::display`).
        let factor = client.display().factor.max(1) as u32;
        let view = (client.size.0.max(1) as u32, client.size.1.max(1) as u32);
        self.draw
            .set_screen(factor, (self.config.width, self.config.height));
        self.draw
            .prepare(&self.device, &self.queue, client, view, &mut encoder);
        let t1 = bench::now();
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
        if let Some(t) = self.timer.as_mut().filter(|_| timing.is_some()) {
            t.end(&mut encoder);
        }
        let commands = encoder.finish();
        let t2 = bench::now();
        self.queue.submit([commands]);
        if let (Some(t), Some(n)) = (self.timer.as_mut(), timing) {
            t.submitted(n);
        }
        self.window.pre_present_notify();
        self.queue.present(frame);
        self.frames_presented += 1;
        if timing.is_some() {
            end.encode = t2 - t1;
            end.submit = bench::now() - t2;
            end.laps = self.draw.laps;
            end.stats = self.draw.take_stats();
            if let Some(t) = self.timer.as_mut() {
                end.gpu = t.collect(&self.device);
            }
        }
        Ok((end, Drawn::Yes))
    }

    /// The benchmark's last GPU times, once the device is idle (native).
    fn finish_timer(&mut self, b: &mut Bench) {
        if let Some(t) = self.timer.as_mut() {
            #[cfg(not(target_arch = "wasm32"))]
            let _ = self.device.poll(wgpu::PollType::wait_indefinitely());
            let rest = t.collect(&self.device);
            b.gpu_times(&rest);
        }
    }
}

/// Mirrors the client's probe into the page when it changed (web only):
/// `data-glasnost-view` (`x,y`) and `data-glasnost-selected`, both empty
/// outside a game.
fn publish_probe(client: &Client, last: &mut Option<((i32, i32), usize)>) {
    let now = client.probe();
    if now == *last {
        return;
    }
    *last = now;
    #[cfg(target_arch = "wasm32")]
    if let Some(body) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.body())
    {
        let data = body.dataset();
        let (view, selected) = now.map_or((String::new(), String::new()), |((x, y), n)| {
            (format!("{x},{y}"), n.to_string())
        });
        let _ = data.set("glasnostView", &view);
        let _ = data.set("glasnostSelected", &selected);
    }
}

/// Mirrors the display into the page when it changed (web only; the
/// browser check of the display scale): `data-glasnost-display` holds
/// `<physical w>x<physical h> <factor> <view w>x<view h> <host scale>`.
fn publish_display(client: &Client, last: &mut String) {
    let d = client.display();
    let now = format!(
        "{}x{} {} {}x{} {}",
        d.physical.0, d.physical.1, d.factor, client.size.0, client.size.1, d.host_scale
    );
    if now == *last {
        return;
    }
    #[cfg(target_arch = "wasm32")]
    if let Some(body) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.body())
    {
        let _ = body.dataset().set("glasnostDisplay", &now);
    }
    *last = now;
}

/// Stores the benchmark's outputs: natively files in its folder, on the web
/// downloads (`save`, the web shell's) and the page's `data-glasnost-bench`
/// (`done` or `failed`) with the summary in `data-glasnost-bench-summary`
/// and the rows in `data-glasnost-bench-csv`.
fn finish_bench(b: &Bench, save: Option<super::report::SaveReport>) {
    let summary = b.summary();
    log::info!("benchmark summary:\n{summary}");
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = save;
        match b.save() {
            Ok(paths) => {
                for p in paths {
                    log::info!("benchmark: wrote {p}");
                }
            }
            Err(e) => log::error!("benchmark: {e}"),
        }
        println!("{summary}");
    }
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(save) = save {
            for (name, bytes) in b.outputs() {
                if let Err(e) = save(&name, &bytes) {
                    log::error!("benchmark: {e}");
                }
            }
        }
        if let Some(body) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.body())
        {
            let data = body.dataset();
            let _ = data.set("glasnostBenchSummary", &summary);
            let _ = data.set("glasnostBenchCsv", &b.csv());
            let state = if b.failure().is_some() {
                "failed"
            } else {
                "done"
            };
            let _ = data.set("glasnostBench", state);
        }
    }
}
