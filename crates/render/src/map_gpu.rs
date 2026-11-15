//! The map scene on the GPU (WebGL2 core path, PLAN.md 4.1).
//!
//! Scene images go into an `R8Uint` array texture atlas, palette rows into an
//! `Rgba8Unorm` texture, and each visible item is one instance. Depth uses a
//! `Depth32Float` buffer holding depth / 65536, which is exact for every
//! depth value 0 … 65535 of S-0102; the depth tests are the hardware tests
//! (≤ for tiles, < for everything else, claims 9 and 10), and writers are
//! the pipelines that store depth. Items are drawn in scene order, batched
//! into runs that share a pipeline, so the result matches
//! [`Scene::compose`] pixel for pixel. The one exception is shadows: the GPU
//! halves the expanded 8-bit channels by blending, rounded down (a shadow's
//! source alpha `SHADOW_FACTOR` in the shader, the opaque pipelines blending
//! source + target × source alpha so bodies and their shadows share a
//! pipeline and a draw), so a shadowed pixel packs to the CPU's 5-6-5
//! halving and a later translucent drawing reads the same backdrop, but its
//! low display bits can differ from the CPU's re-expanded channels (S-0102
//! claim 26, F7: an engine that keeps 8-bit channels packs to the same
//! hi-colour pixel; the display path here keeps the expanded channels).

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::compose::{ComposeQuirks, DepthTest, Paint, Profile, expand565};
use crate::scene::{Filter, ItemDepth, Scene, View};
use crate::spots::{SpotGroup, SpotKind};

/// WGSL source.
pub const SHADER_SOURCE: &str = include_str!("map_gpu.wgsl");

/// Depth format of the map pass.
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

/// Colour format of the map pass target.
pub const COLOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

const NO_IMAGE: u32 = u32::MAX;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
struct ViewUniform {
    origin: [i32; 2],
    size: [i32; 2],
    reference: i32,
    rows_per_column: i32,
    pad: [i32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
struct Instance {
    image: [u32; 4],
    place: [i32; 4],
    depth: [u32; 4],
    paint: [u32; 4],
    /// The companion depth image's world x and y, width and height.
    companion: [i32; 4],
}

/// Where an image sits in the atlas.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Slot {
    x: u32,
    y: u32,
    layer: u32,
}

impl Slot {
    /// No place in the atlas: the image is wider or taller than a layer and
    /// its items are not drawn (design: render (#108)).
    const ABSENT: Slot = Slot {
        x: 0,
        y: 0,
        layer: NO_IMAGE,
    };

    /// Whether the image has a place in the atlas.
    fn present(self) -> bool {
        self.layer != NO_IMAGE
    }
}

/// The height of the shelves that take an image `h` rows tall, in layers
/// of `side` rows: 8, 12, 16, 24, 32, 48 and so on, so a shelf is at most
/// half again as tall as an image it takes (design: render (#105)).
fn shelf_class(h: u32, side: u32) -> u32 {
    let mut c = 8u32;
    loop {
        if h <= c {
            return c.min(side);
        }
        if h <= c + c / 2 {
            return (c + c / 2).min(side);
        }
        c *= 2;
    }
}

/// Puts span `(x, w)` back into the sorted, disjoint `spans`, merged with
/// its neighbours.
fn insert_span(spans: &mut Vec<(u32, u32)>, (x, w): (u32, u32)) {
    let i = spans.partition_point(|&(sx, _)| sx < x);
    spans.insert(i, (x, w));
    if i + 1 < spans.len() && spans[i].0 + spans[i].1 == spans[i + 1].0 {
        spans[i].1 += spans[i + 1].1;
        spans.remove(i + 1);
    }
    if i > 0 && spans[i - 1].0 + spans[i - 1].1 == spans[i].0 {
        spans[i - 1].1 += spans[i].1;
        spans.remove(i);
    }
}

/// One shelf: its height class and its free column spans `(x, w)`, sorted.
#[derive(Clone, Debug)]
struct Shelf {
    height: u32,
    free: Vec<(u32, u32)>,
}

/// The atlas's allocator over layers of `side × side` (design: render
/// (#105)): shelves of height classes, first fit over the layers, each
/// shelf with free spans that are merged when an image leaves, and an
/// emptied shelf gives its rows back to its layer.
#[derive(Clone, Debug)]
struct Shelves {
    side: u32,
    /// The free row spans `(y, h)` of each layer, sorted.
    gaps: Vec<Vec<(u32, u32)>>,
    /// The shelves by layer and top row.
    shelves: std::collections::BTreeMap<(u32, u32), Shelf>,
    /// The shelves with a free span, by height class.
    open: std::collections::BTreeMap<u32, std::collections::BTreeSet<(u32, u32)>>,
}

impl Shelves {
    fn new(side: u32, layers: u32) -> Self {
        let mut s = Self {
            side,
            gaps: Vec::new(),
            shelves: Default::default(),
            open: Default::default(),
        };
        s.set_layers(layers);
        s
    }

    /// Places into `layers` layers: more, or fewer when those left out hold
    /// no shelf.
    fn set_layers(&mut self, layers: u32) {
        let keep = layers.max(self.used_layers()) as usize;
        self.gaps.truncate(keep);
        while self.gaps.len() < keep {
            self.gaps.push(vec![(0, self.side)]);
        }
    }

    /// One past the last layer holding a shelf.
    fn used_layers(&self) -> u32 {
        self.shelves.keys().next_back().map_or(0, |&(l, _)| l + 1)
    }

    /// Atlas pixels in shelves.
    fn committed(&self) -> u64 {
        self.shelves
            .values()
            .map(|s| u64::from(s.height) * u64::from(self.side))
            .sum()
    }

    /// A place for a `w` × `h` image, `None` when there is none (or the
    /// image is empty or larger than a layer).
    fn place(&mut self, w: u32, h: u32) -> Option<Slot> {
        let side = self.side;
        if w == 0 || h == 0 || too_large(w, h, side) {
            return None;
        }
        let class = shelf_class(h, side);
        let mut found = None;
        if let Some(set) = self.open.get(&class) {
            for &at in set {
                let shelf = &self.shelves[&at];
                if let Some(i) = shelf.free.iter().position(|&(_, fw)| fw >= w) {
                    found = Some((at, i));
                    break;
                }
            }
        }
        if let Some((at, i)) = found {
            let shelf = self.shelves.get_mut(&at).expect("an open shelf");
            let (x, fw) = shelf.free[i];
            if fw == w {
                shelf.free.remove(i);
            } else {
                shelf.free[i] = (x + w, fw - w);
            }
            if shelf.free.is_empty()
                && let Some(set) = self.open.get_mut(&class)
            {
                set.remove(&at);
            }
            return Some(Slot {
                x,
                y: at.1,
                layer: at.0,
            });
        }
        let (layer, i) = self.gaps.iter().enumerate().find_map(|(layer, gaps)| {
            gaps.iter()
                .position(|&(_, gh)| gh >= class)
                .map(|i| (layer, i))
        })?;
        let gaps = &mut self.gaps[layer];
        let (y, gh) = gaps[i];
        if gh == class {
            gaps.remove(i);
        } else {
            gaps[i] = (y + class, gh - class);
        }
        let at = (layer as u32, y);
        let free = if w < side {
            vec![(w, side - w)]
        } else {
            Vec::new()
        };
        if !free.is_empty() {
            self.open.entry(class).or_default().insert(at);
        }
        self.shelves.insert(
            at,
            Shelf {
                height: class,
                free,
            },
        );
        Some(Slot {
            x: 0,
            y,
            layer: layer as u32,
        })
    }

    /// Gives back the place of a `w` wide image at `slot`.
    fn free(&mut self, slot: Slot, w: u32) {
        let at = (slot.layer, slot.y);
        let side = self.side;
        let Some(shelf) = self.shelves.get_mut(&at) else {
            return;
        };
        insert_span(&mut shelf.free, (slot.x, w));
        let class = shelf.height;
        if shelf.free == [(0, side)] {
            self.shelves.remove(&at);
            if let Some(set) = self.open.get_mut(&class) {
                set.remove(&at);
            }
            if let Some(gaps) = self.gaps.get_mut(slot.layer as usize) {
                insert_span(gaps, (slot.y, class));
            }
        } else {
            self.open.entry(class).or_default().insert(at);
        }
    }
}

/// Packs rectangles into layers of `side × side`, tallest first, into
/// shelves of at most `max_layers` layers. Returns the slots and the
/// allocator, which places later rectangles around them. An empty
/// rectangle, or one wider or taller than a layer, gets [`Slot::ABSENT`]
/// (design: render (#108)); so does one that finds no place.
fn pack(sizes: &[(u32, u32)], side: u32, max_layers: u32) -> (Vec<Slot>, Shelves) {
    let mut order: Vec<usize> = (0..sizes.len()).collect();
    order.sort_by_key(|&i| std::cmp::Reverse(sizes[i].1));
    let mut slots = vec![Slot::ABSENT; sizes.len()];
    let mut shelves = Shelves::new(side, max_layers);
    for i in order {
        let (w, h) = sizes[i];
        // design: render (#108): an image larger than a layer is left out, not the whole upload
        slots[i] = shelves.place(w, h).unwrap_or(Slot::ABSENT);
    }
    (slots, shelves)
}

/// Whether a `w` × `h` image is wider or taller than a layer of `side`;
/// such an image gets [`Slot::ABSENT`] (design: render (#108)).
fn too_large(w: u32, h: u32, side: u32) -> bool {
    w > side || h > side
}

/// The layers an atlas of `layers` grows to, at most `max`: half again,
/// at least two more (design: render (#105)).
fn grown_layers(layers: u32, max: u32) -> u32 {
    layers.saturating_add((layers / 2).max(2)).min(max)
}

/// Warns about the images of `placed` (size and slot) that got no slot in
/// an atlas of `side`: their items are not drawn on the GPU.
fn warn_absent(placed: impl IntoIterator<Item = ((u32, u32), Slot)>, side: u32) {
    for ((w, h), slot) in placed {
        if !slot.present() {
            log::warn!(
                "a {w} × {h} image exceeds the GPU atlas of {side} × {side}: its items are not drawn"
            );
        }
    }
}

/// Atlas layers kept free at upload for images added later
/// ([`MapRenderer::sync`]); our own choice.
const SPARE_LAYERS: u32 = 1;

/// The side of an atlas layer when the device allows it; our own choice.
const ATLAS_SIDE: u32 = 4096;

/// The largest side of the palette-row texture on `device`: the atlas's
/// default side, whatever side a test gives the atlas.
fn row_side(device: &wgpu::Device) -> u32 {
    device.limits().max_texture_dimension_2d.min(ATLAS_SIDE)
}

/// Pipelines by role.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Role {
    Tile,
    /// Writer, reader and a drawing that ignores depth (neither reads nor
    /// writes it, S-0102 claim 25): opaque drawings and shadows alike, a
    /// shadow darkening what is under it by the blend (design: render
    /// (#114)); a shadow that stores its depth where it darkens (S-0102
    /// claim 27) is a writer.
    Writer,
    Reader,
    Ignore,
    /// Translucent drawings (S-0102 claim 32, F8) with the depth tests of
    /// a tile, a writer, a reader and a drawing that ignores depth: the
    /// shader blends with a copy of the target.
    BlendTile,
    BlendWriter,
    BlendReader,
    BlendIgnore,
    /// Single-pixel drawings ([`crate::spots`]): depth readers that compute
    /// their pixel from the copy of the target.
    Spot,
}

