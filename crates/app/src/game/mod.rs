//! The game client (milestone M5b): main menu, skirmish setup, the game
//! with its sidebar, and the end screen, drawn with our own interface
//! (`glasnost_ui`) through the 2D batch (`glasnost_render::batch2d`).
//!
//! - [`play`]: the running game's model without drawing (selection, tools,
//!   sidebar clicks, orders through `glasnost_ai::game::Game`).
//! - [`session`]: a game on a loaded map: camera, battlefield, radar,
//!   gestures and the interface's draw list.
//! - [`paint`]: draw lists to 2D-batch quads.
//! - [`client`]: the screens, pacing, sound and input without a window.
//! - [`install`]: the install's rules, maps and samples for the menus.
//! - [`link`]: the connection to a relay (M9 N3), native or from the web shell.
//! - [`pump`]: a network game's updates while its window draws no frames.
//! - [`report`]: where a desync report goes (M9 N4), a file or a download.
//! - [`name_tip`]: the name tip over the battlefield (S-0111 claims 20 to 26)
//!   and the timing, layout and look the sidebar's tips share with it.
//! - [`sidebar_tip`]: the sidebar's tips, their places and texts (S-0411
//!   claims 20 to 22).
//! - [`status_font`]: the game bitmap font for status text.
//! - [`strings`]: display texts from the string table (S-0004 claim 17).
//! - [`captions`]: captions for announcements (M8 step C3).
//! - [`bot`]: a person played by our AI over the relay (M9 N6, the soak).
//! - [`bench`]: the performance benchmark (M8 step C4).
//! - [`touch`]: touch gestures and touchpad swipes (our own addition).

pub mod band;
pub mod bench;
pub mod bot;
pub mod captions;
pub mod capture;
pub mod client;
pub mod cursor;
pub mod direct;
pub mod install;
pub mod link;
pub mod music_share;
pub mod name_tip;
pub mod paint;
pub mod play;
pub mod pump;
pub mod report;
pub mod scroll;
pub mod session;
pub mod sidebar_art;
pub mod sidebar_tip;
pub mod status_font;
pub mod strings;
pub mod touch;
pub mod window;

pub use glasnost_net::desync::Perturb;
pub use window::{DrawLaps, DrawStats, GameOptions, GpuDraw, run_game};
