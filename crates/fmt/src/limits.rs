//! Size limits that keep decoders bounded on hostile input.
//!
//! These are engineering bounds of our own, far above anything a real file
//! needs, not properties of the formats.

/// Most pixels a single decoded image may have (sprite frame, tile, PCX).
pub const MAX_IMAGE_PIXELS: usize = 1 << 26;

/// Most bytes a single decompression call may produce.
pub const MAX_DECOMPRESSED: usize = 1 << 28;

/// Most voxels' worth of grid cells (x × y × z) in one voxel limb.
pub const MAX_VOXEL_CELLS: usize = 1 << 24;