/// Every role, in pipeline order.
const ROLES: [Role; 9] = [
    Role::Tile,
    Role::Writer,
    Role::Reader,
    Role::Ignore,
    Role::BlendTile,
    Role::BlendWriter,
    Role::BlendReader,
    Role::BlendIgnore,
    Role::Spot,
];

impl Role {
    /// The role's pipeline.
    fn index(self) -> usize {
        ROLES.iter().position(|&r| r == self).unwrap_or(0)
    }

    /// Whether the role blends with the target's earlier pixels in the
    /// shader, so that it needs a copy of them.
    fn translucent(self) -> bool {
        matches!(
            self,
            Role::BlendTile
                | Role::BlendWriter
                | Role::BlendReader
                | Role::BlendIgnore
                | Role::Spot
        )
    }
}

/// What the map pass drew and uploaded: the counters of the performance
/// benchmark (M8 step C4). [`MapRenderer::take_stats`] returns them and
/// starts counting again.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MapStats {
    /// Scene items (stack copies counted) looked at by the last render: the
    /// cell pass's items near the view and every item of the object pass
    /// (design: render (#113)).
    pub items: usize,
    /// Instances drawn (each a six-vertex quad).
    pub instances: usize,
    /// Draw calls.
    pub draws: usize,
    /// Render passes begun.
    pub passes: usize,
    /// Copies of the target into the backdrop (one per translucent run).
    pub copies: usize,
    /// Pixels those copies moved.
    pub copied_px: u64,
    /// Texture and buffer writes.
    pub uploads: usize,
    /// Bytes those writes moved.
    pub upload_bytes: u64,
}

/// How full the atlas and the palette-row texture are: the benchmark's
/// gauges (#105, #107), read with [`MapRenderer::atlas_use`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AtlasUse {
    /// The side of an atlas layer.
    pub side: u32,
    /// Atlas layers.
    pub layers: u32,
    /// Atlas pixels given to images so far: whole shelves, from the top of
    /// each layer down to the lowest shelf opened.
    pub committed_px: u64,
    /// Palette rows uploaded.
    pub rows: usize,
}

/// GPU resources for one scene.
pub struct MapRenderer {
    pipelines: [wgpu::RenderPipeline; ROLES.len()],
    bind_group: wgpu::BindGroup,
    layout: wgpu::BindGroupLayout,
    view_buffer: wgpu::Buffer,
    instances: wgpu::Buffer,
    instance_capacity: usize,
    slots: Vec<Slot>,
    /// The size of each image of [`Scene::images`] as it was placed: a
    /// rewrite at another size moves it (design: render (#105)).
    sizes: Vec<(u32, u32)>,
    /// Atlas slots of [`Scene::signed`].
    signed_slots: Vec<Slot>,
    rows_per_column: u32,
    atlas: wgpu::Texture,
    atlas_layers: u32,
    /// The device's limit on the atlas's layers.
    max_layers: u32,
    shelves: Shelves,
    row_count: usize,
    /// The scene's [`Scene::rewrites`] when last uploaded or synced.
    rewrites_seen: u64,
    /// The scene's [`Scene::row_rewrites`] when last uploaded or synced
    /// (design: render (#107)).
    row_rewrites_seen: u64,
    target_format: wgpu::TextureFormat,
    rows: wgpu::Texture,
    /// The light-alpha texture and its size.
    alpha: (wgpu::Texture, (u32, u32)),
    /// Whether the light-alpha texture holds 127 everywhere.
    alpha_default: bool,
    /// The values the light-alpha texture holds when not `alpha_default`:
    /// a frame whose values are the same is not uploaded again (M8 step
    /// C4).
    alpha_held: Vec<u8>,
    /// The copy of the target's pixels that a translucent drawing blends
    /// with (S-0102 claim 32), of the light-alpha texture's size.
    backdrop: wgpu::Texture,
    /// The counters since the last [`MapRenderer::take_stats`].
    stats: MapStats,
    /// The last frame's instance list, as the instance buffer holds it, and
    /// their view rectangles, kept for their room.
    list: Vec<Instance>,
    rects: Vec<[u32; 4]>,
    /// The list before the last, kept for its room.
    spare: Vec<Instance>,
    /// Whether the instance buffer holds `list` (false for a new buffer).
    list_held: bool,
    /// The cell pass's items by place, kept while the cell pass stays
    /// (design: render (#113)).
    cull: Option<CellCull>,
    /// Images rewritten at another size since the last render.
    resized: Vec<u32>,
    /// A frame's cell-pass items near the view, kept for its room.
    near: Vec<u32>,
    /// The view's pixels marked for laying out spot groups (design: render
    /// (#115, #116)).
    spot_stamps: SpotStamps,
}

/// The side of a [`CellCull`] bucket in world pixels (design: render
/// (#113): about a cell's width, so that the buckets a view meets hold few
/// items outside it; a 1920 × 1080 view meets about 560).
const CULL_BUCKET: i32 = 64; // design: render (#113)

/// The most buckets an item is filed in; a larger item is looked at in
/// every frame (design: render (#113)).
const CULL_SPAN: i64 = 256; // design: render (#113)

/// The items of a cell pass filed by the world buckets their rectangles
/// meet (design: render (#113)), so that a frame looks only at those near
/// its view. It holds while the scene's [`Scene::cell_pass_rev`] and
/// [`Scene::cell_pass_len`] are those it was made for and none of the
/// images its items draw changed size.
struct CellCull {
    rev: u64,
    len: usize,
    /// Whether the cell pass draws image `i`.
    images: Vec<bool>,
    /// The bucket of the grid's first column and row.
    origin: (i32, i32),
    /// The grid's columns and rows.
    dims: (i32, i32),
    /// Each bucket's items, in scene order, row by row.
    buckets: Vec<Vec<u32>>,
    /// Items looked at in every frame: a stack (its copies depend on the
    /// view) or an item larger than [`CULL_SPAN`] buckets.
    always: Vec<u32>,
}

impl CellCull {
    /// Files the cell pass of `scene`.
    fn new(scene: &Scene) -> Self {
        let len = scene.cell_pass_len.min(scene.items.len());
        let mut images = vec![false; scene.images.len()];
        let mut always = Vec::new();
        let mut filed: Vec<(u32, (i32, i32, i32, i32))> = Vec::new();
        let (mut lo, mut hi) = ((i32::MAX, i32::MAX), (i32::MIN, i32::MIN));
        for (n, item) in scene.items[..len].iter().enumerate() {
            let Some(seen) = images.get_mut(item.image as usize) else {
                always.push(n as u32);
                continue;
            };
            *seen = true;
            let (l, t, r, b) = scene.bounds(item);
            if r <= l || b <= t {
                // Draws no pixel anywhere; looked at so that its count stays.
                always.push(n as u32);
                continue;
            }
            let span = (
                l.div_euclid(CULL_BUCKET),
                t.div_euclid(CULL_BUCKET),
                (r - 1).div_euclid(CULL_BUCKET),
                (b - 1).div_euclid(CULL_BUCKET),
            );
            let count = (i64::from(span.2) - i64::from(span.0) + 1)
                * (i64::from(span.3) - i64::from(span.1) + 1);
            if item.stack.is_some() || count > CULL_SPAN {
                always.push(n as u32);
                continue;
            }
            lo = (lo.0.min(span.0), lo.1.min(span.1));
            hi = (hi.0.max(span.2), hi.1.max(span.3));
            filed.push((n as u32, span));
        }
        let dims = if filed.is_empty() {
            (0, 0)
        } else {
            (hi.0 - lo.0 + 1, hi.1 - lo.1 + 1)
        };
        let mut buckets = vec![Vec::new(); (dims.0 as usize) * (dims.1 as usize)];
        for (n, (x0, y0, x1, y1)) in filed {
            for by in y0..=y1 {
                for bx in x0..=x1 {
                    let at = (by - lo.1) as usize * dims.0 as usize + (bx - lo.0) as usize;
                    buckets[at].push(n);
                }
            }
        }
        Self {
            rev: scene.cell_pass_rev,
            len: scene.cell_pass_len,
            images,
            origin: lo,
            dims,
            buckets,
            always,
        }
    }

    /// Whether the cull holds for `scene`, whose images `resized` changed
    /// size since it was last asked.
    fn holds(&self, scene: &Scene, resized: &[u32]) -> bool {
        self.rev == scene.cell_pass_rev
            && self.len == scene.cell_pass_len
            && scene.items.len() >= self.len
            && !resized
                .iter()
                .any(|&i| self.images.get(i as usize).copied().unwrap_or(false))
    }

