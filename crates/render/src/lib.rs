//! Renderer on wgpu.
//!
//! The core path stays inside the WebGL2 feature set: no compute, storage
//! buffers, indirect draws or non-zero `base_vertex`; large tables go in data
//! textures. On the web the instance selects WebGPU at runtime where the
//! browser offers it; any WebGPU-only extra must check
//! [`gpu::extras_available`] first (PLAN.md 4.1).
//!
//! **Milestone M0** provides device setup ([`gpu`]) and the palette-indexed
//! quad ([`palette_quad`]) that proves the path on both backends.
//! **M1** adds sprite atlases, house-colour remap rows and voxel baking for
//! the asset viewer; **M2** terrain chunks and lighting; **M3+** draw lists
//! built from simulation snapshots. The renderer only reads simulation state.

pub mod batch2d;
pub mod compose;
pub mod gpu;
pub mod light;
pub mod map_gpu;
#[cfg(not(target_arch = "wasm32"))]
pub mod offscreen;
pub mod overlay;
pub mod palette;
pub mod palette_quad;
pub mod projection;
pub mod rz;
pub mod scene;
pub mod shroud;
pub mod spots;
pub mod tilt;
pub mod timer;
pub mod voxel;

pub use wgpu;
