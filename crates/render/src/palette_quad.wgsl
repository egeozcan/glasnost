// Palette-indexed textured quad (M0 demo of the core renderer path).
//
// WebGL2 core path only: uniform buffer + two plain textures read with
// textureLoad. No samplers, storage buffers, compute or vertex buffers.

struct Params {
    // Half-extent of the quad in clip space.
    scale: vec2<f32>,
    // Row of the palette texture to read colours from.
    palette_row: u32,
    // Shade offset added within each 16-entry ramp (palette cycling).
    cycle: u32,
};

@group(0) @binding(0) var<uniform> params: Params;
// R8Uint: one palette index per texel.
@group(0) @binding(1) var indices: texture_2d<u32>;
// RGBA8: 256 columns (palette entries) × one row per palette.
@group(0) @binding(2) var palette: texture_2d<f32>;

struct VsOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex: u32) -> VsOut {
    // Two triangles covering the unit square: (0,0) (1,0) (0,1) / (0,1) (1,0) (1,1).
    let corner_x = f32((vertex == 1u) || (vertex == 4u) || (vertex == 5u));
    let corner_y = f32((vertex == 2u) || (vertex == 3u) || (vertex == 5u));
    let uv = vec2<f32>(corner_x, corner_y);
    var out: VsOut;
    // uv.y grows downwards (image row order); clip-space y grows upwards.
    out.position = vec4<f32>((uv.x * 2.0 - 1.0) * params.scale.x, (1.0 - uv.y * 2.0) * params.scale.y, 0.0, 1.0);
    out.uv = uv;
    return out;
}

// Must match `cycled_index` in palette_quad.rs.
fn cycled_index(index: u32, cycle: u32) -> u32 {
    return (index & 0xF0u) | ((index + cycle) & 0x0Fu);
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let dims = vec2<i32>(textureDimensions(indices));
    let texel = clamp(vec2<i32>(floor(in.uv * vec2<f32>(dims))), vec2<i32>(0, 0), dims - vec2<i32>(1, 1));
    let index = textureLoad(indices, texel, 0).r;
    let entry = cycled_index(index, params.cycle);
    return textureLoad(palette, vec2<i32>(i32(entry), i32(params.palette_row)), 0);
}