    /// Puts into `out`, in scene order, the cell-pass items whose rectangle
    /// may meet the view of `origin` and `size`: every one that does.
    fn near(&self, origin: (i32, i32), size: (i32, i32), out: &mut Vec<u32>) {
        out.clear();
        out.extend_from_slice(&self.always);
        if size.0 > 0 && size.1 > 0 && self.dims.0 > 0 {
            let x0 = (origin.0.div_euclid(CULL_BUCKET) - self.origin.0).max(0);
            let y0 = (origin.1.div_euclid(CULL_BUCKET) - self.origin.1).max(0);
            let x1 = ((origin.0 + size.0 - 1).div_euclid(CULL_BUCKET) - self.origin.0)
                .min(self.dims.0 - 1);
            let y1 = ((origin.1 + size.1 - 1).div_euclid(CULL_BUCKET) - self.origin.1)
                .min(self.dims.1 - 1);
            for by in y0..=y1 {
                for bx in x0..=x1 {
                    out.extend_from_slice(&self.buckets[(by * self.dims.0 + bx) as usize]);
                }
            }
        }
        out.sort_unstable();
        out.dedup();
    }
}

/// Checks that `near` holds every cell-pass item of `scene` whose rectangle
/// meets `view` (debug builds: a cell pass changed without a new
/// [`Scene::cell_pass_rev`] shows here).
#[cfg(debug_assertions)]
fn check_near(scene: &Scene, view: &View, near: &[u32]) {
    let (ox, oy) = view.origin;
    let (w, h) = (view.size.0 as i32, view.size.1 as i32);
    for (n, item) in scene.items[..scene.cell_pass_len.min(scene.items.len())]
        .iter()
        .enumerate()
    {
        if item.stack.is_some() || item.image as usize >= scene.images.len() {
            continue;
        }
        let (l, t, r, b) = scene.bounds(item);
        let meets = r > ox && b > oy && l < ox + w && t < oy + h;
        assert!(
            !meets || near.binary_search(&(n as u32)).is_ok(),
            "cell-pass item {n} meets the view but the kept cull left it out: the cell pass \
             changed without a new Scene::cell_pass_rev (design: render (#113))"
        );
    }
}

/// Why the scene could not be uploaded.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UploadError(pub String);

impl std::fmt::Display for UploadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for UploadError {}

