// Map scene drawing on the GPU: the same depth rules as `compose.rs`
// (S-0102 F1 to F3), inside WebGL2 limits (no storage buffers, integer
// texel fetches only). Each instance is one scene item.

struct View {
    // World pixel of view pixel (0, 0).
    origin: vec2<i32>,
    // View size in pixels.
    size: vec2<i32>,
    // Depth reference C (S-0102 F1).
    reference: i32,
    // Number of palette rows per column of the rows texture.
    rows_per_column: i32,
    pad0: i32,
    pad1: i32,
};

@group(0) @binding(0) var<uniform> view: View;
@group(0) @binding(1) var atlas: texture_2d_array<u32>;
@group(0) @binding(2) var rows: texture_2d<f32>;
// Light-alpha value of each view pixel (S-0104 claim 15, S-0110 claim 5).
@group(0) @binding(3) var light_alpha: texture_2d<u32>;
// The target's pixels drawn before a translucent item, under that item
// (S-0102 claim 32).
@group(0) @binding(4) var backdrop: texture_2d<f32>;

struct Instance {
    // Atlas x, y, layer, and width | height << 16.
    @location(0) image: vec4<u32>,
    // World x, world y, depth bias, tile reference row (world).
    @location(1) place: vec4<i32>,
    // Depth image atlas x, y, layer (0xFFFFFFFF: none), flags: bits 0 and 1
    // the depth kind (tile, flat, slope, upright), bit 2 set for a profile
    // drawing with a companion depth image, whose atlas slot x, y and layer
    // are then x, y and z.
    @location(2) depth: vec4<u32>,
    // Palette row (the table's first row when z > 0; a pixel particle's
    // colour when y bit 3 is set); y: bit 0 set for a tile diamond (no
    // transparent index), bits 1 and 2 the translucency code (S-0102 claim
    // 32), bit 3 a pixel particle (S-0616 claim 28), bits 16 to 31 the protection colour mask K
    // (S-0102 claim 34, F9); z the shade table's row count N for a drawing
    // that reads the light-alpha values, else 0; w its rho.
    @location(3) paint: vec4<u32>,
    // The companion depth image's world x, y, width and height.
    @location(4) companion: vec4<i32>,
};

struct Varyings {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) image: vec4<u32>,
    @location(1) @interpolate(flat) place: vec4<i32>,
    @location(2) @interpolate(flat) depth: vec4<u32>,
    @location(3) @interpolate(flat) paint: vec4<u32>,
    @location(4) @interpolate(flat) companion: vec4<i32>,
};

@vertex
fn vs_main(@builtin(vertex_index) v: u32, inst: Instance) -> Varyings {
    var corners = array<vec2<i32>, 6>(
        vec2<i32>(0, 0), vec2<i32>(1, 0), vec2<i32>(0, 1),
        vec2<i32>(0, 1), vec2<i32>(1, 0), vec2<i32>(1, 1),
    );
    let size = vec2<i32>(i32(inst.image.w & 0xFFFFu), i32(inst.image.w >> 16u));
    let p = inst.place.xy - view.origin + corners[v] * size;
    let ndc = vec2<f32>(
        f32(p.x) / f32(view.size.x) * 2.0 - 1.0,
        1.0 - f32(p.y) / f32(view.size.y) * 2.0,
    );
    var out: Varyings;
    out.position = vec4<f32>(ndc, 0.5, 1.0);
    out.image = inst.image;
    out.place = inst.place;
    out.depth = inst.depth;
    out.paint = inst.paint;
    out.companion = inst.companion;
    return out;
}

// d(r) of S-0102 F1 for view row r. In play C − w stays inside 0 … 65535.
fn row_depth(r: i32) -> i32 {
    return view.reference - (r + view.origin.y);
}

// Q(v) = trunc(v / 3) × 3 (S-0102 F2); v is positive in play.
fn quantise(v: i32) -> i32 {
    return (v / 3) * 3;
}

