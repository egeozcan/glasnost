// The 2D interface batch: instanced quads in screen pixels.
//
// WebGL2 core path only: one uniform block, plain textures read with
// textureLoad, per-instance integer vertex attributes, vertices from the
// vertex index. No samplers, storage buffers, compute or indirect draws.
//
// Modes (must match batch2d.rs):
//   0 solid:  the instance colour
//   1 sprite: atlas index through a palette row; index 0 is transparent
//   2 glyph:  atlas index 0 is transparent, any other index the colour
//   3 image:  an RGBA image texel

struct Screen {
    size: vec2<f32>,
    pad: vec2<f32>,
};

@group(0) @binding(0) var<uniform> screen: Screen;
// R8Uint: the atlas of palette indices (sprites and glyphs).
@group(0) @binding(1) var atlas: texture_2d<u32>;
// RGBA8: 256 columns × one row per palette.
@group(0) @binding(2) var palettes: texture_2d<f32>;
// RGBA8: the image of this run (a 1 × 1 dummy when none).
@group(1) @binding(0) var image: texture_2d<f32>;

struct VsOut {
    @builtin(position) position: vec4<f32>,
    @location(0) local: vec2<f32>,
    @location(1) @interpolate(flat) src: vec4<i32>,
    @location(2) @interpolate(flat) dsize: vec2<i32>,
    @location(3) @interpolate(flat) rgba: u32,
    @location(4) @interpolate(flat) mode_row: u32,
};

@vertex
fn vs_main(
    @builtin(vertex_index) vertex: u32,
    @location(0) dst: vec4<i32>,
    @location(1) src: vec4<i32>,
    @location(2) rgba: u32,
    @location(3) mode_row: u32,
) -> VsOut {
    let cx = f32((vertex == 1u) || (vertex == 4u) || (vertex == 5u));
    let cy = f32((vertex == 2u) || (vertex == 3u) || (vertex == 5u));
    let corner = vec2<f32>(cx, cy);
    let size = vec2<f32>(dst.zw);
    let px = vec2<f32>(dst.xy) + corner * size;
    var out: VsOut;
    out.position = vec4<f32>(px.x / screen.size.x * 2.0 - 1.0, 1.0 - px.y / screen.size.y * 2.0, 0.0, 1.0);
    out.local = corner * size;
    out.src = src;
    out.dsize = dst.zw;
    out.rgba = rgba;
    out.mode_row = mode_row;
    return out;
}

fn unpack(c: u32) -> vec4<f32> {
    return vec4<f32>(f32(c & 255u), f32((c >> 8u) & 255u), f32((c >> 16u) & 255u), f32(c >> 24u)) / 255.0;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let mode = in.mode_row & 255u;
    let row = i32(in.mode_row >> 8u);
    if (mode == 0u) {
        return unpack(in.rgba);
    }
    // The texel of this pixel: pixel centres scaled from the destination to
    // the source rectangle (exact for 1:1 and integer ratios).
    let l = floor(in.local);
    let t = vec2<i32>(floor((l + vec2<f32>(0.5, 0.5)) * vec2<f32>(in.src.zw) / vec2<f32>(in.dsize)));
    let texel = in.src.xy + clamp(t, vec2<i32>(0, 0), in.src.zw - vec2<i32>(1, 1));
    if (mode == 3u) {
        return textureLoad(image, texel, 0);
    }
    let index = textureLoad(atlas, texel, 0).r;
    if (index == 0u) {
        discard;
    }
    if (mode == 2u) {
        return unpack(in.rgba);
    }
    return textureLoad(palettes, vec2<i32>(i32(index), row), 0);
}
