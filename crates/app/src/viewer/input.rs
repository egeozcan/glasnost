//! winit window events to egui input.
//!
//! `egui-winit` does not build for the web, so this small translation
//! serves both targets: pointer, wheel, keys, typed text, modifiers, focus,
//! size and scale. Clipboard and IME are not needed by the viewer.

use egui::{
    Event, Key, Modifiers, MouseWheelUnit, PointerButton, Pos2, RawInput, TouchPhase, Vec2,
};
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::keyboard::{Key as WinitKey, NamedKey};

/// Accumulates egui input between frames.
pub struct Input {
    raw: RawInput,
    pointer: Option<Pos2>,
    modifiers: Modifiers,
    pixels_per_point: f32,
    size: (u32, u32),
}

impl Input {
    /// Input for a window of `size` physical pixels at `scale` pixels per point.
    pub fn new(size: (u32, u32), scale: f32) -> Self {
        let mut input = Self {
            raw: RawInput::default(),
            pointer: None,
            modifiers: Modifiers::default(),
            pixels_per_point: scale,
            size,
        };
        input.update_screen();
        input
    }

    fn update_screen(&mut self) {
        let ppp = self.pixels_per_point.max(0.1);
        self.raw.screen_rect = Some(egui::Rect::from_min_size(
            Pos2::ZERO,
            Vec2::new(self.size.0 as f32 / ppp, self.size.1 as f32 / ppp),
        ));
        self.raw
            .viewports
            .entry(egui::ViewportId::ROOT)
            .or_default()
            .native_pixels_per_point = Some(ppp);
    }

    /// Sets the largest texture side the device supports.
    pub fn set_max_texture_side(&mut self, side: usize) {
        self.raw.max_texture_side = Some(side);
    }

    /// Feeds one window event. Returns whether egui may want to repaint.
    pub fn on_event(&mut self, event: &WindowEvent) -> bool {
        let ppp = self.pixels_per_point.max(0.1);
        let events = &mut self.raw.events;
        match event {
            WindowEvent::Resized(size) => {
                self.size = (size.width, size.height);
                self.update_screen();
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.pixels_per_point = *scale_factor as f32;
                self.update_screen();
            }
            WindowEvent::CursorMoved { position, .. } => {
                let pos = Pos2::new(position.x as f32 / ppp, position.y as f32 / ppp);
                self.pointer = Some(pos);
                events.push(Event::PointerMoved(pos));
            }
            WindowEvent::CursorLeft { .. } => {
                self.pointer = None;
                events.push(Event::PointerGone);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let button = match button {
                    MouseButton::Left => PointerButton::Primary,
                    MouseButton::Right => PointerButton::Secondary,
                    MouseButton::Middle => PointerButton::Middle,
                    MouseButton::Back => PointerButton::Extra1,
                    MouseButton::Forward => PointerButton::Extra2,
                    MouseButton::Other(_) => return false,
                };
                if let Some(pos) = self.pointer {
                    events.push(Event::PointerButton {
                        pos,
                        button,
                        pressed: *state == ElementState::Pressed,
                        modifiers: self.modifiers,
                    });
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let (unit, delta) = match delta {
                    MouseScrollDelta::LineDelta(x, y) => (MouseWheelUnit::Line, Vec2::new(*x, *y)),
                    MouseScrollDelta::PixelDelta(p) => (
                        MouseWheelUnit::Point,
                        Vec2::new(p.x as f32 / ppp, p.y as f32 / ppp),
                    ),
                };
                events.push(Event::MouseWheel {
                    unit,
                    delta,
                    phase: TouchPhase::Move,
                    modifiers: self.modifiers,
                });
            }
            WindowEvent::ModifiersChanged(m) => {
                let s = m.state();
                self.modifiers = Modifiers {
                    alt: s.alt_key(),
                    ctrl: s.control_key(),
                    shift: s.shift_key(),
                    mac_cmd: cfg!(target_os = "macos") && s.super_key(),
                    command: if cfg!(target_os = "macos") {
                        s.super_key()
                    } else {
                        s.control_key()
                    },
                };
                events.push(Event::ModifiersChanged(self.modifiers));
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let pressed = event.state == ElementState::Pressed;
                if let Some(key) = translate_key(&event.logical_key) {
                    events.push(Event::Key {
                        key,
                        physical_key: None,
                        pressed,
                        repeat: event.repeat,
                        modifiers: self.modifiers,
                    });
                }
                if pressed
                    && !self.modifiers.ctrl
                    && !self.modifiers.mac_cmd
                    && let Some(text) = &event.text
                {
                    let text: String = text.chars().filter(|c| !c.is_control()).collect();
                    if !text.is_empty() {
                        events.push(Event::Text(text));
                    }
                }
            }
            WindowEvent::Focused(focused) => {
                self.raw.focused = *focused;
                events.push(Event::WindowFocused(*focused));
            }
            _ => return false,
        }
        true
    }

    /// The input gathered since the last call, stamped with `time` seconds.
    pub fn take(&mut self, time: f64) -> RawInput {
        self.raw.time = Some(time);
        let mut next = RawInput {
            screen_rect: self.raw.screen_rect,
            viewports: self.raw.viewports.clone(),
            max_texture_side: self.raw.max_texture_side,
            focused: self.raw.focused,
            ..RawInput::default()
        };
        std::mem::swap(&mut next, &mut self.raw);
        next
    }
}

fn translate_key(key: &WinitKey) -> Option<Key> {
    match key {
        WinitKey::Named(named) => Some(match named {
            NamedKey::ArrowDown => Key::ArrowDown,
            NamedKey::ArrowLeft => Key::ArrowLeft,
            NamedKey::ArrowRight => Key::ArrowRight,
            NamedKey::ArrowUp => Key::ArrowUp,
            NamedKey::Escape => Key::Escape,
            NamedKey::Tab => Key::Tab,
            NamedKey::Backspace => Key::Backspace,
            NamedKey::Enter => Key::Enter,
            NamedKey::Space => Key::Space,
            NamedKey::Insert => Key::Insert,
            NamedKey::Delete => Key::Delete,
            NamedKey::Home => Key::Home,
            NamedKey::End => Key::End,
            NamedKey::PageUp => Key::PageUp,
            NamedKey::PageDown => Key::PageDown,
            _ => return None,
        }),
        WinitKey::Character(c) => Key::from_name(c.as_str()),
        _ => None,
    }
}

/// Seconds since an arbitrary start, for egui's clock.
pub fn now_seconds() -> f64 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::sync::OnceLock;
        static START: OnceLock<std::time::Instant> = OnceLock::new();
        START
            .get_or_init(std::time::Instant::now)
            .elapsed()
            .as_secs_f64()
    }
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .and_then(|w| w.performance())
            .map_or(0.0, |p| p.now() / 1000.0)
    }
}