// Whether pixel (c, r) of a 60 × 29 box is one of a tile diamond's 900
// pixels (S-0010 Formulas, Diamond rows).
fn in_diamond(c: i32, r: i32) -> bool {
    if r < 0 || r > 28 {
        return false;
    }
    var f: i32;
    var n: i32;
    if r <= 14 {
        f = 28 - 2 * r;
        n = 4 * (r + 1);
    } else {
        f = 2 * (r - 14);
        n = 4 * (29 - r);
    }
    return c >= f && c < f + n;
}

struct Fragment {
    @location(0) color: vec4<f32>,
    @builtin(frag_depth) depth: f32,
};

@fragment
fn fs_main(in: Varyings) -> Fragment {
    let px = vec2<i32>(floor(in.position.xy));
    let top_left = in.place.xy - view.origin;
    let local = px - top_left;
    let index = textureLoad(atlas, vec2<i32>(in.image.xy) + local, i32(in.image.z), 0).r;
    if index == 0u && !((in.paint.y & 1u) == 1u && in_diamond(local.x, local.y)) {
        discard;
    }

    let flags = in.depth.w;
    let kind = flags & 3u;
    let height = i32(in.image.w >> 16u);
    var depth: i32;
    if (flags & 4u) != 0u {
        // S-0102 claim 31, F5: a pixel outside the companion's stored
        // rectangle is not drawn and writes no depth.
        let c = px - (in.companion.xy - view.origin);
        if c.x < 0 || c.y < 0 || c.x >= in.companion.z || c.y >= in.companion.w {
            discard;
        }
        var a = i32(textureLoad(atlas, vec2<i32>(in.depth.xy) + c, i32(in.depth.z), 0).r);
        if a >= 128 {
            a = a - 256;
        }
        // S-0102 F2: with a companion every row takes the anchor value (flat
        // d(r₀) + β, slope Q(d(r₀) + β), upright d(r₁) + β), minus a.
        let r0 = top_left.y;
        var anchor: i32;
        if kind == 1u {
            anchor = row_depth(r0) + in.place.z;
        } else if kind == 2u {
            anchor = quantise(row_depth(r0) + in.place.z);
        } else {
            anchor = row_depth(r0 + height - 1) + in.place.z;
        }
        depth = anchor - a;
    } else if kind == 0u {
        // Tile (S-0102 F3): e + d(t + 30).
        let t = in.place.w - view.origin.y;
        var e = 0;
        if in.depth.z != 0xFFFFFFFFu {
            e = i32(textureLoad(atlas, vec2<i32>(in.depth.xy) + local, i32(in.depth.z), 0).r);
        }
        depth = e + row_depth(t + 30);
    } else if kind == 1u {
        // Flat: d(r) + β.
        depth = row_depth(px.y) + in.place.z;
    } else if kind == 2u {
        // Slope: Q(d(r₀) + β) − ⌊2j / 3⌋, r₀ the first row of the
        // rectangle before clipping (S-0102 F2: clipping removes pixels only).
        let r0 = top_left.y;
        depth = quantise(row_depth(r0) + in.place.z) - (2 * (px.y - r0)) / 3;
    } else {
        // Upright: Q(d(r₁) + β) − ⌊(r₁ − r) / 3⌋, r₁ the last row of the
        // rectangle before clipping (S-0102 F2).
        let r1 = top_left.y + height - 1;
        depth = quantise(row_depth(r1) + in.place.z) - (r1 - px.y) / 3;
    }
    depth = clamp(depth, 0, 65535);
    if (in.paint.y & 8u) != 0u {
        // S-0616 claim 28: a pixel particle's own colour (x = R << 16 |
        // G << 8 | B), not drawn at light-alpha 0, each channel times a / 128
        // (truncated) below 127, packed to 5-6-5, opaque.
        let a = textureLoad(light_alpha, px, 0).r;
        if a == 0u {
            discard;
        }
        var c = vec3<u32>((in.paint.x >> 16u) & 255u, (in.paint.x >> 8u) & 255u, in.paint.x & 255u);
        if a < 127u {
            c = (c * a) / 128u;
        }
        var pix: Fragment;
        pix.color = vec4<f32>(expand(c.r >> 3u, 5u), expand(c.g >> 2u, 6u), expand(c.b >> 3u, 5u), 1.0);
        pix.depth = f32(depth) / 65536.0;
        return pix;
    }
    var row = i32(in.paint.x);
    if in.paint.z > 0u {
        // S-0104 claim 15: row = min(N - 1, floor(a * rho * (N - 1) / 32258)),
        // a the pixel's light-alpha value after the covers (S-0110 claim 5).
        let a = i32(textureLoad(light_alpha, px, 0).r);
        let span = i32(in.paint.z) - 1;
        row = row + min(span, (a * i32(in.paint.w) * span) / 32258);
    }
    let column = row / view.rows_per_column;
    let y = row - column * view.rows_per_column;
    var out: Fragment;
    out.color = textureLoad(rows, vec2<i32>(column * 256 + i32(index), y), 0);
    let code = (in.paint.y >> 1u) & 3u;
    let mask = in.paint.y >> 16u;
    if code == 0u && mask != 0u {
        // S-0102 claim 34 and F9: the opaque pixel becomes S | K, the packed
        // shaded pixel united with the mask.
        let s = packed(out.color);
        let u = ((s.r << 11u) | (s.g << 5u) | s.b) | mask;
        out.color = vec4<f32>(
            expand((u >> 11u) & 31u, 5u),
            expand((u >> 5u) & 63u, 6u),
            expand(u & 31u, 5u),
            1.0,
        );
    }
    if code != 0u {
        // S-0102 claim 32 and F8: blend the packed source channels with the
        // packed channels already drawn there, each channel by itself.
        let s = packed(out.color);
        let d = packed(textureLoad(backdrop, px, 0));
        out.color = vec4<f32>(
            expand(blend(s.r, d.r, code), 5u),
            expand(blend(s.g, d.g, code), 6u),
            expand(blend(s.b, d.b, code), 5u),
            1.0,
        );
    }
    out.depth = f32(depth) / 65536.0;
    return out;
}

