//! The map picker of the skirmish setup screen (design: map picker (lead's
//! request 2026-11-13)): a search field, filters and a sort over the maps
//! offered, the list they leave, and the chosen map's preview with its
//! start markers (S-0009 claims 24 and 25).
//!
//! The search, the filters, the sort, the size classes and the layout are
//! our own design. The preview's pixels are the client's (it decodes them
//! and hands this screen an image id); where the markers go over a drawn
//! preview is S-0009's (claim 25 and Formulas, [`marker_corners`]).
//!
//! The picker never changes which maps exist or their order in the
//! install: it gives indices into the maps offered, and the setup keeps
//! such an index as its map.

use crate::draw::{DrawList, Rgba, colour};
use crate::font;
use crate::geom::{Point, Rect};
use crate::setup::{MapEntry, Setup, SetupHit};
use crate::widget::{self, Targets};

/// What the picker knows of a map besides its entry.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MapFacts {
    /// Width and height of the map size entry (S-0009 claim 30).
    pub size: (i32, i32),
    /// The theater's name as shown ("" unknown).
    pub theater: String,
    /// Whether the map has an embedded preview: a valid preview size and a
    /// preview block with packed bytes. A block that turns out not to
    /// expand still shows an image (black where nothing decodes, S-0009
    /// claim 24).
    pub preview: bool,
}

/// The facts of a map the client gave none for (nothing known).
static NO_FACTS: MapFacts = MapFacts {
    size: (0, 0),
    theater: String::new(),
    preview: false,
};

/// A map's size class: the smallest third of the maps offered by area is
/// small, the middle third medium, the rest large; maps of equal area share
/// a class ([`size_classes`]).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SizeClass {
    /// Every size.
    #[default]
    Any,
    /// The smallest third.
    Small,
    /// The middle third.
    Medium,
    /// The largest third.
    Large,
}

/// The preview filter.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PreviewFilter {
    /// Maps with and without a preview.
    #[default]
    Any,
    /// Maps with a preview.
    With,
    /// Maps without one.
    Without,
}

/// The list's order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Sort {
    /// The order the install offers them in.
    #[default]
    Listed,
    /// By name, ignoring case.
    Name,
    /// By start count, then name.
    Players,
    /// By area, then name.
    Size,
}

/// A filter, or the sort, of the picker.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FilterKind {
    /// The start count.
    Players,
    /// The size class.
    Size,
    /// The theater.
    Theater,
    /// Whether the map has a preview.
    Preview,
    /// The sort.
    Sort,
}

/// The longest search typed, in characters.
// design: map picker (lead's request 2026-11-13)
pub const SEARCH_CHARS: usize = 40;

/// List rows shown at once.
// design: map picker (lead's request 2026-11-13)
pub const ROWS: usize = 20;

/// The height of a list row.
// design: map picker (lead's request 2026-11-13)
pub const ROW_H: i32 = 16;

/// The search, the filters and the sort.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MapQuery {
    /// Words every shown map's name holds, ignoring case.
    pub search: String,
    /// The start count shown, 0 for every count.
    pub players: u32,
    /// The size class shown.
    pub size: SizeClass,
    /// The theater shown, "" for every theater.
    pub theater: String,
    /// The preview filter.
    pub preview: PreviewFilter,
    /// The list's order.
    pub sort: Sort,
}

const SIZES: [SizeClass; 4] = [
    SizeClass::Any,
    SizeClass::Small,
    SizeClass::Medium,
    SizeClass::Large,
];
const PREVIEWS: [PreviewFilter; 3] = [
    PreviewFilter::Any,
    PreviewFilter::With,
    PreviewFilter::Without,
];
const SORTS: [Sort; 4] = [Sort::Listed, Sort::Name, Sort::Players, Sort::Size];

impl SizeClass {
    /// The name shown and kept.
    pub fn name(self) -> &'static str {
        match self {
            Self::Any => "Any",
            Self::Small => "Small",
            Self::Medium => "Medium",
            Self::Large => "Large",
        }
    }
}

impl PreviewFilter {
    /// The name shown and kept.
    pub fn name(self) -> &'static str {
        match self {
            Self::Any => "Any",
            Self::With => "With",
            Self::Without => "Without",
        }
    }
}

impl Sort {
    /// The name shown and kept.
    pub fn name(self) -> &'static str {
        match self {
            Self::Listed => "As found",
            Self::Name => "Name",
            Self::Players => "Players",
            Self::Size => "Size",
        }
    }
}

