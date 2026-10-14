//! The game client (milestone M5b): main menu, skirmish setup, the game
//! with its sidebar, and the end screen, drawn with our own interface
//! (`glasnost_ui`) through the 2D batch (`glasnost_render::batch2d`).
//!
//! - [`play`]: the running game's model without drawing (selection, tools,
//!   sidebar clicks, orders through `glasnost_ai::game::Game`).

pub mod play;