// The 5-6-5 channels of a colour whose 8-bit channels widen them by
// repeating their top bits (S-0103 claim 9; compose.rs expand565).
fn packed(c: vec4<f32>) -> vec3<u32> {
    let b = vec3<u32>(round(c.rgb * 255.0));
    return vec3<u32>(b.r >> 3u, b.g >> 2u, b.b >> 3u);
}

// A channel of `bits` bits widened to 8 bits, as a unit float.
fn expand(c: u32, bits: u32) -> f32 {
    return f32((c << (8u - bits)) | (c >> (2u * bits - 8u))) / 255.0;
}

// S-0102 F8 for one channel: each division rounds down before the sum.
fn blend(s: u32, d: u32, code: u32) -> u32 {
    if code == 1u {
        return 3u * (s / 4u) + d / 4u;
    } else if code == 2u {
        return s / 2u + d / 2u;
    }
    return s / 4u + 3u * (d / 4u);
}

// A 5-6-5 colour as a target colour (channels widened by repeating their
// top bits, as `packed` reads them back).
fn colour565(c: u32) -> vec4<f32> {
    return vec4<f32>(expand((c >> 11u) & 31u, 5u), expand((c >> 5u) & 63u, 6u), expand(c & 31u, 5u), 1.0);
}

// The target pixel at `p` drawn before the spot batch, widened to 8-bit
// channels with zero low bits (S-0518 F7 and F8, S-0607 F5).
fn widened(p: vec2<i32>) -> vec3<u32> {
    let s = packed(textureLoad(backdrop, p, 0));
    return vec3<u32>(s.r << 3u, s.g << 2u, s.b << 3u);
}