/// Steps `v` by `d` through `opts`, round; a value not offered steps from
/// the first.
fn cycle<T: PartialEq + Clone>(opts: &[T], v: &T, d: i32) -> T {
    let i = opts.iter().position(|o| o == v).unwrap_or(0) as i32;
    opts[(i + d).rem_euclid(opts.len().max(1) as i32) as usize].clone()
}

/// The start counts the maps offer, "every count" (0) first.
pub fn player_options(maps: &[MapEntry]) -> Vec<u32> {
    let mut v: Vec<u32> = maps.iter().map(|m| m.starts).collect();
    v.sort_unstable();
    v.dedup();
    v.insert(0, 0);
    v
}

/// The theaters the maps offer, "every theater" ("") first.
pub fn theater_options(facts: &[MapFacts]) -> Vec<String> {
    let mut v: Vec<String> = facts
        .iter()
        .map(|f| f.theater.clone())
        .filter(|t| !t.is_empty())
        .collect();
    v.sort();
    v.dedup();
    v.insert(0, String::new());
    v
}

/// The size class of each of `n` maps by area ([`SizeClass`]).
pub fn size_classes(n: usize, facts: &[MapFacts]) -> Vec<SizeClass> {
    let area = |i: usize| {
        let (w, h) = facts.get(i).unwrap_or(&NO_FACTS).size;
        i64::from(w.max(0)) * i64::from(h.max(0))
    };
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by_key(|&i| (area(i), i));
    let mut out = vec![SizeClass::Medium; n];
    let mut last: Option<(i64, SizeClass)> = None;
    for (rank, &i) in order.iter().enumerate() {
        let a = area(i);
        let class = match last {
            Some((pa, c)) if pa == a => c,
            _ => match rank * 3 / n.max(1) {
                0 => SizeClass::Small,
                1 => SizeClass::Medium,
                _ => SizeClass::Large,
            },
        };
        out[i] = class;
        last = Some((a, class));
    }
    out
}