impl MapRenderer {
    /// Uploads `scene` for drawing into targets of `target_format`
    /// ([`COLOR_FORMAT`] or its sRGB form; with an sRGB target the palette
    /// rows are stored sRGB too, so decode on fetch and encode on store
    /// cancel out and the bytes stay exact).
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &Scene,
        target_format: wgpu::TextureFormat,
    ) -> Result<Self, UploadError> {
        Self::with_side(device, queue, scene, target_format, ATLAS_SIDE)
    }

    /// [`Self::new`] with atlas layers of at most `side` × `side` (the
    /// device's limit lowers it further): tests fill a small atlas.
    pub fn with_side(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &Scene,
        target_format: wgpu::TextureFormat,
        side: u32,
    ) -> Result<Self, UploadError> {
        let limits = device.limits();
        let side = limits.max_texture_dimension_2d.min(side).max(1);
        // Indexed images, then the signed companion depth images (their bytes
        // are the two's complement values). An empty image (a released one,
        // design: render (#104)) takes no place.
        let sizes: Vec<(u32, u32)> = scene
            .images
            .iter()
            .map(|i| (i.width(), i.height()))
            .chain(scene.signed.iter().map(|i| (i.width, i.height)))
            .collect();
        let max_layers = limits.max_texture_array_layers.max(1);
        let (mut slots, mut shelves) = pack(&sizes, side, max_layers);
        warn_absent(sizes.iter().copied().zip(slots.iter().copied()), side);
        let unplaced = sizes
            .iter()
            .zip(&slots)
            .filter(|&(&(w, h), s)| w > 0 && h > 0 && !too_large(w, h, side) && !s.present())
            .count();
        if unplaced > 0 {
            return Err(UploadError(format!(
                "{unplaced} images find no place in {max_layers} atlas layers"
            )));
        }
        let signed_slots = slots.split_off(scene.images.len());
        let sizes: Vec<(u32, u32)> = sizes[..scene.images.len()].to_vec();
        // On the GL backends a one-layer texture is a plain 2D texture and
        // cannot be viewed as an array, so the atlas always has two layers.
        // Spare layers take images added later: half again, at least
        // SPARE_LAYERS (design: render (#105)).
        let used = shelves.used_layers();
        let layers = used
            .saturating_add((used / 2).max(SPARE_LAYERS))
            .min(max_layers)
            .max(2);
        shelves.set_layers(layers);
        let atlas_texture = atlas_texture(device, side, layers);
        // design: render (#105): each image written on its own, with no staging copy of the whole atlas
        let images = scene
            .images
            .iter()
            .map(|i| (i.width(), i.height(), i.pixels()))
            .chain(
                scene
                    .signed
                    .iter()
                    .map(|i| (i.width, i.height, bytemuck::cast_slice::<i8, u8>(&i.values))),
            );
        for ((w, h, pixels), &slot) in images.zip(slots.iter().chain(&signed_slots)) {
            write_image(queue, &atlas_texture, slot, w, h, pixels);
        }
        let (rows_texture, rows_per_column) =
            rows_texture(device, queue, scene, row_side(device), target_format)?;
        let view_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("map view"),
            size: std::mem::size_of::<ViewUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("map"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Uint,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Uint,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });
        let alpha = alpha_texture(device, queue, (1, 1));
        let backdrop = backdrop_texture(device, (1, 1));
        let bind_group = bind_group(
            device,
            &layout,
            &view_buffer,
            &atlas_texture,
            &rows_texture,
            &alpha,
            &backdrop,
        );
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("map"),
            source: wgpu::ShaderSource::Wgsl(SHADER_SOURCE.into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("map"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let attributes = wgpu::vertex_attr_array![
            0 => Uint32x4, 1 => Sint32x4, 2 => Uint32x4, 3 => Uint32x4, 4 => Sint32x4
        ];
        let make = |role: Role| {
            let (compare, write) = match role {
                Role::Tile => (wgpu::CompareFunction::LessEqual, true),
                Role::Writer => (wgpu::CompareFunction::Less, true),
                Role::Reader => (wgpu::CompareFunction::Less, false),
                // spec: S-0102 claim 25 (a drawing with no depth test neither reads nor writes depth)
                Role::Ignore => (wgpu::CompareFunction::Always, false),
                Role::BlendTile => (wgpu::CompareFunction::LessEqual, true),
                Role::BlendWriter => (wgpu::CompareFunction::Less, true),
                Role::BlendReader => (wgpu::CompareFunction::Less, false),
                Role::BlendIgnore => (wgpu::CompareFunction::Always, false),
                // spec: S-0518 claim 20 (a beam reads depth and never writes it; strictly less passes)
                Role::Spot => (wgpu::CompareFunction::Less, false),
            };
            // design: render (#114): source + target × source alpha; `fs_main` gives an opaque
            // drawing alpha 0 (its own colour) and a shadow black with alpha `SHADOW_FACTOR`
            // (the target darkened), so bodies and shadows share a pipeline and a draw
            let blend = matches!(role, Role::Writer | Role::Reader | Role::Ignore).then_some(
                wgpu::BlendState {
                    color: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::One,
                        dst_factor: wgpu::BlendFactor::SrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha: wgpu::BlendComponent::OVER,
                },
            );
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("map"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    compilation_options: Default::default(),
                    buffers: &[Some(wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Instance>() as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &attributes,
                    })],
                },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: DEPTH_FORMAT,
                    depth_write_enabled: Some(write),
                    depth_compare: Some(compare),
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some(if role == Role::Spot {
                        "fs_spot"
                    } else {
                        "fs_main"
                    }),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: target_format,
                        blend,
                        write_mask: wgpu::ColorWrites::COLOR,
                    })],
                }),
                multiview_mask: None,
                cache: None,
            })
        };
        let pipelines = ROLES.map(make);
        let instance_capacity = 1024;
        let instances = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("map instances"),
            size: (instance_capacity * std::mem::size_of::<Instance>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Ok(Self {
            pipelines,
            bind_group,
            layout,
            view_buffer,
            instances,
            instance_capacity,
            slots,
            sizes,
            signed_slots,
            rows_per_column,
            atlas: atlas_texture,
            atlas_layers: layers,
            max_layers,
            shelves,
            row_count: scene.rows.len(),
            rewrites_seen: scene.rewrites,
            row_rewrites_seen: scene.row_rewrites,
            target_format,
            rows: rows_texture,
            alpha: (alpha, (1, 1)),
            alpha_default: true,
            alpha_held: Vec::new(),
            backdrop,
            stats: MapStats::default(),
            list: Vec::new(),
            rects: Vec::new(),
            spare: Vec::new(),
            list_held: false,
            cull: None,
            resized: Vec::new(),
            near: Vec::new(),
            spot_stamps: SpotStamps::default(),
        })
    }

    /// The counters since the last call (or the upload), and a fresh start.
    pub fn take_stats(&mut self) -> MapStats {
        std::mem::take(&mut self.stats)
    }

    /// How full the atlas and the row texture are now.
    pub fn atlas_use(&self) -> AtlasUse {
        AtlasUse {
            side: self.shelves.side,
            layers: self.atlas_layers,
            committed_px: self.shelves.committed(),
            rows: self.row_count,
        }
    }

    /// Uploads what `scene` gained since the upload or the last sync: images
    /// appended to [`Scene::images`] go into the atlas's free space, images
    /// rewritten in place ([`Scene::replace_image`], [`Scene::resize_image`])
    /// are written again, into their own slots or, at another size, into
    /// new ones (their old places are freed), and palette rows appended to
    /// [`Scene::rows`] are written into the row texture's room, or replace
    /// it with a roomier one when it is full, and rows rewritten in place
    /// ([`Scene::replace_rows`]) are written again (design: render (#107)).
    /// Images and rows already uploaded must otherwise be unchanged. When
    /// nothing fits, the atlas
    /// grows (design: render (#105)). Fails when it cannot grow any more
    /// (the device's layer limit) or the scene lost images; upload the
    /// scene anew then, which packs only the images it holds.
    pub fn sync(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &Scene,
    ) -> Result<(), UploadError> {
        if scene.images.len() < self.slots.len() || scene.signed.len() < self.signed_slots.len() {
            return Err(UploadError("the scene lost images".into()));
        }
        if scene.rewrites != self.rewrites_seen {
            for (&id, &at) in &scene.rewritten {
                if at <= self.rewrites_seen {
                    continue;
                }
                let i = id as usize;
                let (Some(&slot), Some(img)) = (self.slots.get(i), scene.images.get(i)) else {
                    continue;
                };
                let size = (img.width(), img.height());
                if size == self.sizes[i] {
                    self.write(queue, slot, size.0, size.1, img.pixels());
                    continue;
                }
                // design: render (#105): a rewrite at another size leaves its place for a new one
                if slot.present() {
                    self.shelves.free(slot, self.sizes[i].0);
                }
                self.slots[i] = Slot::ABSENT;
                self.sizes[i] = size;
                // design: render (#113): the items that draw it may cover other buckets now
                self.resized.push(id);
                let slot = self.place(device, queue, scene, size)?;
                self.slots[i] = slot;
                self.write(queue, slot, size.0, size.1, img.pixels());
            }
            self.rewrites_seen = scene.rewrites;
        }
        for i in self.slots.len()..scene.images.len() {
            let img = &scene.images[i];
            let size = (img.width(), img.height());
            // Unplaced while it finds a place, so a growth leaves it out.
            self.slots.push(Slot::ABSENT);
            self.sizes.push(size);
            let slot = self.place(device, queue, scene, size)?;
            self.slots[i] = slot;
            self.write(queue, slot, size.0, size.1, img.pixels());
        }
        for i in self.signed_slots.len()..scene.signed.len() {
            let img = &scene.signed[i];
            self.signed_slots.push(Slot::ABSENT);
            let slot = self.place(device, queue, scene, (img.width, img.height))?;
            self.signed_slots[i] = slot;
            let bytes = bytemuck::cast_slice::<i8, u8>(&img.values);
            self.write(queue, slot, img.width, img.height, bytes);
        }
        if scene.row_rewrites != self.row_rewrites_seen {
            // design: render (#107): rows rewritten in place (a freed table's rows taken by
            // another) are written again, in runs of neighbouring rows
            let held = self.row_count.min(scene.rows.len());
            let mut run: Option<(usize, usize)> = None;
            for (&r, &at) in &scene.rows_rewritten {
                let r = r as usize;
                if at <= self.row_rewrites_seen || r >= held {
                    continue;
                }
                run = match run {
                    Some((start, end)) if end == r => Some((start, r + 1)),
                    Some((start, end)) => {
                        self.write_row_range(queue, scene, start, end);
                        Some((r, r + 1))
                    }
                    None => Some((r, r + 1)),
                };
            }
            if let Some((start, end)) = run {
                self.write_row_range(queue, scene, start, end);
            }
            self.row_rewrites_seen = scene.row_rewrites;
        }
        let room = (self.rows.width() / 256 * self.rows_per_column) as usize;
        if scene.rows.len() > self.row_count && scene.rows.len() <= room {
            // design: render (#107): rows appended within the texture's room are written on
            // their own
            self.write_row_range(queue, scene, self.row_count, scene.rows.len());
            self.row_count = scene.rows.len();
        } else if scene.rows.len() != self.row_count {
            let (rows, per_column) =
                rows_texture(device, queue, scene, row_side(device), self.target_format)?;
            self.stats.uploads += 1;
            self.stats.upload_bytes += u64::from(rows.width()) * u64::from(rows.height()) * 4;
            self.bind_group = bind_group(
                device,
                &self.layout,
                &self.view_buffer,
                &self.atlas,
                &rows,
                &self.alpha.0,
                &self.backdrop,
            );
            self.rows = rows;
            self.rows_per_column = per_column;
            self.row_count = scene.rows.len();
        }
        Ok(())
    }

    /// Writes `scene`'s rows `start .. end` into the row texture, which has
    /// room for them, a run of rows per column (design: render (#107)).
    fn write_row_range(&mut self, queue: &wgpu::Queue, scene: &Scene, start: usize, end: usize) {
        let per_column = self.rows_per_column as usize;
        let last = end.min(scene.rows.len());
        let mut r = start;
        while r < last {
            let (col, y) = (r / per_column, r % per_column);
            let end = last.min((col + 1) * per_column);
            let bytes: Vec<u8> = scene.rows[r..end].iter().flat_map(row_texels).collect();
            let n = (end - r) as u32;
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &self.rows,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: col as u32 * 256,
                        y: y as u32,
                        z: 0,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                &bytes,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(256 * 4),
                    rows_per_image: Some(n),
                },
                wgpu::Extent3d {
                    width: 256,
                    height: n,
                    depth_or_array_layers: 1,
                },
            );
            self.stats.uploads += 1;
            self.stats.upload_bytes += bytes.len() as u64;
            r = end;
        }
    }

    /// A place in the atlas for a `size` image: [`Slot::ABSENT`] for an
    /// empty image or one larger than a layer (with a warning), else a
    /// free place, growing the atlas when there is none (design: render
    /// (#105)).
    fn place(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &Scene,
        (w, h): (u32, u32),
    ) -> Result<Slot, UploadError> {
        let side = self.shelves.side;
        if w == 0 || h == 0 {
            return Ok(Slot::ABSENT);
        }
        if too_large(w, h, side) {
            // design: render (#108): left out with no place used, so no rebuild follows
            warn_absent([((w, h), Slot::ABSENT)], side);
            return Ok(Slot::ABSENT);
        }
        loop {
            if let Some(slot) = self.shelves.place(w, h) {
                return Ok(slot);
            }
            self.grow(device, queue, scene)?;
        }
    }

    /// Grows the atlas by [`grown_layers`]: a new texture into which every
    /// placed image is written again from `scene`, into its own slot. The
    /// slots, pipelines, row texture and targets stay (design: render
    /// (#105)). Fails at the device's layer limit.
    fn grow(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &Scene,
    ) -> Result<(), UploadError> {
        let layers = grown_layers(self.atlas_layers, self.max_layers);
        if layers <= self.atlas_layers {
            return Err(UploadError(format!(
                "the atlas is full at the device's limit of {} layers",
                self.max_layers
            )));
        }
        let side = self.shelves.side;
        let atlas = atlas_texture(device, side, layers);
        for (i, &slot) in self.slots.iter().enumerate() {
            let img = &scene.images[i];
            // An image rewritten at another size and not yet moved is written
            // when it is.
            if (img.width(), img.height()) == self.sizes[i] {
                self.stats.uploads += 1;
                self.stats.upload_bytes += u64::from(img.width()) * u64::from(img.height());
                write_image(queue, &atlas, slot, img.width(), img.height(), img.pixels());
            }
        }
        for (img, &slot) in scene.signed.iter().zip(&self.signed_slots) {
            self.stats.uploads += 1;
            self.stats.upload_bytes += u64::from(img.width) * u64::from(img.height);
            let bytes = bytemuck::cast_slice::<i8, u8>(&img.values);
            write_image(queue, &atlas, slot, img.width, img.height, bytes);
        }
        self.shelves.set_layers(layers);
        self.atlas = atlas;
        self.atlas_layers = layers;
        self.bind_group = bind_group(
            device,
            &self.layout,
            &self.view_buffer,
            &self.atlas,
            &self.rows,
            &self.alpha.0,
            &self.backdrop,
        );
        log::info!("the battlefield atlas grew to {layers} layers of {side} × {side}");
        Ok(())
    }

    /// Writes a `w` × `h` image of bytes `pixels` into the atlas at `slot`.
    fn write(&mut self, queue: &wgpu::Queue, slot: Slot, w: u32, h: u32, pixels: &[u8]) {
        if w > 0 && h > 0 && slot.present() {
            self.stats.uploads += 1;
            self.stats.upload_bytes += u64::from(w) * u64::from(h);
            write_image(queue, &self.atlas, slot, w, h, pixels);
        }
    }

    /// Records the scene for `view` into `encoder`, rendering into `target`
    /// (a [`COLOR_FORMAT`] texture of the view's size that can be copied
    /// from, as [`targets`] makes it) with `depth` (a [`DEPTH_FORMAT`] view
    /// of the same size). Returns the number of items drawn.
    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::Texture,
        depth: &wgpu::TextureView,
        scene: &Scene,
        view: &View,
        filter: &Filter,
        q: &ComposeQuirks,
    ) -> usize {
        self.render_lit(
            device, queue, encoder, target, depth, scene, view, filter, q, None,
        )
    }

    /// [`Self::render`] with the light-alpha values `alpha` of the view's
    /// pixels, row by row (S-0110 claims 5 and 6; `None`: 127 everywhere).
    ///
    /// A translucent item (S-0102 claim 32) blends with the pixels drawn
    /// before it: the pass ends before it, the target's pixels under it
    /// are copied to the backdrop texture, and the pass resumes, so the
    /// shader applies F8 to the packed channels exactly, as the CPU
    /// reference does. This needs a target of [`COLOR_FORMAT`].
    #[allow(clippy::too_many_arguments)]
    pub fn render_lit(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::Texture,
        depth: &wgpu::TextureView,
        scene: &Scene,
        view: &View,
        filter: &Filter,
        q: &ComposeQuirks,
        alpha: Option<&[u8]>,
    ) -> usize {
        self.set_alpha(device, queue, view.size, alpha);
        let uniform = ViewUniform {
            origin: [view.origin.0, view.origin.1],
            size: [view.size.0 as i32, view.size.1 as i32],
            reference: view.reference,
            rows_per_column: self.rows_per_column as i32,
            pad: [0; 2],
        };
        queue.write_buffer(&self.view_buffer, 0, bytemuck::bytes_of(&uniform));
        let (ox, oy) = view.origin;
        let (w, h) = (view.size.0 as i32, view.size.1 as i32);
        // The buffers of the frame before the last, emptied, so that a frame
        // does not grow them again; the last frame's list stays, to compare.
        let mut list: Vec<Instance> = std::mem::take(&mut self.spare);
        list.clear();
        // The view rectangle each instance draws into, for placing the
        // backdrop copies (`passes`).
        let mut rects: Vec<[u32; 4]> = std::mem::take(&mut self.rects);
        rects.clear();
        // A spot instance draws its own pixel.
        let spot_rects = |list: &[Instance], rects: &mut Vec<[u32; 4]>| {
            for i in &list[rects.len()..] {
                rects.push([(i.place[0] - ox) as u32, (i.place[1] - oy) as u32, 1, 1]);
            }
        };
        // Runs of instances that share a pipeline; a translucent item is a
        // run of its own, with the view rectangle it covers.
        let mut batches: Vec<Batch> = Vec::new();
        let mut spot_groups = scene.spots.iter().peekable();
        // design: render (#113): the cell pass's items that may meet the view, from the cull
        // kept while the cell pass stays, then every item of the object pass; an item left out
        // would draw no pixel, so the instances and their order are those of every item
        let cell_len = scene.cell_pass_len.min(scene.items.len());
        if !self
            .cull
            .as_ref()
            .is_some_and(|c| c.holds(scene, &self.resized))
        {
            self.cull = (cell_len > 0).then(|| CellCull::new(scene));
        }
        self.resized.clear();
        let mut near = std::mem::take(&mut self.near);
        match &self.cull {
            Some(c) => c.near(view.origin, (w, h), &mut near),
            None => near.clear(),
        }
        #[cfg(debug_assertions)]
        check_near(scene, view, &near);
        let order = near
            .iter()
            .map(|&n| n as usize)
            .chain(cell_len..scene.items.len());
        // spec: S-0109 claim 27 (a stack's copies, lowest first, as many as the view's top row allows)
        let copies = order.flat_map(|n| {
            let i = &scene.items[n];
            let count = i.stack.map_or(1, |s| s.copies(oy));
            (0..count).map(move |j| (n, i, j))
        });
        for (n, base, j) in copies {
            self.stats.items += 1;
            while let Some(g) = spot_groups.next_if(|g| g.at <= n) {
                if filter.shows_spots(g) {
                    spot_instances(g, view, &mut self.spot_stamps, &mut list, &mut batches);
                    spot_rects(&list, &mut rects);
                }
            }
            // Copy 0 is the item itself; a higher copy moves only its row and bias.
            let moved;
            let item = if j == 0 {
                base
            } else {
                moved = base.copies(oy).nth(j as usize).unwrap_or(*base);
                &moved
            };
            if !filter.shows(item) {
                continue;
            }
            let (l, t, r, b) = scene.bounds(item);
            // spec: S-0101 F4b (an image is drawn iff its drawn extent meets the view; a bounds miss draws no pixel)
            if r <= ox || b <= oy || l >= ox + w || t >= oy + h {
                continue;
            }
            let img = &scene.images[item.image as usize];
            let slot = self.slots[item.image as usize];
            if !slot.present() {
                // design: render (#108): an image larger than the atlas is not drawn
                continue;
            }
            let mut companion_rect = [0; 4];
            let (role, flags, bias, reference_row, depth_slot) = match item.depth {
                ItemDepth::Tile {
                    depth,
                    reference_row,
                } => (
                    // spec: S-0102 claim 9 (a tile without depth data does not touch the buffer: it neither reads nor writes depth)
                    if depth.is_some() {
                        Role::Tile
                    } else {
                        Role::Ignore
                    },
                    0u32,
                    0,
                    reference_row,
                    depth.map(|d| self.slots[d as usize]),
                ),
                ItemDepth::Profile {
                    profile,
                    bias,
                    companion,
                } => {
                    let mut kind = match profile {
                        Profile::Flat => 1,
                        Profile::Slope => 2,
                        Profile::Upright => 3,
                    };
                    // spec: S-0102 F2, F5 (a companion depth image: bit 2 of the flags, its atlas
                    // slot in the depth fields, its world rectangle in `companion`)
                    let companion_slot = companion.map(|(i, cx, cy)| {
                        let img = &scene.signed[i as usize];
                        kind |= 4;
                        companion_rect = [cx, cy, img.width as i32, img.height as i32];
                        self.signed_slots[i as usize]
                    });
                    let role = match (item.paint, item.test) {
                        // spec: S-0102 claim 27 (a cell-pass shadow stores its depth, quirks.cell_pass_shadows_write_depth)
                        (Paint::Shadow, DepthTest::Writer) if q.cell_pass_shadows_write_depth => {
                            Role::Writer
                        }
                        // spec: S-0102 claim 25 (no depth test: drawn whatever the stored depth), S-0111 claim 16 (the self-heal marker)
                        (Paint::Shadow, DepthTest::Ignore) => Role::Ignore,
                        // A shadow otherwise reads depth and stores none.
                        (Paint::Shadow, _) => Role::Reader,
                        (Paint::Translucent(..), DepthTest::Reader) => Role::BlendReader,
                        (Paint::Translucent(..), DepthTest::Tile) => Role::BlendTile,
                        (Paint::Translucent(..), DepthTest::Writer) => Role::BlendWriter,
                        (Paint::Translucent(..), DepthTest::Ignore) => Role::BlendIgnore,
                        (_, DepthTest::Reader) => Role::Reader,
                        (_, DepthTest::Tile) => Role::Tile,
                        (_, DepthTest::Writer) => Role::Writer,
                        (_, DepthTest::Ignore) => Role::Ignore,
                    };
                    (role, kind, bias, 0, companion_slot)
                }
            };
            // `diamond` holds the diamond flag in bit 0, the translucency
            // code in bits 1 and 2, the pixel-particle flag in bit 3, the
            // shadow flag in bit 4 and the protection colour mask in bits 16
            // to 31.
            let (row, diamond, shade_rows, rho) = match item.paint {
                Paint::Row(r) => (r as u32, 0, 0, 0),
                // spec: S-0010 claim 22 (a diamond has no transparent index)
                Paint::Diamond(r) => (r as u32, 1, 0, 0),
                // spec: S-0102 claim 34, F9 (the opaque drawing's mask K)
                Paint::Lit(s) => (
                    s.base,
                    u32::from(s.mask) << 16,
                    u32::from(s.rows),
                    u32::from(s.rho),
                ),
                Paint::LitDiamond(s) => (
                    s.base,
                    1 | (u32::from(s.mask) << 16),
                    u32::from(s.rows),
                    u32::from(s.rho),
                ),
                // spec: S-0102 claim 32 (codes 1 to 3 blend by F8)
                Paint::Translucent(s, t) => {
                    (s.base, t.code() << 1, u32::from(s.rows), u32::from(s.rho))
                }
                // design: render (#114): bit 4 marks a shadow
                Paint::Shadow => (0, 1 << 4, 0, 0),
                // spec: S-0616 claim 28 (a pixel particle's own colour, flagged by bit 3)
                Paint::Pixel(c) => (
                    (u32::from(c.r) << 16) | (u32::from(c.g) << 8) | u32::from(c.b),
                    1 << 3,
                    0,
                    0,
                ),
            };
            if depth_slot.is_some_and(|s| !s.present()) {
                // design: render (#108): nor is an item whose depth image is larger than the atlas
                continue;
            }
            let ds = depth_slot.unwrap_or(Slot {
                x: 0,
                y: 0,
                layer: NO_IMAGE,
            });
            list.push(Instance {
                image: [
                    slot.x,
                    slot.y,
                    slot.layer,
                    img.width() | (img.height() << 16),
                ],
                place: [item.x, item.y, bias, reference_row],
                depth: [ds.x, ds.y, ds.layer, flags],
                paint: [row, diamond, shade_rows, rho],
                companion: companion_rect,
            });
            let n = list.len() as u32;
            // The view pixels the item covers (a translucent item blends
            // with them).
            let x0 = (l - ox).max(0) as u32;
            let y0 = (t - oy).max(0) as u32;
            let x1 = (r - ox).min(w) as u32;
            let y1 = (b - oy).min(h) as u32;
            let rect = [x0, y0, x1 - x0, y1 - y0];
            rects.push(rect);
            if role.translucent() {
                batches.push((role, n - 1..n, Some(rect)));
                continue;
            }
            match batches.last_mut() {
                Some((r, range, None)) if *r == role => range.end = n,
                _ => batches.push((role, n - 1..n, None)),
            }
        }
        for g in spot_groups.filter(|g| filter.shows_spots(g)) {
            spot_instances(g, view, &mut self.spot_stamps, &mut list, &mut batches);
            spot_rects(&list, &mut rects);
        }
        self.near = near;
        if list.len() > self.instance_capacity {
            self.instance_capacity = list.len().next_power_of_two();
            self.instances = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("map instances"),
                size: (self.instance_capacity * std::mem::size_of::<Instance>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.list_held = false;
        }
        // design: render (#113): the instances the buffer already holds at their places (the
        // last frame's, up to the first that differs) are not written again
        let same = if self.list_held {
            list.iter()
                .zip(&self.list)
                .take_while(|(a, b)| bytemuck::bytes_of(*a) == bytemuck::bytes_of(*b))
                .count()
        } else {
            0
        };
        if same < list.len() {
            let at = (same * std::mem::size_of::<Instance>()) as u64;
            queue.write_buffer(&self.instances, at, bytemuck::cast_slice(&list[same..]));
            self.stats.uploads += 1;
            self.stats.upload_bytes += std::mem::size_of_val(&list[same..]) as u64;
        }
        self.list_held = true;
        self.stats.instances += list.len();
        let target_view = target.create_view(&Default::default());
        let begin = |encoder: &mut wgpu::CommandEncoder, first: bool| {
            let mut pass = encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("map"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &target_view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: if first {
                                wgpu::LoadOp::Clear(wgpu::Color::BLACK)
                            } else {
                                wgpu::LoadOp::Load
                            },
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: depth,
                        // spec: S-0102 claim 8 (the depth buffer is filled with 65535 on a full redraw)
                        depth_ops: Some(wgpu::Operations {
                            load: if first {
                                wgpu::LoadOp::Clear(
                                    f32::from(crate::compose::DEPTH_CLEAR) / 65536.0,
                                )
                            } else {
                                wgpu::LoadOp::Load
                            },
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    ..Default::default()
                })
                .forget_lifetime();
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.set_vertex_buffer(0, self.instances.slice(..));
            pass
        };
        self.stats.draws += batches.len();
        let mut first = true;
        for (copies, run) in passes(batches, &rects) {
            for [x, y, cw, ch] in copies {
                // spec: S-0102 claim 32 (a translucent pixel blends with the already-painted
                // destination: the pixels drawn so far are copied before the item is drawn)
                encoder.copy_texture_to_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: target,
                        mip_level: 0,
                        origin: wgpu::Origin3d { x, y, z: 0 },
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::TexelCopyTextureInfo {
                        texture: &self.backdrop,
                        mip_level: 0,
                        origin: wgpu::Origin3d { x, y, z: 0 },
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::Extent3d {
                        width: cw,
                        height: ch,
                        depth_or_array_layers: 1,
                    },
                );
                self.stats.copies += 1;
                self.stats.copied_px += u64::from(cw) * u64::from(ch);
            }
            let mut pass = begin(encoder, first);
            first = false;
            self.stats.passes += 1;
            for (role, range, _) in run {
                pass.set_pipeline(&self.pipelines[role.index()]);
                pass.draw(0..6, range);
            }
        }
        let drawn = list.len();
        self.spare = std::mem::replace(&mut self.list, list);
        self.rects = rects;
        drawn
    }
}

/// A run of instances sharing a pipeline, with the view rectangle a
/// blending run covers.
type Batch = (Role, std::ops::Range<u32>, Option<[u32; 4]>);

/// The render passes of a frame's `batches` (`rects`: the view rectangle
/// each instance draws into), each with the backdrop copies taken before
/// it begins.
///
/// A blending run reads the copy of the pixels under it, which must hold
/// what was drawn before it. The copy is taken where the last pass began
/// when nothing drawn since then meets the run's rectangle (the pixels
/// there are the same at both moments), else a new pass begins before the
/// run. So runs that do not meet what was drawn since share one pass break
/// instead of one each (M8 step C4); the first pass, which clears the
/// target, takes no copy. There is always at least one pass.
fn passes(batches: Vec<Batch>, rects: &[[u32; 4]]) -> Vec<(Vec<[u32; 4]>, Vec<Batch>)> {
    let meets = |a: &[u32; 4], b: &[u32; 4]| {
        a[0] < b[0] + b[2] && b[0] < a[0] + a[2] && a[1] < b[1] + b[3] && b[1] < a[1] + a[3]
    };
    let mut out: Vec<(Vec<[u32; 4]>, Vec<Batch>)> = vec![(Vec::new(), Vec::new())];
    // The rectangles drawn since the last pass began.
    let mut drawn: Vec<[u32; 4]> = Vec::new();
    for b in batches {
        if let Some(c) = b.2 {
            if out.len() > 1 && !drawn.iter().any(|d| meets(d, &c)) {
                if let Some(last) = out.last_mut() {
                    last.0.push(c);
                }
            } else {
                out.push((vec![c], Vec::new()));
                drawn.clear();
            }
        }
        drawn.extend_from_slice(&rects[b.1.start as usize..b.1.end as usize]);
        if let Some(last) = out.last_mut() {
            last.1.push(b);
        }
    }
    out
}

/// Appends the instances of spot group `g` (one 1 × 1 instance per spot in
/// the view) and their batches. A batch reads the copy of the target taken
/// before it, so a batch ends before a spot that hits a pixel the batch
/// already holds: overlapping spots then apply in order, as on the CPU
/// ([`crate::spots::apply`]). A magnetic wave's spots never repeat a pixel
/// and read the copy around them (S-0607 claim 25).
///
/// A group with no magnetic wave is laid out by [`layered_spot_instances`]
/// instead (design: render (#115, #116)).
fn spot_instances(
    g: &SpotGroup,
    view: &View,
    stamps: &mut SpotStamps,
    list: &mut Vec<Instance>,
    batches: &mut Vec<Batch>,
) {
    if g.spots.len() < SpotStamps::LAYERS as usize
        && !g.spots.iter().any(|s| matches!(s.kind, SpotKind::Wave { .. }))
    {
        layered_spot_instances(g, view, stamps, list, batches);
        return;
    }
    let (ox, oy) = view.origin;
    let (w, h) = (view.size.0 as i32, view.size.1 as i32);
    let mut held: std::collections::HashSet<(i32, i32)> = std::collections::HashSet::new();
    let mut rect: Option<[i32; 4]> = None;
    let mut start = list.len() as u32;
    let close =
        |list: &Vec<Instance>, start: u32, rect: Option<[i32; 4]>, batches: &mut Vec<Batch>| {
            let n = list.len() as u32;
            if let Some([x0, y0, x1, y1]) = rect
                && n > start
            {
                let (x0, y0) = (x0.max(0), y0.max(0));
                let (x1, y1) = (x1.min(w), y1.min(h));
                batches.push((
                    Role::Spot,
                    start..n,
                    Some([x0 as u32, y0 as u32, (x1 - x0) as u32, (y1 - y0) as u32]),
                ));
            }
        };
    for s in &g.spots {
        let (x, y) = (s.x - ox, s.y - oy);
        if x < 0 || y < 0 || x >= w || y >= h {
            continue;
        }
        let (kind, colour, weights, shift) = match s.kind {
            SpotKind::Opaque { colour } => (0u32, u32::from(colour), 0u32, (0, 0)),
            SpotKind::Additive { rgb } => (1, rgb_word(rgb), 0, (0, 0)),
            SpotKind::Blended {
                rgb,
                line,
                screen,
                alpha_from,
            } => (
                2,
                rgb_word(rgb),
                u32::from(line) | (u32::from(screen) << 16),
                alpha_from,
            ),
            SpotKind::Flash { rgb } => (3, rgb_word(rgb), 0, (0, 0)),
            SpotKind::Wave { shift, k } => {
                let m = crate::spots::WAVE_MARGIN;
                if x < m || y < m || x >= w - m || y >= h - m {
                    continue;
                }
                (4, 0, u32::from(k), shift)
            }
        };
        if !held.insert((x, y)) {
            close(list, start, rect, batches);
            start = list.len() as u32;
            held.clear();
            held.insert((x, y));
            rect = None;
        }
        // The rectangle of target pixels the batch reads: its own pixels and,
        // for a wave, the source pixels.
        let (sx, sy) = (x + i32::from(shift.0), y + i32::from(shift.1));
        let r = rect.get_or_insert([x, y, x + 1, y + 1]);
        r[0] = r[0].min(x).min(sx);
        r[1] = r[1].min(y).min(sy);
        r[2] = r[2].max(x + 1).max(sx + 1);
        r[3] = r[3].max(y + 1).max(sy + 1);
        list.push(Instance {
            image: [0, 0, NO_IMAGE, 1 | (1 << 16)],
            place: [s.x, s.y, s.bias, 0],
            depth: [0, 0, NO_IMAGE, 0],
            paint: [kind, colour, weights, 0],
            companion: [i32::from(shift.0), i32::from(shift.1), 0, 0],
        });
    }
    close(list, start, rect, batches);
}

/// The view's pixels marked for laying out one spot group, kept for their
/// room (design: render (#115, #116)): a word a pixel, the group's
/// generation in the top 8 bits and the layer of the last spot at the pixel
/// in the low 24, so that a new group needs no clearing.
#[derive(Default)]
struct SpotStamps {
    cells: Vec<u32>,
    /// The generation of the group being laid out, 1 to 255 (0 marks
    /// nothing).
    generation: u32,
    /// Each in-view spot's layer, in group order.
    layers: Vec<u32>,
    /// Each layer's spot count, then where its spots go.
    counts: Vec<u32>,
    /// Each layer's rectangle of reading spots (x0, y0, x1, y1).
    reads: Vec<Option<[i32; 4]>>,
    /// The group's instances in layer order.
    sorted: Vec<Instance>,
}

impl SpotStamps {
    /// The layers a word holds; a group with as many spots or more is laid
    /// out spot by spot.
    const LAYERS: u32 = 1 << 24;

    /// Starts a group over a view of `n` pixels.
    fn begin(&mut self, n: usize) {
        if self.cells.len() != n {
            self.cells.clear();
            self.cells.resize(n, 0);
            self.generation = 0;
        }
        self.generation += 1;
        if self.generation > 255 {
            self.cells.fill(0);
            self.generation = 1;
        }
    }

    /// The layer of the group's last spot at pixel `i`, if one is there.
    fn get(&self, i: usize) -> Option<u32> {
        let c = self.cells[i];
        (c >> 24 == self.generation).then_some(c & (Self::LAYERS - 1))
    }

    fn set(&mut self, i: usize, layer: u32) {
        self.cells[i] = (self.generation << 24) | layer;
    }
}

/// Appends the instances of spot group `g`, which holds no magnetic wave,
/// and their batches, one batch a layer (design: render (#115, #116)).
///
/// A spot that reads the target (additive and blended, S-0518 F7 and F8)
/// reads the copy taken before its batch, which must hold every earlier
/// spot at its pixel; one that does not (opaque, S-0518 claim 23; a beam
/// flash, S-0514 claim 25) only replaces the pixel. So the first spot at a
/// pixel is in layer 0, and a later one in the layer of the spot before it
/// there, one higher if it reads. The layers are drawn in turn, each spot
/// after every earlier spot at its pixel, and within a layer in group
/// order, which the draw keeps; spots read and write no depth (S-0518 claim
/// 20), so none changes whether another is drawn. A layer's batch covers
/// the pixels its reading spots read, and a layer without one takes no
/// copy: the pixels are those [`crate::spots::apply`] gives, with a copy
/// and a pass for each depth of repetition instead of each repeat.
fn layered_spot_instances(
    g: &SpotGroup,
    view: &View,
    stamps: &mut SpotStamps,
    list: &mut Vec<Instance>,
    batches: &mut Vec<Batch>,
) {
    let (ox, oy) = view.origin;
    let (w, h) = (view.size.0 as i32, view.size.1 as i32);
    stamps.begin(view.size.0 as usize * view.size.1 as usize);
    stamps.layers.clear();
    stamps.reads.clear();
    let start = list.len();
    let mut top = 0;
    for s in &g.spots {
        let (x, y) = (s.x - ox, s.y - oy);
        if x < 0 || y < 0 || x >= w || y >= h {
            continue;
        }
        let (kind, colour, weights, alpha_from, reads) = match s.kind {
            SpotKind::Opaque { colour } => (0u32, u32::from(colour), 0u32, (0, 0), false),
            SpotKind::Additive { rgb } => (1, rgb_word(rgb), 0, (0, 0), true),
            SpotKind::Blended {
                rgb,
                line,
                screen,
                alpha_from,
            } => (
                2,
                rgb_word(rgb),
                u32::from(line) | (u32::from(screen) << 16),
                alpha_from,
                true,
            ),
            SpotKind::Flash { rgb } => (3, rgb_word(rgb), 0, (0, 0), false),
            SpotKind::Wave { .. } => continue,
        };
        let i = y as usize * view.size.0 as usize + x as usize;
        let layer = stamps.get(i).map_or(0, |l| l + u32::from(reads));
        stamps.set(i, layer);
        stamps.layers.push(layer);
        top = top.max(layer);
        if reads {
            // A blended spot's `alpha_from` reads the light-alpha values, not the target.
            let l = layer as usize;
            if stamps.reads.len() <= l {
                stamps.reads.resize(l + 1, None);
            }
            let r = stamps.reads[l].get_or_insert([x, y, x + 1, y + 1]);
            r[0] = r[0].min(x);
            r[1] = r[1].min(y);
            r[2] = r[2].max(x + 1);
            r[3] = r[3].max(y + 1);
        }
        list.push(Instance {
            image: [0, 0, NO_IMAGE, 1 | (1 << 16)],
            place: [s.x, s.y, s.bias, 0],
            depth: [0, 0, NO_IMAGE, 0],
            paint: [kind, colour, weights, 0],
            companion: [i32::from(alpha_from.0), i32::from(alpha_from.1), 0, 0],
        });
    }
    if list.len() == start {
        return;
    }
    let layers = top as usize + 1;
    stamps.counts.clear();
    stamps.counts.resize(layers, 0);
    for &l in &stamps.layers {
        stamps.counts[l as usize] += 1;
    }
    if layers > 1 {
        // A stable counting sort by layer.
        let mut at = 0;
        for c in &mut stamps.counts {
            let n = *c;
            *c = at;
            at += n;
        }
        stamps.sorted.clear();
        stamps.sorted.resize(list.len() - start, Instance::zeroed());
        for (k, inst) in list[start..].iter().enumerate() {
            let slot = &mut stamps.counts[stamps.layers[k] as usize];
            stamps.sorted[*slot as usize] = *inst;
            *slot += 1;
        }
        list[start..].copy_from_slice(&stamps.sorted);
    }
    // `counts` now holds each layer's end (or, for one layer, its count).
    let mut from = start as u32;
    for l in 0..layers {
        let end = if layers > 1 {
            start as u32 + stamps.counts[l]
        } else {
            list.len() as u32
        };
        let rect = stamps.reads.get(l).copied().flatten().map(|[x0, y0, x1, y1]| {
            [x0 as u32, y0 as u32, (x1 - x0) as u32, (y1 - y0) as u32]
        });
        batches.push((Role::Spot, from..end, rect));
        from = end;
    }
}

/// Three 8-bit channels in one word, red lowest.
fn rgb_word(c: [u8; 3]) -> u32 {
    u32::from(c[0]) | (u32::from(c[1]) << 8) | (u32::from(c[2]) << 16)
}

impl MapRenderer {
    /// Makes the light-alpha texture hold `alpha` (or 127 everywhere) for
    /// a view of `size`.
    fn set_alpha(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        size: (u32, u32),
        alpha: Option<&[u8]>,
    ) {
        let size = (size.0.max(1), size.1.max(1));
        let n = size.0 as usize * size.1 as usize;
        let alpha = alpha.filter(|a| a.len() == n);
        if self.alpha.1 != size {
            let t = alpha_texture(device, queue, size);
            let backdrop = backdrop_texture(device, size);
            self.bind_group = bind_group(
                device,
                &self.layout,
                &self.view_buffer,
                &self.atlas,
                &self.rows,
                &t,
                &backdrop,
            );
            self.alpha = (t, size);
            self.backdrop = backdrop;
            self.alpha_default = true;
        }
        let bytes = match alpha {
            Some(a) if !self.alpha_default && self.alpha_held == a => return,
            Some(a) => {
                self.alpha_default = false;
                self.alpha_held.clear();
                self.alpha_held.extend_from_slice(a);
                std::borrow::Cow::Borrowed(a)
            }
            None if self.alpha_default => return,
            None => {
                self.alpha_default = true;
                self.alpha_held.clear();
                std::borrow::Cow::Owned(vec![crate::light::DEFAULT_LIGHT_ALPHA as u8; n])
            }
        };
        self.stats.uploads += 1;
        self.stats.upload_bytes += bytes.len() as u64;
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.alpha.0,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &bytes,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(size.0),
                rows_per_image: Some(size.1),
            },
            wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
        );
    }
}