// 8-bit channels reduced to 5-6-5 by dropping the low bits.
fn pack565(c: vec3<u32>) -> u32 {
    return ((c.r >> 3u) << 11u) | ((c.g >> 2u) << 5u) | (c.b >> 3u);
}

fn unpack_rgb(w: u32) -> vec3<u32> {
    return vec3<u32>(w & 255u, (w >> 8u) & 255u, (w >> 16u) & 255u);
}

// Single-pixel drawings (spots.rs): paint.x the kind (0 opaque, 1 additive,
// 2 blended, 3 beam flash, 4 magnetic wave), paint.y the colour (5-6-5 for
// opaque, else 8-bit channels red lowest), paint.z the blended weights
// T[w] | T[256 − w] << 16 or the wave's k, companion.xy the alpha or
// source offset, place.z the depth bias.
@fragment
fn fs_spot(in: Varyings) -> Fragment {
    let px = vec2<i32>(floor(in.position.xy));
    let kind = in.paint.x;
    var depth = row_depth(px.y) + in.place.z;
    if kind == 3u {
        // S-0514 F8: (d(r) + b) mod 65536.
        depth = ((depth % 65536) + 65536) % 65536;
    }
    depth = clamp(depth, 0, 65535);
    let a = textureLoad(light_alpha, px, 0).r;
    let d = widened(px);
    var c: vec3<u32>;
    if kind == 0u {
        // S-0518 claim 23: opaque, light-alpha does not apply.
        var out0: Fragment;
        out0.color = colour565(in.paint.y);
        out0.depth = f32(depth) / 65536.0;
        return out0;
    } else if kind == 1u {
        // S-0518 F7.
        let l = unpack_rgb(in.paint.y);
        var t = d + (l * a) / 128u;
        var k = 0u;
        var x = 0u;
        for (var i = 0; i < 3; i++) {
            if t[i] >= 255u {
                k += 1u;
                x += t[i] - 255u;
            }
        }
        var spill = 0u;
        if k == 1u {
            spill = x / 2u;
        } else if k == 2u {
            spill = x;
        }
        for (var i = 0; i < 3; i++) {
            if t[i] < 255u {
                t[i] += spill;
            }
        }
        c = min(t, vec3<u32>(255u));
    } else if kind == 2u {
        // S-0518 F8, with the pair's first pixel's light-alpha.
        let ap = clamp(px + in.companion.xy, vec2<i32>(0), view.size - vec2<i32>(1));
        let a2 = textureLoad(light_alpha, ap, 0).r;
        if a2 == 0u {
            discard;
        }
        let l = unpack_rgb(in.paint.y);
        let tw = in.paint.z & 0xFFFFu;
        let tr = in.paint.z >> 16u;
        c = d;
        for (var i = 0; i < 3; i++) {
            if l[i] > 0u {
                c[i] = min(((d[i] * tr + l[i] * tw) * a2) >> 15u, 255u);
            }
        }
    } else if kind == 3u {
        // S-0514 claim 25.
        if a == 0u {
            discard;
        }
        let p = unpack_rgb(in.paint.y);
        let l = vec3<u32>((p.r >> 3u) << 3u, (p.g >> 2u) << 2u, (p.b >> 3u) << 3u);
        if a == 127u {
            c = l;
        } else {
            c = min((l * a) / 128u, vec3<u32>(255u));
        }
    } else {
        // S-0607 F5: the source pixel as before the wave drew, boosted.
        let s = widened(px + in.companion.xy);
        let k = in.paint.z;
        c = vec3<u32>(min(s.r + ((k / 2u) * s.r) / 256u, 255u), s.g, min(s.b + (4u * k * s.b) / 256u, 255u));
    }
    var out: Fragment;
    out.color = colour565(pack565(c));
    out.depth = f32(depth) / 65536.0;
    return out;
}
