//! The map viewer (milestone M2): decode a map (S-0009), build its scene
//! (S-0101 to S-0105, S-0107, S-0108) and show it with scrolling, a
//! minimap, layer toggles and object info.

pub mod anims;
pub mod beams;
pub mod bolts;
pub mod build;
#[cfg(test)]
mod buildup_tests;
#[cfg(test)]
mod capture_tests;
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
#[cfg(test)]
mod named_palette_tests;
pub mod particles;
pub mod piece_pool;
#[cfg(test)]
mod piece_tests;
pub mod pip_scale;
pub mod png;
#[cfg(test)]
mod projectile_tests;
pub mod projectiles;
pub mod protection;
pub mod quirks;
pub mod radar;
#[cfg(test)]
mod radar_tests;
pub mod rocking;
#[cfg(test)]
mod rocking_tests;
pub mod shadowing;
#[cfg(test)]
mod shot_anim_tests;
pub mod sidebar;
#[cfg(test)]
mod sinking_tests;
pub mod slot_play;
pub mod slots;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod transient_tests;
#[cfg(test)]
mod turret_index_tests;
pub mod unload;
pub mod walk;