/// A light-alpha texture of `size`, 127 everywhere (S-0110 claim 5).
fn alpha_texture(device: &wgpu::Device, queue: &wgpu::Queue, size: (u32, u32)) -> wgpu::Texture {
    device.create_texture_with_data(
        queue,
        &wgpu::TextureDescriptor {
            label: Some("map light-alpha"),
            size: wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Uint,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        },
        wgpu::util::TextureDataOrder::LayerMajor,
        &vec![crate::light::DEFAULT_LIGHT_ALPHA as u8; size.0 as usize * size.1 as usize],
    )
}

/// The backdrop texture of a view of `size`: a copy of the target's pixels
/// under a translucent item (S-0102 claim 32).
fn backdrop_texture(device: &wgpu::Device, size: (u32, u32)) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("map backdrop"),
        size: wgpu::Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: COLOR_FORMAT,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    })
}

/// An empty atlas texture of `layers` layers of `side` × `side`.
fn atlas_texture(device: &wgpu::Device, side: u32, layers: u32) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("map atlas"),
        size: wgpu::Extent3d {
            width: side,
            height: side,
            depth_or_array_layers: layers,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::R8Uint,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    })
}

/// Writes a `w` × `h` image of bytes `pixels` into `atlas` at `slot`; an
/// empty or unplaced image writes nothing.
fn write_image(
    queue: &wgpu::Queue,
    atlas: &wgpu::Texture,
    slot: Slot,
    w: u32,
    h: u32,
    pixels: &[u8],
) {
    if w == 0 || h == 0 || !slot.present() {
        return;
    }
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: atlas,
            mip_level: 0,
            origin: wgpu::Origin3d {
                x: slot.x,
                y: slot.y,
                z: slot.layer,
            },
            aspect: wgpu::TextureAspect::All,
        },
        pixels,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(w),
            rows_per_image: Some(h),
        },
        wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
    );
}

