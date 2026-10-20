//! The map viewer (milestone M2): decode a map (S-0009), build its scene
//! (S-0101 to S-0105, S-0107, S-0108) and show it with scrolling, a
//! minimap, layer toggles and object info.

pub mod anims;
pub mod beams;
pub mod build;
pub mod combat;
pub mod cover;
pub mod credits;
pub mod find;
pub mod harvesting;
pub mod layers;
pub mod live;
#[cfg(test)]
mod live_tests;
pub mod load;
pub mod particles;
pub mod png;
#[cfg(test)]
mod projectile_tests;
pub mod projectiles;
pub mod protection;
pub mod quirks;
pub mod radar;
pub mod sidebar;
pub mod slot_play;
pub mod slots;
#[cfg(test)]
mod tests;
pub mod unload;