impl MapQuery {
    /// Whether nothing is filtered and the order is the install's.
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }

    /// Typed text goes into the search: printable characters only, up to
    /// [`SEARCH_CHARS`].
    pub fn type_text(&mut self, text: &str) {
        for c in text.chars().filter(|c| !c.is_control()) {
            if self.search.chars().count() < SEARCH_CHARS {
                self.search.push(c);
            }
        }
    }

    /// Backspace in the search.
    pub fn backspace(&mut self) {
        self.search.pop();
    }

    /// Steps a filter, or the sort, through what the maps offer.
    pub fn step(&mut self, k: FilterKind, d: i32, maps: &[MapEntry], facts: &[MapFacts]) {
        match k {
            FilterKind::Players => self.players = cycle(&player_options(maps), &self.players, d),
            FilterKind::Size => self.size = cycle(&SIZES, &self.size, d),
            FilterKind::Theater => {
                self.theater = cycle(&theater_options(facts), &self.theater, d);
            }
            FilterKind::Preview => self.preview = cycle(&PREVIEWS, &self.preview, d),
            FilterKind::Sort => self.sort = cycle(&SORTS, &self.sort, d),
        }
    }

    /// Whether map `i` passes the search and the filters.
    fn passes(&self, m: &MapEntry, f: &MapFacts, class: SizeClass) -> bool {
        let name = m.label.to_lowercase();
        let words = self.search.to_lowercase();
        words.split_whitespace().all(|w| name.contains(w))
            && (self.players == 0 || m.starts == self.players)
            && (self.size == SizeClass::Any || class == self.size)
            && (self.theater.is_empty() || f.theater.eq_ignore_ascii_case(&self.theater))
            && match self.preview {
                PreviewFilter::Any => true,
                PreviewFilter::With => f.preview,
                PreviewFilter::Without => !f.preview,
            }
    }

    /// The maps that pass, as indices into `maps`, in the query's order.
    /// `facts` is by the same index; a map without facts has none of the
    /// filtered properties.
    pub fn shown(&self, maps: &[MapEntry], facts: &[MapFacts]) -> Vec<usize> {
        let classes = size_classes(maps.len(), facts);
        let fact = |i: usize| facts.get(i).unwrap_or(&NO_FACTS);
        let mut v: Vec<usize> = (0..maps.len())
            .filter(|&i| self.passes(&maps[i], fact(i), classes[i]))
            .collect();
        let name = |i: usize| maps[i].label.to_lowercase();
        let area = |i: usize| {
            let (w, h) = fact(i).size;
            i64::from(w.max(0)) * i64::from(h.max(0))
        };
        match self.sort {
            Sort::Listed => {}
            Sort::Name => v.sort_by_cached_key(|&i| (name(i), i)),
            Sort::Players => v.sort_by_cached_key(|&i| (maps[i].starts, name(i), i)),
            Sort::Size => v.sort_by_cached_key(|&i| (area(i), name(i), i)),
        }
        v
    }

    /// The query as kept in the settings: `(key, value)` pairs, values
    /// without line breaks.
    pub fn kept(&self) -> Vec<(&'static str, String)> {
        let search: String = self.search.chars().filter(|c| !c.is_control()).collect();
        let theater: String = self.theater.chars().filter(|c| !c.is_control()).collect();
        vec![
            ("search", search),
            ("players", self.players.to_string()),
            ("size", self.size.name().to_owned()),
            ("theater", theater),
            ("preview", self.preview.name().to_owned()),
            ("sort", self.sort.name().to_owned()),
        ]
    }

    /// Reads one kept pair; a key not the query's, or a value that is
    /// wrong, leaves the query as it is. Returns whether the key was the
    /// query's.
    pub fn keep(&mut self, key: &str, value: &str) -> bool {
        let named = |name: &str| value.trim().eq_ignore_ascii_case(name);
        match key {
            "search" => {
                self.search = value
                    .chars()
                    .filter(|c| !c.is_control())
                    .take(SEARCH_CHARS)
                    .collect();
            }
            "players" => {
                if let Ok(n) = value.trim().parse::<u32>() {
                    self.players = n;
                }
            }
            "size" => {
                if let Some(s) = SIZES.into_iter().find(|s| named(s.name())) {
                    self.size = s;
                }
            }
            "theater" => {
                self.theater = value
                    .chars()
                    .filter(|c| !c.is_control())
                    .take(SEARCH_CHARS)
                    .collect::<String>()
                    .trim()
                    .to_owned();
            }
            "preview" => {
                if let Some(p) = PREVIEWS.into_iter().find(|p| named(p.name())) {
                    self.preview = p;
                }
            }
            "sort" => {
                if let Some(s) = SORTS.into_iter().find(|s| named(s.name())) {
                    self.sort = s;
                }
            }
            _ => return false,
        }
        true
    }

    /// The text shown for a filter's or the sort's value.
    pub fn value_label(&self, k: FilterKind) -> String {
        match k {
            FilterKind::Players if self.players == 0 => "Any".into(),
            FilterKind::Players => format!("{} players", self.players),
            FilterKind::Size => self.size.name().into(),
            FilterKind::Theater if self.theater.is_empty() => "Any".into(),
            FilterKind::Theater => self.theater.clone(),
            FilterKind::Preview => self.preview.name().into(),
            FilterKind::Sort => self.sort.name().into(),
        }
    }
}

/// The selection rule: the selected map while it is shown, else the first
/// shown map; `None` when nothing is shown.
pub fn keep_or_first(selected: usize, shown: &[usize]) -> Option<usize> {
    if shown.contains(&selected) {
        Some(selected)
    } else {
        shown.first().copied()
    }
}

/// The preview start markers of a map: its header block's count, origin,
/// extent and pairs (S-0009 claim 25), as the client read them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Marks {
    /// The marker count N_h.
    pub count: i32,
    /// The origin (X0, Y0).
    pub origin: (i32, i32),
    /// The extent (Wh, Hh).
    pub extent: (i32, i32),
    /// Pairs 1 … min(N_h, 8).
    pub pairs: Vec<(i32, i32)>,
}

/// The marker scale: a pair's displacement is taken in per-mille of the
/// extent (S-0009 Formulas, preview marker position).
const MARKER_SCALE: i32 = 1000; // spec: S-0009 Constants (preview marker scale)
/// The marker corner shift (S-0009 Formulas).
const MARKER_SHIFT: (i32, i32) = (9, 6); // spec: S-0009 Constants (preview marker corner shift)
/// Markers are drawn only for 1 ≤ N_h ≤ 8 (S-0009 claim 25).
const MARKER_MOST: i32 = 8; // spec: S-0009 claim 25 (markers drawn only when 1 ≤ N_h ≤ 8)