/// The rows per column and the columns of a palette-row texture for `rows`
/// rows whose sides are at most `side` (design: render (#107)): room for
/// half again as many rows, at least 64, so rows appended later are
/// written on their own until the room runs out; `None` past `side ÷ 256`
/// columns of `side` rows.
fn row_layout(rows: usize, side: u32) -> Option<(u32, u32)> {
    let most = (side / 256) as usize * side as usize;
    if most == 0 || rows > most {
        return None;
    }
    let room = (rows + rows / 2).max(64).min(most);
    let per_column = room.min(side as usize) as u32;
    Some((per_column, (room as u32).div_ceil(per_column)))
}

/// The texels of palette row `row`, as the row texture holds them.
fn row_texels(row: &[u16; 256]) -> impl Iterator<Item = u8> + '_ {
    row.iter().flat_map(|&c| {
        let (r, g, b) = expand565(c);
        [r, g, b, 255]
    })
}

/// The palette-row texture of `scene` and its rows per column, with room
/// for rows appended later ([`row_layout`]).
fn rows_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    scene: &Scene,
    side: u32,
    target_format: wgpu::TextureFormat,
) -> Result<(wgpu::Texture, u32), UploadError> {
    let (rows_per_column, columns) = row_layout(scene.rows.len(), side)
        .ok_or_else(|| UploadError("too many palette rows".into()))?;
    let mut row_bytes = vec![0u8; (columns * 256 * rows_per_column * 4) as usize];
    for (r, row) in scene.rows.iter().enumerate() {
        let (col, y) = (r as u32 / rows_per_column, r as u32 % rows_per_column);
        let at = ((y * columns * 256 + col * 256) * 4) as usize;
        for (to, from) in row_bytes[at..at + 256 * 4].iter_mut().zip(row_texels(row)) {
            *to = from;
        }
    }
    let texture = device.create_texture_with_data(
        queue,
        &wgpu::TextureDescriptor {
            label: Some("map rows"),
            size: wgpu::Extent3d {
                width: columns * 256,
                height: rows_per_column,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: if target_format.is_srgb() {
                wgpu::TextureFormat::Rgba8UnormSrgb
            } else {
                wgpu::TextureFormat::Rgba8Unorm
            },
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        },
        wgpu::util::TextureDataOrder::LayerMajor,
        &row_bytes,
    );
    Ok((texture, rows_per_column))
}

/// The map pass's bind group.
fn bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    view_buffer: &wgpu::Buffer,
    atlas: &wgpu::Texture,
    rows: &wgpu::Texture,
    alpha: &wgpu::Texture,
    backdrop: &wgpu::Texture,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("map"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: view_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&atlas.create_view(
                    &wgpu::TextureViewDescriptor {
                        dimension: Some(wgpu::TextureViewDimension::D2Array),
                        ..Default::default()
                    },
                )),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(
                    &rows.create_view(&wgpu::TextureViewDescriptor::default()),
                ),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(
                    &alpha.create_view(&wgpu::TextureViewDescriptor::default()),
                ),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::TextureView(
                    &backdrop.create_view(&wgpu::TextureViewDescriptor::default()),
                ),
            },
        ],
    })
}

