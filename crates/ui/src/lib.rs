//! Glasnost's game interface: screens, widgets, layout and hit-testing,
//! independent of the GPU. Every screen describes itself as a
//! [`draw::DrawList`] that the renderer's 2D batch draws, and keeps the
//! rectangles of its targets for hit-testing.
//!
//! The look is our own (graphite panels, amber accents, our own bitmap
//! font). Development tools use egui and live elsewhere, never here. The
//! interface only reads simulation state and gives orders.

pub mod chat;
pub mod draw;
pub mod end;
pub mod font;
pub mod geom;
pub mod hud;
pub mod menu;
pub mod modes;
pub mod netmenu;
pub mod settings;
pub mod setup;
pub mod shell;
pub mod status;
pub mod watch;
pub mod widget;