/// One coordinate of a marker: S-0009 Formulas, preview marker position.
fn marker_axis(v: i32, origin: i32, extent: i32, drawn: i32, start: i32, shift: i32) -> i32 {
    // spec: S-0009 Formulas (signed 32-bit values; every difference, product and sum wraps; each quotient truncates toward zero; −2147483648 ÷ −1 wraps, S-0205 claim 3)
    let inner = v
        .wrapping_sub(origin)
        .wrapping_mul(MARKER_SCALE)
        .wrapping_div(extent);
    inner
        .wrapping_mul(drawn)
        .wrapping_div(MARKER_SCALE)
        .wrapping_add(start)
        .wrapping_sub(shift)
}

/// Where the marker graphics' top-left corners go over a preview drawn at
/// `drawn` (S-0009 claim 25 and Formulas), marker 1 first. None unless 1 ≤
/// N_h ≤ 8; none with an extent of 0 (the original's crash, which our
/// engine replaces by drawing no markers).
pub fn marker_corners(m: &Marks, drawn: Rect) -> Vec<(i32, i32)> {
    if !(1..=MARKER_MOST).contains(&m.count) {
        return Vec::new();
    }
    // spec: S-0009 claim 25 and Edge cases (an extent of 0: our engine draws no preview markers)
    if m.extent.0 == 0 || m.extent.1 == 0 {
        return Vec::new();
    }
    m.pairs
        .iter()
        .take(m.count as usize)
        .map(|&(x, y)| {
            (
                marker_axis(x, m.origin.0, m.extent.0, drawn.w, drawn.x, MARKER_SHIFT.0),
                marker_axis(y, m.origin.1, m.extent.1, drawn.h, drawn.y, MARKER_SHIFT.1),
            )
        })
        .collect()
}

/// The marker graphic's size: twice the corner shift, so the pair's place
/// is its centre.
// design: map picker (lead's request 2026-11-13)
const MARKER_SIZE: (i32, i32) = (18, 12);

/// What the client has of the chosen map's preview.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreviewShow {
    /// No preview to show: a placeholder.
    None,
    /// A preview being decoded.
    Pending,
    /// The decoded preview: the client's image `id` of `size` pixels.
    Image {
        /// The image id.
        id: u32,
        /// Its width and height.
        size: (i32, i32),
    },
}

/// The chosen map's preview as the screens show it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shown<'a> {
    /// The preview.
    pub preview: PreviewShow,
    /// Its markers, when the client draws them.
    pub marks: Option<&'a Marks>,
}

impl Shown<'_> {
    /// Nothing to show.
    pub const NONE: Shown<'static> = Shown {
        preview: PreviewShow::None,
        marks: None,
    };
}

/// The rectangle a `size` image takes fitted inside `inner`, keeping its
/// proportions, centred.
// design: map picker (lead's request 2026-11-13)
pub fn fit_image(size: (i32, i32), inner: Rect) -> Rect {
    let (pw, ph) = (i64::from(size.0.max(1)), i64::from(size.1.max(1)));
    let (iw, ih) = (i64::from(inner.w.max(0)), i64::from(inner.h.max(0)));
    let (w, h) = if pw * ih <= ph * iw {
        ((pw * ih / ph).max(1), ih)
    } else {
        (iw, (ph * iw / pw).max(1))
    };
    let (w, h) = (w as i32, h as i32);
    Rect::new(
        inner.x + (inner.w - w) / 2,
        inner.y + (inner.h - h) / 2,
        w,
        h,
    )
}

/// The marker graphic's ground (our own look).
// design: map picker (lead's request 2026-11-13)
const MARKER_GROUND: Rgba = [0, 0, 0, 200];

/// Draws a preview box: the preview fitted inside with its markers, or a
/// placeholder (design: map picker (lead's request 2026-11-13)).
pub fn draw_preview(d: &mut DrawList, r: Rect, shown: &Shown<'_>, empty: &str) {
    d.fill(r, colour::BACKDROP);
    d.frame(r, colour::EDGE);
    match shown.preview {
        PreviewShow::None => d.text_centred(r, empty, colour::DIM),
        PreviewShow::Pending => d.text_centred(r, "...", colour::DIM),
        PreviewShow::Image { id, size } => {
            let drawn = fit_image(size, r.inset(2));
            d.image(id, drawn);
            let Some(m) = shown.marks else {
                return;
            };
            for (i, (x, y)) in marker_corners(m, drawn).into_iter().enumerate() {
                let box_r = Rect::new(x, y, MARKER_SIZE.0, MARKER_SIZE.1).intersect(&r);
                if box_r.is_empty() {
                    continue;
                }
                d.fill(box_r, MARKER_GROUND);
                d.frame(box_r, colour::ACCENT);
                d.text_centred(box_r, &(i + 1).to_string(), colour::ACCENT);
            }
        }
    }
}