/// Creates a colour target of `format` and a depth buffer for a view of
/// `size`.
pub fn targets(
    device: &wgpu::Device,
    size: (u32, u32),
    format: wgpu::TextureFormat,
) -> (wgpu::Texture, wgpu::Texture) {
    let make = |format, usage, label| {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: size.0.max(1),
                height: size.1.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        })
    };
    (
        make(
            format,
            wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            "map colour",
        ),
        make(
            DEPTH_FORMAT,
            wgpu::TextureUsages::RENDER_ATTACHMENT,
            "map depth",
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The backdrop copies share pass breaks only where that copies the
    /// same pixels (M8 step C4): a blending run that meets nothing drawn
    /// since the last break takes its copy there; one that meets it, or
    /// any in the first pass, begins a pass of its own.
    #[test]
    fn backdrop_copies_share_breaks_only_where_nothing_changed() {
        let r = |x, y, w, h| [x, y, w, h];
        // Instances: 0 opaque over the left, 1 blending on the right, 2
        // blending further right, 3 blending over 0, 4 opaque over 1.
        let rects = [
            r(0, 0, 10, 10),
            r(20, 0, 5, 5),
            r(30, 0, 5, 5),
            r(2, 2, 4, 4),
            r(20, 0, 2, 2),
        ];
        let batches: Vec<Batch> = vec![
            (Role::Writer, 0..1, None),
            (Role::BlendReader, 1..2, Some(rects[1])),
            (Role::BlendReader, 2..3, Some(rects[2])),
            (Role::BlendReader, 3..4, Some(rects[3])),
            (Role::Reader, 4..5, None),
        ];
        let p = passes(batches, &rects);
        // The first pass (it clears) has no copy; 1 begins a pass; 2 shares
        // its break (it does not meet 1); 3 shares it too (it meets only 0,
        // drawn before the break, so the copy holds it); 4 joins the pass.
        assert_eq!(p.len(), 2);
        assert!(p[0].0.is_empty());
        assert_eq!(p[0].1.len(), 1);
        assert_eq!(p[1].0, vec![rects[1], rects[2], rects[3]]);
        assert_eq!(p[1].1.len(), 4);

        // A blending run over a blending run drawn since the break begins
        // a new pass with its own copy.
        let rects = [r(0, 0, 4, 4), r(10, 0, 4, 4), r(12, 2, 4, 4), r(0, 0, 1, 1)];
        let batches: Vec<Batch> = vec![
            (Role::Tile, 0..1, None),
            (Role::BlendWriter, 1..2, Some(rects[1])),
            (Role::BlendIgnore, 2..3, Some(rects[2])),
            (Role::Spot, 3..4, Some(rects[3])),
        ];
        let p = passes(batches, &rects);
        assert_eq!(
            p.iter().map(|(c, _)| c.len()).collect::<Vec<_>>(),
            vec![0, 1, 2]
        );
        assert_eq!(p[2].0, vec![rects[2], rects[3]]);
        // Every run is kept, in order.
        let order: Vec<u32> = p
            .iter()
            .flat_map(|(_, b)| b.iter().map(|b| b.1.start))
            .collect();
        assert_eq!(order, vec![0, 1, 2, 3]);

        // A frame that starts with a blending run, and one with no runs.
        let rects = [r(0, 0, 1, 1)];
        let p = passes(vec![(Role::Spot, 0..1, Some(rects[0]))], &rects);
        assert_eq!(p.len(), 2);
        assert!(p[0].0.is_empty() && p[0].1.is_empty());
        assert_eq!(p[1].0, vec![rects[0]]);
        assert_eq!(passes(Vec::new(), &[]).len(), 1);
    }

    /// Whether no two of `placed` (slot and size) overlap.
    fn apart(placed: &[(Slot, (u32, u32))]) -> bool {
        placed.iter().enumerate().all(|(i, (a, (aw, ah)))| {
            placed[i + 1..].iter().all(|(b, (bw, bh))| {
                a.layer != b.layer
                    || a.x + aw <= b.x
                    || b.x + bw <= a.x
                    || a.y + ah <= b.y
                    || b.y + bh <= a.y
            })
        })
    }

    #[test]
    fn shelf_packing_fits() {
        let sizes = [(10, 10), (5, 12), (10, 10)];
        let (slots, shelves) = pack(&sizes, 16, 8);
        assert!(slots.iter().all(|s| s.present()));
        assert!(shelves.used_layers() >= 2);
        let placed: Vec<_> = slots.into_iter().zip(sizes).collect();
        assert!(apart(&placed));
    }

    #[test]
    fn later_images_go_around_the_packed_ones() {
        let sizes = [(10, 10), (5, 12)];
        let (slots, mut shelves) = pack(&sizes, 16, 8);
        let next = shelves.place(6, 4).unwrap();
        let mut placed: Vec<_> = slots.into_iter().zip(sizes).collect();
        placed.push((next, (6, 4)));
        assert!(apart(&placed));
    }

    /// design: render (#105): shelf heights are classes, at most half again
    /// an image's height.
    #[test]
    fn shelf_classes() {
        let classes: Vec<u32> = [1, 8, 9, 12, 13, 16, 17, 24, 25, 33, 49, 100, 4000]
            .iter()
            .map(|&h| shelf_class(h, 4096))
            .collect();
        assert_eq!(
            classes,
            [8, 8, 12, 12, 16, 16, 24, 24, 32, 48, 64, 128, 4096]
        );
        assert_eq!(shelf_class(40, 40), 40, "never taller than a layer");
    }

    /// design: render (#105): a freed place is used again, spans merge, and
    /// an emptied shelf gives its rows back to its layer.
    #[test]
    fn freed_places_are_used_again() {
        let mut s = Shelves::new(32, 1);
        let a = s.place(10, 8).unwrap();
        let b = s.place(10, 8).unwrap();
        let c = s.place(12, 8).unwrap();
        assert_eq!((a.y, b.y, c.y), (0, 0, 0));
        assert!(
            s.place(4, 8).is_some_and(|d| d.y == 8),
            "the first shelf is full"
        );
        s.free(b, 10);
        s.free(a, 10);
        // The merged span of a and b takes a 20-wide image.
        assert_eq!(
            s.place(20, 7),
            Some(Slot {
                x: 0,
                y: 0,
                layer: 0
            })
        );
        let tall = Shelves::new(32, 1).committed();
        assert_eq!(tall, 0);
        let mut t = Shelves::new(32, 1);
        let x = t.place(32, 32).unwrap();
        assert!(t.place(1, 1).is_none(), "full");
        t.free(x, 32);
        assert_eq!(t.committed(), 0, "the emptied shelf is gone");
        assert!(t.place(32, 24).is_some() && t.place(32, 8).is_some());
    }

    /// design: render (#105): images of mixed heights arriving in turn keep
    /// to their own shelves.
    #[test]
    fn mixed_heights_keep_to_their_classes() {
        let mut s = Shelves::new(256, 4);
        let mut placed = Vec::new();
        for _ in 0..64 {
            for size in [(8, 64), (8, 8)] {
                placed.push((s.place(size.0, size.1).unwrap(), size));
            }
        }
        assert!(apart(&placed));
        assert_eq!(s.committed(), (2 * 64 + 2 * 8) * 256);
    }

    /// design: render (#108): an image wider or taller than an atlas layer
    /// gets no slot and fails nothing; the others are placed as before.
    #[test]
    fn an_image_larger_than_a_layer_gets_no_slot() {
        let (slots, mut shelves) = pack(&[(10, 10), (17, 1), (5, 12), (4, 20), (0, 3)], 16, 8);
        assert!(slots[0].present() && slots[2].present());
        assert_eq!(slots[1], Slot::ABSENT);
        assert_eq!(slots[3], Slot::ABSENT);
        assert_eq!(slots[4], Slot::ABSENT, "an empty image takes no place");
        assert_eq!(shelves.place(17, 1), None);
        assert_eq!(shelves.place(1, 17), None);
        let mut full = Shelves::new(16, 1);
        assert!(full.place(16, 16).is_some());
        assert!(full.place(16, 16).is_none());
    }

    /// design: render (#105): an atlas grows by half again, at least two
    /// layers, up to the device's limit.
    #[test]
    fn grown_layers_follow_the_limit() {
        assert_eq!(grown_layers(2, 256), 4);
        assert_eq!(grown_layers(8, 256), 12);
        assert_eq!(grown_layers(250, 256), 256);
        assert_eq!(grown_layers(256, 256), 256);
    }

    /// design: render (#107): a row texture has room for half again as many
    /// rows, at least 64, in columns of at most `side` rows, up to `side ÷
    /// 256` columns.
    #[test]
    fn row_layouts_leave_room() {
        assert_eq!(row_layout(0, 4096), Some((64, 1)));
        assert_eq!(row_layout(800, 4096), Some((1200, 1)));
        assert_eq!(row_layout(3000, 4096), Some((4096, 2)));
        // At the limit of 16 columns, the room is what is left.
        assert_eq!(row_layout(60000, 4096), Some((4096, 16)));
        assert_eq!(row_layout(65536, 4096), Some((4096, 16)));
        assert_eq!(row_layout(65537, 4096), None);
        assert_eq!(row_layout(1, 255), None);
    }

    #[test]
    fn instance_is_80_bytes() {
        assert_eq!(std::mem::size_of::<Instance>(), 80);
    }
}