/// The picker's state, kept by the shell while the setup screen lives.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MapPicker {
    /// Whether the picker is open over the setup screen.
    pub open: bool,
    /// The search, filters and sort (kept in the settings).
    pub query: MapQuery,
    /// The first list row shown.
    pub scroll: usize,
    /// Whether typing goes to the search.
    pub typing: bool,
}

/// The filters' row: kind, label, left offset and width.
const FILTERS: [(FilterKind, &str, i32, i32); 4] = [
    (FilterKind::Players, "Players", 0, 148),
    (FilterKind::Size, "Size", 152, 148),
    (FilterKind::Theater, "Theater", 304, 148),
    (FilterKind::Preview, "Preview", 456, 144),
];

/// The sort choice: label, left offset and width.
const SORT_AT: (&str, i32, i32) = ("Sort", 420, 180);

impl MapPicker {
    /// The scroll that keeps list position `at` of `len` rows in view.
    pub fn reveal(&mut self, at: usize, len: usize) {
        if at < self.scroll {
            self.scroll = at;
        } else if at >= self.scroll + ROWS {
            self.scroll = at + 1 - ROWS;
        }
        self.scroll = self.scroll.min(len.saturating_sub(ROWS));
    }

    /// Scrolls the list by `d` rows within `len` rows.
    pub fn scroll_by(&mut self, d: i32, len: usize) {
        let s = (self.scroll as i64 + i64::from(d)).max(0) as usize;
        self.scroll = s.min(len.saturating_sub(ROWS));
    }

    /// Draws the picker over a window of `size` and lays out its targets.
    /// `shown` is the query's list ([`MapQuery::shown`]), `selected` the
    /// setup's map, `preview` the selected map's preview.
    #[allow(clippy::too_many_arguments)]
    pub fn view(
        &self,
        size: (i32, i32),
        maps: &[MapEntry],
        facts: &[MapFacts],
        selected: usize,
        shown: &[usize],
        preview: &Shown<'_>,
        hover: Option<Point>,
    ) -> (DrawList, Targets<SetupHit>) {
        let mut d = DrawList::new();
        let mut t = Targets::new();
        let (w, h) = size;
        d.fill(Rect::new(0, 0, w, h), colour::BACKDROP);
        let outer = Setup::panel_rect(size);
        widget::panel(&mut d, outer, Some("CHOOSE A MAP"));
        let hot = |r: Rect| hover.is_some_and(|p| r.contains(p));
        let x0 = outer.x + 10;
        let y0 = outer.y + 24;
        // The search, its clear button and the sort.
        d.text_left(Rect::new(x0, y0, 44, 16), "Search", colour::DIM);
        let field = Rect::new(x0 + 46, y0, 296, 16);
        let q = &self.query;
        let b = widget::search_field(
            &mut d,
            field,
            &q.search,
            "Type part of a name",
            self.typing,
            hot(field),
        );
        t.push(b, SetupHit::SearchField);
        let clear = Rect::new(x0 + 348, y0, 60, 16);
        let can_clear = !q.search.is_empty();
        widget::button(&mut d, clear, "Clear", hot(clear), can_clear);
        if can_clear {
            t.push(clear, SetupHit::ClearSearch);
        }
        let (label, sx, sw) = SORT_AT;
        let r = Rect::new(x0 + sx, y0, sw, 16);
        let (a, bb) = widget::choice(&mut d, r, label, &q.value_label(FilterKind::Sort), true);
        t.push_choice(
            a,
            SetupHit::Filter(FilterKind::Sort, -1),
            bb,
            SetupHit::Filter(FilterKind::Sort, 1),
        );
        // The filters.
        let y1 = y0 + 22;
        for (k, label, fx, fw) in FILTERS {
            let r = Rect::new(x0 + fx, y1, fw, 16);
            let (a, bb) = widget::choice(&mut d, r, label, &q.value_label(k), true);
            t.push_choice(a, SetupHit::Filter(k, -1), bb, SetupHit::Filter(k, 1));
        }
        // The list, with its scroll bar.
        let y2 = y1 + 24;
        let list = Rect::new(x0, y2, 300, ROWS as i32 * ROW_H);
        d.fill(list, colour::BACKDROP);
        d.frame(list, colour::EDGE);
        let rows = Rect::new(list.x + 1, list.y, list.w - 12, list.h);
        let scroll = self.scroll.min(shown.len().saturating_sub(ROWS));
        for (row, &i) in shown.iter().enumerate().skip(scroll).take(ROWS) {
            let Some(m) = maps.get(i) else {
                continue;
            };
            let r = Rect::new(
                rows.x,
                rows.y + (row - scroll) as i32 * ROW_H,
                rows.w,
                ROW_H,
            );
            let chosen = i == selected;
            if chosen {
                d.fill(r, colour::HOT);
                d.frame(r, colour::ACCENT);
            } else if hot(r) {
                d.fill(r, colour::RAISED);
            }
            let count = format!("{} players", m.starts);
            let cw = font::text_width(&count) + 6;
            let ink = if chosen { colour::ACCENT } else { colour::TEXT };
            d.text_left(
                Rect::new(r.x + 2, r.y, r.w - cw - 4, r.h),
                &font::fit(&m.label, r.w - cw - 10),
                ink,
            );
            d.text_left(Rect::new(r.right() - cw, r.y, cw, r.h), &count, colour::DIM);
            t.push_hit(r, SetupHit::PickMap(i));
        }
        if shown.is_empty() {
            d.text_centred(
                Rect::new(rows.x, rows.y + 4, rows.w, ROW_H),
                "No map matches",
                colour::DIM,
            );
        }
        let bar = Rect::new(list.right() - 11, list.y + 1, 10, list.h - 2);
        d.fill(bar, colour::PANEL);
        if shown.len() > ROWS {
            let n = shown.len() as i64;
            let th = (i64::from(bar.h) * ROWS as i64 / n).max(8) as i32;
            let span = i64::from(bar.h - th);
            let most = (n - ROWS as i64).max(1);
            let ty = bar.y + (span * scroll as i64 / most) as i32;
            let thumb = Rect::new(bar.x, ty, bar.w, th);
            d.fill(thumb, colour::EDGE);
            let above = Rect::new(bar.x, bar.y, bar.w, ty - bar.y);
            let below = Rect::new(bar.x, thumb.bottom(), bar.w, bar.bottom() - thumb.bottom());
            if !above.is_empty() {
                t.push_hit(above, SetupHit::ScrollMaps(-(ROWS as i32)));
            }
            if !below.is_empty() {
                t.push_hit(below, SetupHit::ScrollMaps(ROWS as i32));
            }
        }
        // The chosen map: its preview and what is known of it.
        let px = x0 + 310;
        let pbox = Rect::new(px, y2, 290, 200);
        let chosen = shown
            .contains(&selected)
            .then(|| maps.get(selected))
            .flatten();
        let blank = Shown::NONE;
        let empty = if chosen.is_some() {
            "No preview"
        } else {
            "No map chosen"
        };
        draw_preview(
            &mut d,
            pbox,
            if chosen.is_some() { preview } else { &blank },
            empty,
        );
        let mut ly = pbox.bottom() + 8;
        if let Some(m) = chosen {
            let f = facts.get(selected).unwrap_or(&NO_FACTS);
            let mut line = |d: &mut DrawList, text: &str, ink: Rgba| {
                d.text_left(Rect::new(px, ly, 290, 14), &font::fit(text, 286), ink);
                ly += 15;
            };
            line(&mut d, &m.label, colour::ACCENT);
            line(
                &mut d,
                &format!("{} start positions", m.starts),
                colour::TEXT,
            );
            if f.size != (0, 0) {
                line(
                    &mut d,
                    &format!("Size {} x {}", f.size.0, f.size.1),
                    colour::TEXT,
                );
            }
            if !f.theater.is_empty() {
                line(&mut d, &format!("Theater: {}", f.theater), colour::TEXT);
            }
            if !f.preview {
                line(&mut d, "No embedded preview", colour::DIM);
            }
        }
        // The count, and Done.
        let by = outer.bottom() - 30;
        let (status, ink) = if shown.is_empty() {
            (
                "No map matches the search and filters".to_owned(),
                colour::BAD,
            )
        } else {
            (
                format!("{} of {} maps", shown.len(), maps.len()),
                colour::TEXT,
            )
        };
        d.text_left(Rect::new(x0, by, 360, 20), &font::fit(&status, 360), ink);
        let done = Rect::new(outer.right() - 120, by, 110, 20);
        widget::button(&mut d, done, "Done", hot(done), true);
        t.push(done, SetupHit::MapsDone);
        (d, t)
    }
}

#[cfg(test)]
#[path = "mappick_tests.rs"]
mod tests;
