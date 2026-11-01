//! The in-game sidebar and its overlays (our own layout): radar, credits,
//! power, the mode buttons, the four tabs of S-0411 claim 1 and the build
//! icons of the shown tab.

use glasnost_sim::econ::TypeRef;
use glasnost_sim::econ::production;
use glasnost_sim::econ::sidebar;
use glasnost_sim::world::{HouseId, World};

use crate::draw::{DrawList, colour};
use crate::font;
use crate::geom::{Point, Rect};
use crate::modes::Mode;
use crate::widget::{self, Targets};

/// The sidebar's width in pixels (our layout).
pub const SIDEBAR_W: i32 = 168;
/// The radar's size (S-0410 Constants: 140 × 108).
pub const RADAR: (i32, i32) = (140, 108); // spec: S-0410 Constants (radar size)
/// A build icon's cell in the grid (our layout; the icon's art is drawn
/// centred in it and clipped).
pub const CAMEO: (i32, i32) = (68, 52);
/// The number of sidebar tabs (S-0411 claim 1).
pub const TABS: usize = 4; // spec: S-0411 claim 1 (four tabs)

/// What a sidebar icon stands for: a build type, or a superweapon record
/// of the house (S-0004 claim 11: a superweapon icon is an entry of a tab).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Entry {
    /// A build type.
    Type(TypeRef),
    /// A superweapon type, by its position in the type list.
    Superweapon(u32),
}

/// The tab of the superweapon icons (S-0004 claim 11: the second tab).
pub const SW_TAB: usize = 1; // spec: S-0004 claim 11 (superweapons go to the 2nd tab)

/// The house's granted superweapon records in icon order (S-0512 claim 16:
/// the sidebar shows each granted record; S-0004 claim 11 rule 1:
/// superweapons sort by recharge time in ticks, shortest first). Display
/// names compare equal, as in the simulation's own icon sort (a seam of
/// rule 6), so equal times keep the newest grant first.
pub fn superweapons(w: &World, house: HouseId) -> Vec<u32> {
    let Some(hs) = w.sw.houses.get(usize::from(house.0)) else {
        return Vec::new();
    };
    let types = &w.rules().superweapons.types;
    let newest_first = w.rules().quirks.sidebar_tie_newest_first;
    let mut v: Vec<(i32, i64, u32)> = hs
        .records
        .iter()
        .enumerate()
        .filter(|(i, r)| r.granted && *i < types.len())
        .map(|(i, r)| {
            let seq = r.seq as i64;
            // spec: S-0004 claim 11 rule 6 (equal names: the newly added item first; quirks.sidebar_tie_newest_first)
            let seq = if newest_first { -seq } else { seq };
            // spec: S-0004 claim 11 rule 1 (two superweapons sort by recharge time in ticks, shortest first)
            (types[i].charge, seq, i as u32)
        })
        .collect();
    v.sort();
    v.into_iter().map(|(_, _, i)| i).collect()
}

/// The icons of tab `tab` of `house`: on the superweapon tab its
/// superweapons first (S-0004 claim 11 rule 1: a superweapon sorts before
/// anything that is not one), then the build icons in the house's order.
pub fn entries(w: &World, house: HouseId, tab: usize) -> Vec<Entry> {
    let mut v: Vec<Entry> = if tab == SW_TAB {
        superweapons(w, house)
            .into_iter()
            .map(Entry::Superweapon)
            .collect()
    } else {
        Vec::new()
    };
    if let Some(icons) = w
        .houses
        .get(usize::from(house.0))
        .and_then(|h| h.econ.tech.icons.get(tab))
    {
        v.extend(icons.iter().map(|c| Entry::Type(c.t)));
    }
    v
}

/// One build icon as the sidebar draws it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CameoView {
    /// What the icon stands for.
    pub entry: Entry,
    /// The type's name.
    pub name: String,
    /// Its price to the house (S-0404).
    pub price: i32,
    /// Drawn darkened (S-0411 claim 3).
    pub darkened: bool,
    /// The clock stage 1 … 55 while the line's item is of this type
    /// (S-0411 claim 8).
    pub stage: Option<i32>,
    /// The *ready* label (S-0411 claim 8).
    pub ready: bool,
    /// The *on hold* label (S-0411 claim 8).
    pub on_hold: bool,
    /// The queue count (S-0411 claim 7).
    pub queued: Option<i32>,
    /// The icon's art: sprite id, frame and size, when the player's files
    /// have it (S-0007 claims 22 and 25).
    pub art: Option<(u32, u32, i32, i32)>,
    /// A superweapon's charging progress, its clock stage 0 … 53 of 54,
    /// shown while it is not ready (S-0512 claim 16, F3).
    pub charge: Option<i32>,
    /// A drain-mode superweapon's *charging* or *active* label (S-0512
    /// claim 16).
    pub state: Option<&'static str>,
    /// A superweapon's time left in game seconds, *s* of S-0512 F6, while
    /// it is not ready (for the tooltip).
    pub seconds: Option<i32>,
}

/// One tab button.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TabView {
    /// The number of icons in the tab.
    pub count: usize,
    /// The tab holds a done building or defence waiting for placement
    /// (S-0411 claim 11).
    pub waiting: bool,
    /// The tick whose sidebar update started the tab's current flash
    /// (S-0411 claim 11); kept by the client.
    pub flash_start: Option<i32>,
    /// The tab holds a ready superweapon in its flash window (S-0411
    /// claim 11, S-0512 claim 17).
    pub sw_ready: bool,
}

impl TabView {
    /// Whether the tab has a ready subject that makes it flash (S-0411
    /// claim 11).
    pub fn flashes(&self) -> bool {
        self.waiting || self.sw_ready
    }
}

/// Whether a tab whose flash started at tick `t` shows lit at tick `u`
/// (S-0411 claim 11 and Formulas). `extra` is
/// `quirks.tab_flash_initial_extra_period`.
pub fn tab_lit(t: i32, u: i32, extra: bool) -> bool {
    // spec: S-0411 Formulas (each floor division rounds down in the non-negative domain)
    let period = |x: i32| x.div_euclid(10);
    let r = t.rem_euclid(10);
    let (first_toggle, initial) = if extra {
        // spec: S-0411 claim 11 (initial lit iff ⌊t/10⌋ + 1 even; first toggle visible in tick t + 19 − r)
        (t + 19 - r, (period(t) + 1) % 2 == 0)
    } else {
        // spec: S-0411 Edge cases (quirks.tab_flash_initial_extra_period off: initial lit iff ⌊t/10⌋ even; toggle in tick t + 9 − r)
        (t + 9 - r, period(t) % 2 == 0)
    };
    if u < first_toggle {
        initial
    } else {
        // spec: S-0411 claim 11 (after the first toggle, tick u is lit iff ⌊(u + 1)/10⌋ is even)
        period(u + 1) % 2 == 0
    }
}

/// The power bar's display capacity in bands, one per pixel inside its
/// frame (S-0405 F4's *N*, a design parameter).
pub const METER_BANDS: i32 = RADAR.0 - 2; // spec: S-0405 F4 (N > 1, the chosen display capacity: our bar's inner width)

/// The power meter's part extents in whole bands (S-0405 claim 19, F4),
/// each truncated; the quantisation is our design.
pub fn power_meter(w: &World, house: HouseId) -> [i32; 3] {
    use glasnost_sim::econ::power;
    let (o, d) = power::totals(w, house);
    let b = power::meter_base(w, house);
    let (_, ext) = power::meter(b, o, d, METER_BANDS, w.rules().mode);
    ext.map(|e| e.to_i32_common())
}

/// What the sidebar shows.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HudModel {
    /// The credits the display shows (S-0416).
    pub credits: i32,
    /// Power output and drain (S-0405 claim 4).
    pub power: (i32, i32),
    /// The power meter's supplied, near-shortage and further-shortage
    /// extents in pixels of the bar (S-0405 claim 19, F4).
    pub meter: [i32; 3],
    /// The four tabs.
    pub tabs: [TabView; TABS],
    /// The shown tab.
    pub shown: usize,
    /// The icons of the shown tab, in icon order.
    pub cameos: Vec<CameoView>,
    /// The first grid row shown.
    pub scroll: usize,
    /// The active mode, if any.
    pub mode: Option<Mode>,
    /// Whether the radar is available (S-0410).
    pub radar: bool,
    /// The tick number, for the tab flash (S-0411 claim 11).
    pub tick: i32,
    /// `quirks.tab_flash_initial_extra_period` (S-0411 claim 11).
    pub flash_extra: bool,
    /// The battlefield view's outline on the radar (S-0410 claim 11).
    pub view_box: Option<ViewBox>,
    /// The icon the keyboard focus is on, whose tooltip shows when the
    /// pointer is on no icon.
    pub focus_cameo: Option<usize>,
    /// The install's button images the client draws (S-0411 claim 19).
    pub buttons: ButtonArt,
    /// The scroll button (0 up, 1 down) whose press is in progress with the
    /// pointer over it (S-0411 claim 19).
    pub scroll_pressed: Option<usize>,
}

/// One button's image as the client registered it (S-0411 claim 19).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ButtonImage {
    /// The sprite id; frame *f* of the file is the sprite's frame *f*.
    pub id: u32,
    /// The file's canvas size, which is the button's cell (our layout).
    pub size: (i32, i32),
    /// Bit *f* set when frame *f* was read and registered.
    pub frames: u32,
}

impl ButtonImage {
    /// Whether frame `f` can be drawn.
    pub fn has(&self, f: u32) -> bool {
        f < 32 && self.frames & (1 << f) != 0
    }
}

/// The install's images of the sidebar buttons (S-0411 claim 19). A
/// button without one draws our own symbol (claim 19: a file that cannot
/// be resolved).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ButtonArt {
    /// The sell button's image.
    pub sell: Option<ButtonImage>,
    /// The repair button's image.
    pub repair: Option<ButtonImage>,
    /// Each tab's image, in claim 1's tab order.
    pub tabs: [Option<ButtonImage>; TABS],
    /// The scroll-up and scroll-down images.
    pub scroll: [Option<ButtonImage>; 2],
}

impl ButtonArt {
    /// The image of mode button `m` (none for the power button: claim 19,
    /// no install image is imported for it).
    pub fn mode(&self, m: Mode) -> Option<ButtonImage> {
        match m {
            Mode::Sell => self.sell,
            Mode::Repair => self.repair,
            // spec: S-0411 claim 19 (Power-mode button: our design, no install image is imported for it)
            Mode::Power => None,
        }
    }
}

/// A tab button's frame (S-0411 claim 19, first table: the first row that
/// matches, top to bottom). `lit` is claim 11's flash state, `shown`
/// whether the sidebar displays this tab's icons.
pub fn tab_frame(unavailable: bool, lit: bool, shown: bool) -> u32 {
    if unavailable {
        // spec: S-0411 claim 19 (a tab with no icons, unavailable: frame 2, whatever its flash state)
        2
    } else if lit && shown {
        // spec: S-0411 claim 19 (a tab lit, claim 11, and shown: frame 4)
        4
    } else if lit {
        // spec: S-0411 claim 19 (a tab lit: frame 3)
        3
    } else if shown {
        // spec: S-0411 claim 19 (a tab shown: frame 1)
        1
    } else {
        // spec: S-0411 claim 19 (a tab in any other state: frame 0)
        0
    }
}

/// A sell or repair button's frame (S-0411 claim 19, *Mode lighting*: our
/// engine shows frame 1 in every frame drawn while the mode is active,
/// however it was entered).
pub fn mode_frame(active: bool) -> u32 {
    // spec: S-0411 claim 19 (sell, repair: its mode active frame 1, any other state frame 0; Mode lighting: our engine's rows, however the mode was entered)
    u32::from(active)
}

/// A scroll button's frame (S-0411 claim 19, first table). `pressed`: its
/// press is in progress with the pointer over it.
pub fn scroll_frame(disabled: bool, pressed: bool) -> u32 {
    if disabled {
        // spec: S-0411 claim 19 (a disabled scroll button, claim 18: frame 2, even while pressed)
        2
    } else if pressed {
        // spec: S-0411 claim 19 (its press in progress, with the pointer over it: frame 1)
        1
    } else {
        // spec: S-0411 claim 19 (any other state: frame 0; a press with the pointer off it shows frame 0 in our engine, design)
        0
    }
}

/// The outline that marks the battlefield view's part of the map on the
/// radar (S-0410 claim 11), in radar map pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ViewBox {
    /// The radar image (S-0410 F2): the outline is clipped to it.
    pub image: Rect,
    /// The view's left, top, right and bottom edges, unclipped.
    pub edges: (i32, i32, i32, i32),
}

/// The outline's colour (our look).
// design: S-0410 claim 11 leaves the outline's look to the interface; ours is a 1-pixel white frame
pub const VIEW_BOX_INK: [u8; 4] = [255, 255, 255, 255];

/// The segments of the view outline that lie inside the radar image
/// (S-0410 claim 11), offset by `at` (the radar map's top-left on the
/// screen): an edge outside the image is left out, and the others are cut
/// at the image's border, so that a view reaching past the map's edge
/// shows only the edges that are on the map.
pub fn view_box_lines(b: ViewBox, at: Point) -> Vec<((i32, i32), (i32, i32))> {
    let (l, t, r, bt) = b.edges;
    let im = b.image;
    let (x0, y0, x1, y1) = (im.x, im.y, im.right() - 1, im.bottom() - 1);
    let mut out = Vec::new();
    let (cl, cr) = (l.max(x0), r.min(x1));
    let (ct, cb) = (t.max(y0), bt.min(y1));
    if cl > cr || ct > cb {
        return out;
    }
    let mut seg = |a: (i32, i32), c: (i32, i32)| {
        out.push(((a.0 + at.x, a.1 + at.y), (c.0 + at.x, c.1 + at.y)));
    };
    if (y0..=y1).contains(&t) {
        seg((cl, t), (cr, t));
    }
    if (y0..=y1).contains(&bt) {
        seg((cl, bt), (cr, bt));
    }
    if (x0..=x1).contains(&l) {
        seg((l, ct), (l, cb));
    }
    if (x0..=x1).contains(&r) {
        seg((r, ct), (r, cb));
    }
    out
}

/// The tab to show (S-0411 claim 2): the current one while it has icons,
/// otherwise the first tab with icons.
pub fn shown_tab(w: &World, house: HouseId, current: usize) -> usize {
    if !entries(w, house, current).is_empty() {
        return current;
    }
    // spec: S-0411 claim 2 (the shown tab without icons: switch to the first tab with icons)
    (0..TABS)
        .find(|&i| !entries(w, house, i).is_empty())
        .unwrap_or(current)
}

/// Reads the sidebar of `house` from the world. `credits` is the counter's
/// shown value (S-0416); `name` and `art` give each type's name and icon.
pub fn model(
    w: &World,
    house: HouseId,
    shown: usize,
    credits: i32,
    name: &dyn Fn(TypeRef) -> String,
    art: &dyn Fn(Entry) -> Option<(u32, u32, i32, i32)>,
) -> HudModel {
    let mut m = HudModel {
        credits,
        shown: shown_tab(w, house, shown),
        tick: w.tick,
        flash_extra: w.rules().quirks.tab_flash_initial_extra_period,
        ..HudModel::default()
    };
    let Some(h) = w.houses.get(usize::from(house.0)) else {
        return m;
    };
    m.power = (h.econ.power.output, h.econ.power.drain);
    m.meter = power_meter(w, house);
    m.radar = glasnost_sim::econ::radar::available(w, house);
    use glasnost_sim::superweapons::charge::{self, Label};
    for i in 0..TABS {
        let es = entries(w, house, i);
        m.tabs[i].count = es.len();
        // spec: S-0411 claim 11 (a done building or defence waiting for placement flashes its tab; tabs 1 and 2 feed those lines, claim 1)
        m.tabs[i].waiting = i < 2
            && es.iter().any(|e| {
                matches!(e, Entry::Type(t) if sidebar::icon_state(w, house, *t).is_some_and(|s| s.ready))
            });
        // spec: S-0411 claim 11 and S-0512 claim 17 (a ready superweapon inside its flash window flashes its tab)
        m.tabs[i].sw_ready = es
            .iter()
            .any(|e| matches!(e, Entry::Superweapon(k) if charge::flashing(w, house, *k)));
    }
    let types = &w.rules().superweapons.types;
    m.cameos = entries(w, house, m.shown)
        .into_iter()
        .map(|e| match e {
            Entry::Type(t) => {
                let state = sidebar::icon_state(w, house, t);
                CameoView {
                    entry: e,
                    name: name(t),
                    price: production::price(w, house, t),
                    darkened: sidebar::darkened(w, house, t),
                    stage: state.map(|s| s.stage),
                    ready: state.is_some_and(|s| s.ready),
                    on_hold: state.is_some_and(|s| s.on_hold),
                    queued: sidebar::queue_count(w, house, t),
                    art: art(e),
                    charge: None,
                    state: None,
                    seconds: None,
                }
            }
            Entry::Superweapon(k) => {
                let label = charge::label(w, house, k);
                let ty = types.get(k as usize);
                let ready = charge::get(w, house, k).is_some_and(|r| r.ready);
                let drain = ty.is_some_and(|t| t.drain);
                CameoView {
                    entry: e,
                    name: ty.map_or_else(String::new, |t| t.name.clone()),
                    price: 0,
                    darkened: false,
                    stage: None,
                    ready: label == Label::Ready,
                    on_hold: label == Label::OnHold,
                    queued: None,
                    art: art(e),
                    // spec: S-0512 claim 16 (while not ready, and always for drain-mode types, the icon shows the charging progress)
                    charge: (!ready || drain).then(|| charge::stage(w, house, k)),
                    state: match label {
                        Label::Charging => Some("Charging"),
                        Label::Active => Some("Active"),
                        _ => None,
                    },
                    // spec: S-0512 F6 (s = ⌊remaining / 15⌋ game seconds)
                    seconds: charge::get(w, house, k)
                        .filter(|r| !r.ready)
                        .map(|r| r.remaining(w.tick).div_euclid(15)),
                }
            }
        })
        .collect();
    m
}

/// One superweapon countdown row as the battlefield shows it (S-0512
/// claim 18).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CountdownRow {
    /// The superweapon's name.
    pub name: String,
    /// Its house's colour.
    pub colour: [u8; 3],
    /// The time left in game seconds, *s* of S-0512 F6.
    pub seconds: i32,
}

/// The time text of a countdown row (S-0512 claim 18): hours, minutes and
/// seconds when *s* is an hour or more, otherwise minutes and seconds.
pub fn countdown_time(s: i32) -> String {
    let s = s.max(0);
    // spec: S-0512 claim 18 (as hours, minutes and seconds when s is an hour or more, as minutes and seconds otherwise)
    if s >= 3600 {
        format!("{}:{:02}:{:02}", s / 3600, (s / 60) % 60, s % 60)
    } else {
        format!("{}:{:02}", s / 60, s % 60)
    }
}

/// Whether a countdown time at *s* = 0 shows at host time `clock_ms`
/// (S-0512 claim 18: it blinks, changing state once per second of real
/// time; shown in the even seconds, our phase).
pub fn countdown_blink_on(clock_ms: i64) -> bool {
    // spec: S-0512 claim 18 (at s = 0 the time text blinks, changing state once per second of real time)
    clock_ms.div_euclid(1000) % 2 == 0
}

/// Draws the countdown rows in the battlefield `map` (S-0512 claim 18:
/// every row, in grant order, the name in its house's colour), at host
/// time `clock_ms` for the blink.
///
/// design: S-0512 claim 18 leaves position, font and blink look to us: the
/// rows stand right-aligned at the battlefield's top-right corner, away
/// from the status lines at its top-left, one row a line on a dark band,
/// in our interface font; a blinking time is left out in its off state.
pub fn countdown_view(rows: &[CountdownRow], map: Rect, clock_ms: i64) -> DrawList {
    let mut d = DrawList::new();
    let mut y = map.y + 4;
    for r in rows {
        let time = countdown_time(r.seconds);
        let name = format!("{} ", r.name);
        let (nw, tw) = (font::text_width(&name), font::text_width(&time));
        let band = Rect::new(
            map.right() - nw - tw - 10,
            y,
            nw + tw + 6,
            font::LINE_HEIGHT + 2,
        );
        d.fill(band, [0, 0, 0, 150]);
        let [cr, cg, cb] = r.colour;
        d.text(band.x + 3, y + 2, name, [cr, cg, cb, 255], band);
        // spec: S-0512 claim 18 (at s = 0 the time text blinks; its name stays in the house colour)
        if r.seconds > 0 || countdown_blink_on(clock_ms) {
            d.text(band.x + 3 + nw, y + 2, time, colour::TEXT, band);
        }
        y += font::LINE_HEIGHT + 3;
    }
    d
}

/// A target of the game screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HudHit {
    /// The battlefield.
    Map,
    /// The radar (the client maps the pixel through the radar).
    Radar,
    /// A tab button.
    Tab(usize),
    /// The n-th icon of the shown tab.
    Cameo(usize),
    /// Scroll the icon grid by rows.
    Scroll(i32),
    /// A mode button.
    Mode(Mode),
    /// The options menu (pause).
    Options,
}

/// Our cell of a mode button without an image (our layout).
const MODE_CELL: (i32, i32) = (44, 18);
/// Our cell of a tab button without an image (our layout).
const TAB_CELL: (i32, i32) = (32, 18);
/// Our cell of a scroll button without an image (our layout).
const SCROLL_CELL: (i32, i32) = (68, 18);
/// The space between the buttons of a row, and below a row (our layout).
const BUTTON_GAP: i32 = 4;
/// The narrowest power button beside the sell and repair images: our
/// symbol and a 2-pixel margin each side (our layout).
const POWER_MIN_W: i32 = SYMBOL.0 + 4;

/// Cells of `sizes` side by side from the top `y`, `BUTTON_GAP` apart,
/// as many to a line as fit in `bar` and the rest on the lines below; each
/// line centred in `bar` and each cell centred in its line's height. A
/// cell wider than the bar is narrowed to it. Returns the cells and the
/// top of the next row.
fn row(bar: Rect, y: i32, sizes: &[(i32, i32)]) -> (Vec<Rect>, i32) {
    let mut out = Vec::with_capacity(sizes.len());
    let mut y = y;
    let mut rest = sizes;
    while !rest.is_empty() {
        let mut n = 1;
        let mut total = rest[0].0.min(bar.w);
        while let Some(s) = rest.get(n)
            && total + BUTTON_GAP + s.0 <= bar.w
        {
            total += BUTTON_GAP + s.0;
            n += 1;
        }
        let (line, next) = rest.split_at(n);
        let height = line.iter().map(|s| s.1).max().unwrap_or(0);
        let mut x = bar.x + (bar.w - total) / 2;
        for &(w, h) in line {
            let w = w.min(bar.w);
            out.push(Rect::new(x, y + (height - h) / 2, w, h));
            x += w + BUTTON_GAP;
        }
        y += height + BUTTON_GAP;
        rest = next;
    }
    (out, y)
}

/// The game screen's fixed rectangles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Layout {
    /// The battlefield view.
    pub map: Rect,
    /// The sidebar.
    pub bar: Rect,
    /// The radar.
    pub radar: Rect,
    /// The credits line.
    pub credits: Rect,
    /// The power bar.
    pub power: Rect,
    /// The mode buttons: sell, repair, power.
    pub modes: [Rect; 3],
    /// The tab buttons.
    pub tabs: [Rect; TABS],
    /// The icon grid.
    pub grid: Rect,
    /// Scroll up and down.
    pub scroll: [Rect; 2],
    /// The options button.
    pub options: Rect,
}

impl Layout {
    /// The layout for a window of `size`, every button in our own cell.
    pub fn new(size: (i32, i32)) -> Self {
        Self::with_buttons(size, &ButtonArt::default())
    }

    /// The layout for a window of `size` with the button images `art`.
    ///
    /// design: S-0411 claim 19 (*Layout*) leaves positions, spacing and
    /// cells to us and gives no image sizes. A button with an image takes
    /// its file's canvas as its cell, so the image is drawn whole and
    /// unscaled. Each row (sell, repair and power; the four tabs; the two
    /// scroll buttons) is centred in the sidebar with 4 pixels between its
    /// cells and is as tall as its tallest cell; a row too wide for the
    /// sidebar goes on over more lines, and a cell wider than the sidebar
    /// is narrowed to it (its image clipped). The power button keeps our
    /// width of 44, narrowed to what the row leaves (at least 16, else it
    /// takes a line of its own), and the row's height. Without images every
    /// rectangle is the one of our earlier layout.
    pub fn with_buttons(size: (i32, i32), art: &ButtonArt) -> Self {
        let (w, h) = size;
        let bar = Rect::new(w - SIDEBAR_W, 0, SIDEBAR_W, h);
        let x = bar.x + (SIDEBAR_W - RADAR.0) / 2;
        let radar = Rect::new(x, 22, RADAR.0, RADAR.1);
        let options = Rect::new(x, 2, RADAR.0, 16);
        let credits = Rect::new(x, radar.bottom() + 4, RADAR.0, 14);
        let power = Rect::new(x, credits.bottom() + 2, RADAR.0, 8);
        let cell = |a: Option<ButtonImage>, own: (i32, i32)| a.map_or(own, |a| a.size);
        let my = power.bottom() + 6;
        let (sell, repair) = (cell(art.sell, MODE_CELL), cell(art.repair, MODE_CELL));
        let mh = sell.1.max(repair.1);
        let room = bar.w - 4 - sell.0 - repair.0 - 2 * BUTTON_GAP;
        let (modes, ty) = if room >= POWER_MIN_W {
            let pw = MODE_CELL.0.min(room);
            let (r, bottom) = row(bar, my, &[sell, repair, (pw, mh)]);
            ([r[0], r[1], r[2]], bottom)
        } else {
            let (r, bottom) = row(bar, my, &[sell, repair]);
            let (p, bottom) = row(bar, bottom, &[MODE_CELL]);
            ([r[0], r[1], p[0]], bottom)
        };
        let tab_cells: Vec<(i32, i32)> = art.tabs.iter().map(|a| cell(*a, TAB_CELL)).collect();
        let (t, tb) = row(bar, ty, &tab_cells);
        let tabs = [t[0], t[1], t[2], t[3]];
        let gy = tb + 2;
        let scroll_cells = [
            cell(art.scroll[0], SCROLL_CELL),
            cell(art.scroll[1], SCROLL_CELL),
        ];
        // The scroll buttons' height, on one line or two.
        let sh = row(bar, 0, &scroll_cells).1 - BUTTON_GAP;
        let rows = ((h - gy - sh - 8) / (CAMEO.1 + 4)).max(1);
        let grid = Rect::new(x, gy, RADAR.0, rows * (CAMEO.1 + 4));
        let sy = grid.bottom() + 2;
        let (s, _) = row(bar, sy, &scroll_cells);
        let scroll = [s[0], s[1]];
        Self {
            map: Rect::new(0, 0, w - SIDEBAR_W, h),
            bar,
            radar,
            credits,
            power,
            modes,
            tabs,
            grid,
            scroll,
            options,
        }
    }

    /// The number of grid rows shown.
    pub fn rows(&self) -> usize {
        (self.grid.h / (CAMEO.1 + 4)) as usize
    }

    /// The cell of the n-th shown icon (after scrolling).
    pub fn cameo(&self, n: usize) -> Rect {
        let col = (n % 2) as i32;
        let row = (n / 2) as i32;
        Rect::new(
            self.grid.x + col * (CAMEO.0 + 4),
            self.grid.y + row * (CAMEO.1 + 4),
            CAMEO.0,
            CAMEO.1,
        )
    }
}

/// A button symbol: 11 rows of 12 pixels, `#` drawn and `.` clear.
pub type Symbol = [&'static str; SYMBOL.1 as usize];

/// A symbol's size in pixels.
pub const SYMBOL: (i32, i32) = (12, 11);

// design: M8 A4 round 7 note 41 (the command and tab buttons show symbols of our own drawing; no spec names their install images, Q-2238)
const SELL_SYMBOL: Symbol = [
    "....####....",
    "..##....##..",
    ".#...#....#.",
    ".#..####..#.",
    "#..#.#.....#",
    "#...###....#",
    "#....#.#...#",
    ".#..####..#.",
    ".#...#....#.",
    "..##....##..",
    "....####....",
];
const REPAIR_SYMBOL: Symbol = [
    "........#..#",
    "........#..#",
    "........####",
    ".......###..",
    "......###...",
    ".....###....",
    "....###.....",
    "...###......",
    "..###.......",
    ".###........",
    ".##.........",
];
const POWER_SYMBOL: Symbol = [
    ".....#####..",
    "....#####...",
    "...#####....",
    "..#####.....",
    ".#########..",
    "....#####...",
    "...#####....",
    "..####......",
    ".###........",
    ".##.........",
    "#...........",
];
const BUILDINGS_SYMBOL: Symbol = [
    "..##........",
    "..##........",
    "..##..#...#.",
    "..##.##..##.",
    "..#####.###.",
    "############",
    "#..........#",
    "#.##.##.##.#",
    "#.##.##.##.#",
    "#..........#",
    "############",
];
const DEFENCES_SYMBOL: Symbol = [
    "............",
    "....####....",
    "...######...",
    "...#########",
    "...######...",
    "....####....",
    "....####....",
    "...######...",
    "..########..",
    ".##########.",
    ".##########.",
];
const INFANTRY_SYMBOL: Symbol = [
    ".....##.....",
    "....####....",
    ".....##.....",
    "...######...",
    "..#.####.#..",
    "..#.####.#..",
    "....####....",
    "....#..#....",
    "....#..#....",
    "....#..#....",
    "...##..##...",
];
const VEHICLES_SYMBOL: Symbol = [
    "............",
    "............",
    "....####....",
    "...#########",
    "...#####....",
    ".##########.",
    "############",
    "#..........#",
    "#.##.##.##.#",
    ".##########.",
    "............",
];

/// Each tab's symbol and name (shown on hover).
const TAB_BUTTONS: [(Symbol, &str); TABS] = [
    (BUILDINGS_SYMBOL, "Buildings"),
    (DEFENCES_SYMBOL, "Defences"),
    (INFANTRY_SYMBOL, "Infantry"),
    (VEHICLES_SYMBOL, "Vehicles"),
];
/// Each command button's mode, symbol and name (shown on hover).
const MODE_BUTTONS: [(Mode, Symbol, &str); 3] = [
    (Mode::Sell, SELL_SYMBOL, "Sell"),
    (Mode::Repair, REPAIR_SYMBOL, "Repair"),
    (Mode::Power, POWER_SYMBOL, "Power on/off"),
];

/// Draws `s` centred in `r`, one fill per run of drawn pixels in a row.
fn symbol(d: &mut DrawList, r: Rect, s: &Symbol, ink: crate::draw::Rgba) {
    let x0 = r.x + (r.w - SYMBOL.0) / 2;
    let y0 = r.y + (r.h - SYMBOL.1) / 2;
    for (y, row) in s.iter().enumerate() {
        let bytes = row.as_bytes();
        let mut x = 0;
        while x < bytes.len() {
            if bytes[x] != b'#' {
                x += 1;
                continue;
            }
            let start = x;
            while x < bytes.len() && bytes[x] == b'#' {
                x += 1;
            }
            d.fill(
                Rect::new(x0 + start as i32, y0 + y as i32, (x - start) as i32, 1),
                ink,
            );
        }
    }
}

/// A push button showing symbol `s` (see [`widget::button`] for `hot` and
/// `enabled`).
fn symbol_button(d: &mut DrawList, r: Rect, s: &Symbol, hot: bool, enabled: bool) {
    widget::button(d, r, "", hot, enabled);
    symbol(d, r, s, if enabled { colour::TEXT } else { colour::DIM });
}

/// Draws frame `frame` of the button image `a` in its cell `r`, when it
/// has that frame; returns whether it drew (S-0411 claim 19: otherwise the
/// button draws our own symbol). The image is drawn as it is: no hover or
/// press styling is added, since a resting pointer changes no frame and a
/// held press has no frame of its own on a tab, sell or repair button.
fn image(d: &mut DrawList, r: Rect, a: Option<ButtonImage>, frame: u32) -> bool {
    match a {
        Some(a) if a.has(frame) => {
            // design: S-0411 claim 19 (Layout: the image drawn into our own cell, its canvas centred in it)
            d.sprite(
                a.id,
                frame,
                r.x + (r.w - a.size.0) / 2,
                r.y + (r.h - a.size.1) / 2,
                r,
            );
            true
        }
        _ => false,
    }
}

/// The largest scroll position for `n` icons in `rows` rows.
pub fn max_scroll(n: usize, rows: usize) -> usize {
    // spec: S-0411 Formulas, settled scroll end (E = max(0, ⌈N/2⌉ − R), two icons per row)
    n.div_ceil(2).saturating_sub(rows)
}

/// Whether a tab of `n` icons has more than fit in `rows` rows, so that
/// its page buttons are enabled (S-0411 claim 18).
pub fn overflows(n: usize, rows: usize) -> bool {
    // spec: S-0411 claim 18 (both buttons enabled whenever the tab has more icons than fit, two per row)
    n > rows * 2
}

/// Whether a page request in direction `d` is at its requested end
/// (S-0411 claim 18): the top end at *T* = 0, the bottom end when
/// 2 × (*T* + *R*) ≥ *N*.
pub fn at_end(d: i32, top: usize, rows: usize, n: usize) -> bool {
    if d < 0 {
        // spec: S-0411 claim 18 (the top end is T = 0)
        top == 0
    } else {
        // spec: S-0411 claim 18 (the bottom end when 2 × (T + R) ≥ N)
        2 * (top + rows) >= n
    }
}

/// Draws the sidebar. `radar_image`: the client's image id of the radar.
pub fn view(
    m: &HudModel,
    size: (i32, i32),
    radar_image: Option<u32>,
    hover: Option<Point>,
) -> (DrawList, Targets<HudHit>) {
    let l = Layout::with_buttons(size, &m.buttons);
    let mut d = DrawList::new();
    let mut t = Targets::new();
    let hot = |r: Rect| hover.is_some_and(|p| r.contains(p));
    t.push_hit(l.map, HudHit::Map);
    d.fill(l.bar, colour::PANEL);
    d.fill(Rect::new(l.bar.x, 0, 1, l.bar.h), colour::ACCENT);
    widget::button(&mut d, l.options, "Options", hot(l.options), true);
    t.push(l.options, HudHit::Options);

    d.fill(l.radar, colour::BACKDROP);
    d.frame(
        Rect::new(l.radar.x - 1, l.radar.y - 1, l.radar.w + 2, l.radar.h + 2),
        colour::EDGE,
    );
    match radar_image {
        Some(id) if m.radar => {
            d.image(id, l.radar);
            // spec: S-0410 claim 11 (the outline that marks the battlefield view's part of the map is drawn over the radar map)
            if let Some(b) = m.view_box {
                for (a, c) in view_box_lines(b, Point::new(l.radar.x, l.radar.y)) {
                    d.line(a, c, VIEW_BOX_INK);
                }
            }
            t.push_hit(l.radar, HudHit::Radar);
        }
        _ => d.text_centred(l.radar, "No radar", colour::DIM),
    }

    d.text_centred(l.credits, &format!("$ {}", m.credits), colour::ACCENT);
    // Power: S-0405 claim 19's three parts from the left, supplied, near
    // shortage and further shortage (the colours are our look).
    d.fill(l.power, colour::BACKDROP);
    let mut x = l.power.x + 1;
    for (ext, ink) in m
        .meter
        .iter()
        .zip([colour::GOOD, colour::ACCENT, colour::BAD])
    {
        d.fill(Rect::new(x, l.power.y + 1, *ext, l.power.h - 2), ink);
        x += ext;
    }
    // design: M8 A4 round 7 note 43 (with output above drain the supplied part fills the
    // occupied extent, S-0405 claim 19; our mark inside it shows how much of it the drain uses)
    if let Some(at) = drain_mark(m.power, m.meter[0]) {
        d.fill(
            Rect::new(l.power.x + 1 + at, l.power.y, 1, l.power.h),
            colour::TEXT,
        );
    }
    d.frame(l.power, colour::EDGE);
    // The button or bar under the pointer, with its tooltip lines.
    let mut named: Option<(Rect, Vec<String>)> = None;
    if hot(l.power) {
        named = Some((l.power, power_lines(m.power)));
    }

    for (r, (mode, sym, name)) in l.modes.iter().zip(MODE_BUTTONS) {
        let active = m.mode == Some(mode);
        if !image(&mut d, *r, m.buttons.mode(mode), mode_frame(active)) {
            // design: S-0411 claim 19 (a file or frame that cannot be resolved, and the power button: our own symbol)
            symbol_button(&mut d, *r, &sym, hot(*r) || active, true);
        }
        t.push(*r, HudHit::Mode(mode));
        if hot(*r) {
            named = Some((*r, vec![name.to_string()]));
        }
    }
    for (i, r) in l.tabs.iter().enumerate() {
        let tv = m.tabs[i];
        // spec: S-0411 claim 11 (the flash state from the tick that started it; the lit look is implementer design)
        let lit = tv.flashes()
            && tv
                .flash_start
                .is_some_and(|t| tab_lit(t, m.tick, m.flash_extra));
        let active = i == m.shown;
        let (sym, name) = TAB_BUTTONS[i];
        let frame = tab_frame(tv.count == 0, lit, active);
        if !image(&mut d, *r, m.buttons.tabs[i], frame) {
            // design: S-0411 claim 19 (a file or frame that cannot be resolved: our own symbol; claim 11's lit look is then ours)
            symbol_button(&mut d, *r, &sym, active || lit || hot(*r), tv.count > 0);
            if active {
                d.fill(Rect::new(r.x, r.bottom() - 2, r.w, 2), colour::ACCENT);
            }
        }
        if hot(*r) {
            named = Some((*r, vec![name.to_string()]));
        }
        if tv.count > 0 {
            t.push(*r, HudHit::Tab(i));
        }
    }

    d.fill(l.grid, colour::BACKDROP);
    let rows = l.rows();
    let first = m.scroll.min(max_scroll(m.cameos.len(), rows)) * 2;
    let mut tip = None;
    for (n, c) in m.cameos.iter().enumerate().skip(first).take(rows * 2) {
        let r = l.cameo(n - first);
        cameo(&mut d, r, c, hot(r));
        t.push(r, HudHit::Cameo(n));
        if hot(r) || (tip.is_none() && m.focus_cameo == Some(n)) {
            tip = Some((r, c));
        }
    }
    // spec: S-0411 claim 18 (both page buttons disabled when all icons fit)
    let enabled = overflows(m.cameos.len(), rows);
    for (k, (r, label)) in l.scroll.iter().zip(["Up", "Down"]).enumerate() {
        let frame = scroll_frame(!enabled, m.scroll_pressed == Some(k));
        if !image(&mut d, *r, m.buttons.scroll[k], frame) {
            // design: S-0411 claim 19 (a file or frame that cannot be resolved: our own button)
            widget::button(&mut d, *r, label, enabled && hot(*r), enabled);
        }
        if enabled {
            t.push(*r, HudHit::Scroll(if k == 0 { -1 } else { 1 }));
        }
    }
    // The pointer's icon wins over the keyboard's.
    if let Some((r, c)) = tip {
        tooltip(&mut d, r, tooltip_lines(c), size);
    } else if let Some((r, lines)) = named {
        tooltip(&mut d, r, lines, size);
    }
    (d, t)
}

/// Where the drain mark sits inside the power bar, in bands from its inner
/// left edge, for totals `(O, D)` and the supplied extent `supplied`: at
/// ⌊supplied × D / O⌋ when 0 < D < O, none otherwise (with D ≥ O the
/// occupied extent already shows the drain, S-0405 claim 19).
// design: M8 A4 round 7 note 43 (our mark of the drain within the supplied part; the three parts' extents are F4's, unchanged)
pub fn drain_mark((o, d): (i32, i32), supplied: i32) -> Option<i32> {
    (d > 0 && d < o).then(|| (i64::from(supplied) * i64::from(d) / i64::from(o)) as i32)
}

/// The power bar's tooltip: the totals it represents (S-0405 claim 4).
// design: M8 A4 round 7 note 43 (the output and drain shown as numbers on hover)
pub fn power_lines((o, d): (i32, i32)) -> Vec<String> {
    vec![
        "Power".to_string(),
        format!("Output {o}"),
        format!("Drain {d}"),
    ]
}

/// The lines of an icon's tooltip: its name, then a build type's price
/// (S-0404) or a superweapon's state and time left (S-0512 claim 16, F6).
///
/// design: M8 A4 round 5 note 33 asks for prices on the icons; no spec
/// covers a tooltip, so its content and look are ours.
pub fn tooltip_lines(c: &CameoView) -> Vec<String> {
    let mut v = vec![c.name.clone()];
    match c.entry {
        // spec: S-0404 (the price to the house)
        Entry::Type(_) => v.push(format!("$ {}", c.price)),
        Entry::Superweapon(_) => {
            if c.on_hold {
                v.push("On hold".into());
            } else if c.ready {
                v.push("Ready".into());
            } else if let Some(s) = c.seconds {
                let state = c.state.unwrap_or("Charging");
                v.push(format!("{state}: {}", countdown_time(s)));
            }
        }
    }
    v
}

/// Draws a tooltip of `lines` (the first one a name) beside the hovered
/// rectangle `r`, on the battlefield side of the sidebar, kept on the
/// screen.
fn tooltip(d: &mut DrawList, r: Rect, lines: Vec<String>, size: (i32, i32)) {
    let w = lines.iter().map(|s| font::text_width(s)).max().unwrap_or(0) + 8;
    let h = lines.len() as i32 * (font::LINE_HEIGHT + 2) + 4;
    let x = (r.x - w - 4).max(0);
    let y = r.y.min(size.1 - h).max(0);
    let b = Rect::new(x, y, w, h);
    d.fill(b, [0, 0, 0, 210]);
    d.frame(b, colour::EDGE);
    for (i, s) in lines.into_iter().enumerate() {
        let ink = if i == 0 { colour::TEXT } else { colour::ACCENT };
        d.text(x + 4, y + 3 + i as i32 * (font::LINE_HEIGHT + 2), s, ink, b);
    }
}

fn cameo(d: &mut DrawList, r: Rect, c: &CameoView, hot: bool) {
    d.fill(r, colour::RAISED);
    match c.art {
        Some((id, frame, w, h)) => d.sprite(id, frame, r.x + (r.w - w) / 2, r.y + (r.h - h) / 2, r),
        None => {
            // Our placeholder: the type's name on a hatched card.
            for k in (0..r.w + r.h).step_by(6) {
                d.line(
                    (r.x + k - r.h + 1, r.bottom() - 1),
                    (r.x + k, r.y),
                    colour::PANEL,
                );
            }
            d.text_centred(Rect::new(r.x, r.y + 4, r.w, 12), &c.name, colour::TEXT);
            if matches!(c.entry, Entry::Type(_)) {
                d.text_centred(
                    Rect::new(r.x, r.y + 18, r.w, 12),
                    &format!("${}", c.price),
                    colour::DIM,
                );
            }
        }
    }
    if c.darkened {
        d.fill(r, colour::VEIL);
    }
    if let Some(stage) = c.stage.filter(|_| !c.ready) {
        // The clock overlay (S-0411 claim 8): our look veils the part
        // still to build, from the top, in 55 stages.
        // spec: S-0411 claim 8 (stages 1 … 55)
        let veil = r.h - r.h * stage.clamp(1, 55) / 55;
        d.fill(Rect::new(r.x, r.y, r.w, veil), [0, 0, 0, 120]);
    }
    if let Some(stage) = c.charge {
        // A superweapon's charge (S-0512 claim 16, our look): the same
        // veil over the part still to charge, in 54 stages.
        // spec: S-0512 F3 (clock stages 0 … 53 while charging, 54 when ready)
        let veil = r.h - r.h * stage.clamp(0, 54) / 54;
        d.fill(Rect::new(r.x, r.y, r.w, veil), [0, 0, 0, 120]);
    }
    // spec: S-0512 claim 16 (on hold while suspended; otherwise ready; drain-mode types show their state)
    let label = if c.on_hold {
        Some(("On hold", colour::ACCENT))
    } else if c.ready {
        Some(("Ready", colour::GOOD))
    } else {
        c.state.map(|s| (s, colour::TEXT))
    };
    if let Some((s, ink)) = label {
        let band = Rect::new(r.x, r.bottom() - 13, r.w, 13);
        d.fill(band, [0, 0, 0, 170]);
        d.text_centred(band, s, ink);
    }
    if let Some(q) = c.queued {
        let s = q.to_string();
        let bw = font::text_width(&s) + 4;
        let b = Rect::new(r.right() - bw, r.y, bw, 11);
        d.fill(b, [0, 0, 0, 170]);
        d.text(b.x + 2, b.y + 2, s, colour::TEXT, b);
    }
    d.frame(r, if hot { colour::ACCENT } else { colour::EDGE });
}

/// A pause overlay with the options menu (S-0201 claim 8: no cycle runs
/// while it is open).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PauseHit {
    /// Close the menu.
    Resume,
    /// Speed control step (S-0201 claim 8).
    Speed(i32),
    /// Surrender and keep watching (S-0414 claim 11).
    Surrender,
    /// Surrender, then leave the game (S-0201 claim 15, a network game).
    SurrenderLeave,
    /// Open the settings screen.
    Settings,
    /// Leave to the main menu, sending no order.
    Leave,
}

impl PauseHit {
    /// The label of a leave choice.
    fn leave_label(self) -> &'static str {
        match self {
            Self::Surrender => "Surrender",
            Self::SurrenderLeave => "Surrender and leave",
            _ => "Leave game",
        }
    }
}

/// Draws the options menu over the game. `speed` is the index the speed
/// control shows, or `None` when the menu offers no speed control (S-0201
/// claim 17: once the local house is defeated in a network game). In a
/// network game (`network`) the menu stops nothing (S-0201 claim 15), so
/// its title does not say paused, and it offers no settings screen.
/// `leaves` are the leave choices offered, in order (S-0201 claims 1 and
/// 15): any of [`PauseHit::Surrender`], [`PauseHit::SurrenderLeave`] and
/// [`PauseHit::Leave`].
pub fn pause_view(
    size: (i32, i32),
    speed: Option<u8>,
    network: bool,
    leaves: &[PauseHit],
    hover: Option<Point>,
) -> (DrawList, Targets<PauseHit>) {
    let (w, h) = size;
    let mut d = DrawList::new();
    let mut t = Targets::new();
    d.fill(Rect::new(0, 0, w, h), [0, 0, 0, 120]);
    let r = Rect::new((w - 240) / 2, (h - 186) / 2, 240, 186);
    let title = if network {
        "OPTIONS"
    } else {
        "OPTIONS (PAUSED)"
    };
    widget::panel(&mut d, r, Some(title));
    let hot = |x: Rect| hover.is_some_and(|p| x.contains(p));
    if let Some(speed) = speed {
        // spec: S-0201 claim 8 (control position p sets index 6 − p)
        let pos = 6 - speed.min(6);
        let (a, b) = widget::choice(
            &mut d,
            Rect::new(r.x + 10, r.y + 26, 220, 16),
            "Speed",
            &format!("{} of 7", pos + 1),
            true,
        );
        t.push_choice(a, PauseHit::Speed(-1), b, PauseHit::Speed(1));
    }
    let mut y = r.y + 50;
    let mut items = vec![(PauseHit::Resume, "Resume")];
    if !network {
        items.push((PauseHit::Settings, "Settings"));
    }
    items.extend(leaves.iter().map(|&l| (l, l.leave_label())));
    for (hit, label) in items {
        let br = Rect::new(r.x + 30, y, 180, 20);
        widget::button(&mut d, br, label, hot(br), true);
        t.push(br, hit);
        y += 26;
    }
    (d, t)
}

/// The new game-speed index after a step of the options menu's control
/// (S-0201 claim 8).
pub fn step_speed(speed: u8, d: i32) -> u8 {
    // spec: S-0201 claim 8 (index 6 − p for control position p, 0 slowest to 6 fastest)
    let pos = (6 - i32::from(speed.min(6)) + d).clamp(0, 6);
    (6 - pos) as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::draw::Prim;
    use glasnost_sim::rules::Kind;

    fn cameo_view(i: u32) -> CameoView {
        CameoView {
            entry: Entry::Type(TypeRef {
                kind: Kind::Building,
                index: i,
            }),
            name: format!("T{i}"),
            price: 100,
            darkened: false,
            stage: None,
            ready: false,
            on_hold: false,
            queued: None,
            art: None,
            charge: None,
            state: None,
            seconds: None,
        }
    }

    #[test]
    fn layout_fits_the_radar_and_stacks_downwards() {
        let l = Layout::new((1024, 768));
        // spec: S-0410 Constants (radar 140 × 108)
        assert_eq!((l.radar.w, l.radar.h), (140, 108));
        assert_eq!(l.map, Rect::new(0, 0, 1024 - SIDEBAR_W, 768));
        assert!(l.credits.y >= l.radar.bottom());
        assert!(l.tabs[0].y > l.modes[0].bottom());
        assert!(l.grid.y > l.tabs[0].bottom());
        assert!(l.scroll[0].bottom() <= 768);
        assert!(l.rows() >= 1);
        for r in l.tabs.iter().chain(&l.modes) {
            assert!(l.bar.intersect(r) == *r, "{r:?} inside the bar");
        }
    }

    #[test]
    fn icons_hit_in_the_grid_with_scrolling() {
        let mut m = HudModel::default();
        m.tabs[0].count = 30;
        m.cameos = (0..30).map(cameo_view).collect();
        let size = (800, 600);
        let l = Layout::new(size);
        let (_, t) = view(&m, size, None, None);
        assert_eq!(t.hit(l.cameo(0).centre()), Some(HudHit::Cameo(0)));
        assert_eq!(t.hit(l.cameo(3).centre()), Some(HudHit::Cameo(3)));
        m.scroll = 2;
        let (_, t) = view(&m, size, None, None);
        assert_eq!(t.hit(l.cameo(0).centre()), Some(HudHit::Cameo(4)));
        m.scroll = 99;
        let (_, t) = view(&m, size, None, None);
        let last_first = max_scroll(30, l.rows()) * 2;
        assert_eq!(t.hit(l.cameo(0).centre()), Some(HudHit::Cameo(last_first)));
        assert_eq!(t.hit(Point::new(10, 10)), Some(HudHit::Map));
        assert_eq!(t.hit(l.modes[0].centre()), Some(HudHit::Mode(Mode::Sell)));
        // A tab without icons is not a target.
        assert_eq!(t.hit(l.tabs[2].centre()), None);
        assert_eq!(t.hit(l.tabs[0].centre()), Some(HudHit::Tab(0)));
    }

    #[test]
    fn tab_flash_follows_its_start_tick() {
        // spec: S-0411 claim 11 (start at t = 23, r = 3: initial lit iff ⌊t/10⌋ + 1 even, so unlit; first toggle in tick t + 19 − r = 39)
        assert!(!tab_lit(23, 23, true));
        assert!(!tab_lit(23, 38, true));
        assert!(tab_lit(23, 39, true));
        // spec: S-0411 claim 11 (later toggles 10 ticks apart: u lit iff ⌊(u + 1)/10⌋ even)
        assert!(tab_lit(23, 48, true));
        assert!(!tab_lit(23, 49, true));
        // An even ⌊t/10⌋ starts unlit for 20 − r updates.
        assert!(!tab_lit(40, 40, true));
        assert!(!tab_lit(40, 58, true));
        assert!(tab_lit(40, 59, true));
        // spec: S-0411 Edge cases (quirks.tab_flash_initial_extra_period off: initial lit iff ⌊t/10⌋ even, toggle in tick t + 9 − r)
        assert!(tab_lit(23, 23, false));
        assert!(tab_lit(23, 28, false));
        assert!(!tab_lit(23, 29, false));
        assert!(tab_lit(23, 39, false));
        // The drawn tab follows the flash state.
        let mut m = HudModel {
            flash_extra: true,
            ..HudModel::default()
        };
        m.tabs[0] = TabView {
            count: 1,
            waiting: true,
            flash_start: Some(23),
            sw_ready: false,
        };
        m.shown = 1;
        let look = |m: &HudModel| view(m, (800, 600), None, None).0;
        m.tick = 23;
        let a = look(&m);
        m.tick = 38;
        assert_eq!(look(&m), a);
        m.tick = 39;
        assert_ne!(look(&m), a);
        m.tick = 49;
        assert_eq!(look(&m), a);
    }

    #[test]
    fn options_speed_steps_through_the_seven_indices() {
        // spec: S-0201 claim 8 (index 6 − p)
        assert_eq!(step_speed(6, 1), 5);
        assert_eq!(step_speed(0, 1), 0);
        assert_eq!(step_speed(0, -1), 1);
        assert_eq!(step_speed(6, -1), 6);
        let skirmish = [PauseHit::Surrender, PauseHit::Leave];
        let (d, t) = pause_view((800, 600), Some(4), false, &skirmish, None);
        assert!(d.texts().contains(&"3 of 7"));
        assert!(d.texts().contains(&"OPTIONS (PAUSED)"));
        let r = t.rect_of(&PauseHit::Resume).unwrap();
        assert_eq!(t.hit(r.centre()), Some(PauseHit::Resume));
        assert!(t.rect_of(&PauseHit::Settings).is_some());
        let (d, t) = pause_view((800, 600), Some(4), true, &[PauseHit::Leave], None);
        assert!(t.rect_of(&PauseHit::Settings).is_none());
        assert!(
            d.texts().contains(&"OPTIONS"),
            "a network menu stops nothing"
        );
        assert!(t.rect_of(&PauseHit::Speed(1)).is_some());
    }

    // spec: S-0201 claim 17 (no speed control once the local house is defeated in a network game; the view draws none when given none)
    #[test]
    fn options_menu_without_the_speed_control() {
        for network in [false, true] {
            let (d, t) = pause_view((800, 600), None, network, &[PauseHit::Leave], None);
            assert!(!d.texts().iter().any(|x| x.ends_with("of 7")));
            assert!(t.rect_of(&PauseHit::Speed(1)).is_none());
            assert!(t.rect_of(&PauseHit::Speed(-1)).is_none());
            assert!(t.rect_of(&PauseHit::Resume).is_some());
        }
    }

    // spec: S-0201 claim 15 (a network game's two choices that send the surrender order, one leaving and one keeping the player watching; otherwise a leave only)
    #[test]
    fn options_menu_offers_the_leave_choices_it_is_given() {
        let both = [PauseHit::Surrender, PauseHit::SurrenderLeave];
        let (d, t) = pause_view((800, 600), Some(3), true, &both, None);
        assert!(d.texts().contains(&"Surrender"));
        assert!(d.texts().contains(&"Surrender and leave"));
        assert!(t.rect_of(&PauseHit::Leave).is_none());
        let (d, t) = pause_view((800, 600), None, true, &[PauseHit::Leave], None);
        assert!(d.texts().contains(&"Leave game"));
        assert!(t.rect_of(&PauseHit::Surrender).is_none());
        assert!(t.rect_of(&PauseHit::SurrenderLeave).is_none());
    }

    #[test]
    fn shown_tab_falls_back_to_the_first_with_icons() {
        use glasnost_sim::load;
        let w = load::new_game(
            glasnost_ai::fixture::rules(),
            &glasnost_ai::fixture::map(),
            glasnost_ai::fixture::catalogue(),
            &glasnost_ai::fixture::setup(1),
        )
        .unwrap();
        let h = HouseId(0);
        let icons = &w.houses[0].econ.tech.icons;
        let first = icons.iter().position(|v| !v.is_empty());
        // spec: S-0411 claim 2 (switch to the first tab with icons)
        for (cur, tab) in icons.iter().enumerate() {
            let s = shown_tab(&w, h, cur);
            if tab.is_empty() {
                assert_eq!(Some(s), first.or(Some(cur)));
            } else {
                assert_eq!(s, cur);
            }
        }
    }

    #[test]
    fn view_box_lines_keep_the_edges_on_the_image_and_clip_them() {
        let image = Rect::new(10, 20, 100, 50);
        let at = Point::new(1000, 0);
        // spec: S-0410 claim 11 (the view outline is drawn over the radar map)
        let inside = ViewBox {
            image,
            edges: (20, 30, 40, 50),
        };
        let lines = view_box_lines(inside, at);
        assert_eq!(lines.len(), 4);
        assert!(lines.contains(&((1020, 30), (1040, 30))));
        assert!(lines.contains(&((1040, 30), (1040, 50))));
        // A view past the image's left and top: those edges are left out,
        // the others are cut at the image's border.
        let corner = ViewBox {
            image,
            edges: (0, 0, 40, 50),
        };
        let lines = view_box_lines(corner, at);
        assert_eq!(
            lines,
            vec![((1010, 50), (1040, 50)), ((1040, 20), (1040, 50))]
        );
        // Wholly off the image: nothing.
        let off = ViewBox {
            image,
            edges: (200, 0, 300, 10),
        };
        assert!(view_box_lines(off, at).is_empty());
        // Drawn over the radar map only while the radar shows.
        let lines = |m: &HudModel| {
            let d = view(m, (800, 600), Some(1), None).0;
            d.prims
                .iter()
                .filter(|p| matches!(p, Prim::Line { rgba, .. } if *rgba == VIEW_BOX_INK))
                .count()
        };
        let mut m = HudModel {
            view_box: Some(inside),
            ..HudModel::default()
        };
        assert_eq!(lines(&m), 0, "no radar, no outline");
        m.radar = true;
        assert_eq!(lines(&m), 4);
    }

    #[test]
    fn an_icon_tooltip_shows_the_price_or_the_time_left() {
        // spec: S-0404 (the price to the house)
        assert_eq!(tooltip_lines(&cameo_view(3)), ["T3", "$ 100"]);
        let sw = |ready, seconds| CameoView {
            entry: Entry::Superweapon(0),
            ready,
            seconds,
            ..cameo_view(0)
        };
        // spec: S-0512 F6 (the time left in game seconds)
        assert_eq!(
            tooltip_lines(&sw(false, Some(75))),
            ["T0", "Charging: 1:15"]
        );
        assert_eq!(tooltip_lines(&sw(true, None)), ["T0", "Ready"]);
        // Shown for the hovered icon, or else the keyboard's.
        let mut m = HudModel {
            cameos: vec![cameo_view(0), cameo_view(1)],
            ..HudModel::default()
        };
        let texts = |m: &HudModel, hover| {
            view(m, (800, 600), None, hover)
                .0
                .texts()
                .into_iter()
                .map(String::from)
                .collect::<Vec<_>>()
        };
        assert!(!texts(&m, None).contains(&"$ 100".to_string()));
        m.focus_cameo = Some(1);
        let t = texts(&m, None);
        assert!(t.contains(&"T1".to_string()) && t.contains(&"$ 100".to_string()));
        let l = Layout::new((800, 600));
        let over = l.cameo(0);
        let t = texts(&m, Some(Point::new(over.x + 2, over.y + 2)));
        // The pointer's icon: T0's name shows twice (the placeholder card and the tooltip).
        assert_eq!(t.iter().filter(|s| *s == "T0").count(), 2);
        assert_eq!(t.iter().filter(|s| *s == "T1").count(), 1);
    }

    // design: M8 A4 round 7 note 41 (symbols of our own on the command and tab buttons, their names on hover; Q-2238)
    #[test]
    fn command_and_tab_buttons_show_symbols_and_name_on_hover() {
        for s in [
            SELL_SYMBOL,
            REPAIR_SYMBOL,
            POWER_SYMBOL,
            BUILDINGS_SYMBOL,
            DEFENCES_SYMBOL,
            INFANTRY_SYMBOL,
            VEHICLES_SYMBOL,
        ] {
            assert!(s.iter().all(|r| r.len() == SYMBOL.0 as usize));
            assert!(s.iter().all(|r| r.bytes().all(|b| b == b'#' || b == b'.')));
        }
        let mut m = HudModel::default();
        m.tabs[0].count = 1;
        let size = (800, 600);
        let l = Layout::new(size);
        let texts = |m: &HudModel, hover| {
            view(m, size, None, hover)
                .0
                .texts()
                .into_iter()
                .filter(|s| !s.is_empty())
                .map(String::from)
                .collect::<Vec<_>>()
        };
        let plain = texts(&m, None);
        for name in ["Sell", "Repair", "Power on/off", "Buildings", "Vehicles"] {
            assert!(!plain.contains(&name.to_string()), "{name} not as a label");
        }
        // Each button draws its symbol inside its rectangle, dimmed on a tab without icons.
        let (d, _) = view(&m, size, None, None);
        let inks_in = |r: Rect| {
            d.prims
                .iter()
                .filter_map(|p| match p {
                    Prim::Fill { rect, rgba } if r.intersect(rect) == *rect && rect.h == 1 => {
                        Some(*rgba)
                    }
                    _ => None,
                })
                .collect::<Vec<_>>()
        };
        for r in l.modes.iter().chain(&l.tabs[..1]) {
            assert!(inks_in(*r).iter().filter(|c| **c == colour::TEXT).count() >= 11);
        }
        assert!(
            inks_in(l.tabs[3])
                .iter()
                .filter(|c| **c == colour::DIM)
                .count()
                >= 11
        );
        // The hit targets are kept.
        let (_, t) = view(&m, size, None, None);
        assert_eq!(t.hit(l.modes[1].centre()), Some(HudHit::Mode(Mode::Repair)));
        assert_eq!(t.hit(l.tabs[0].centre()), Some(HudHit::Tab(0)));
        // Hover names the button.
        assert!(texts(&m, Some(l.modes[1].centre())).contains(&"Repair".to_string()));
        assert!(texts(&m, Some(l.tabs[3].centre())).contains(&"Vehicles".to_string()));
    }

    // spec: S-0411 claim 19 (the frame tables: the first matching row, top to bottom)
    #[test]
    fn claim19_the_frame_tables() {
        // A tab: unavailable 2 whatever else; lit and shown 4; lit 3; shown 1; else 0.
        for lit in [false, true] {
            for shown in [false, true] {
                assert_eq!(tab_frame(true, lit, shown), 2);
            }
        }
        assert_eq!(tab_frame(false, true, true), 4);
        assert_eq!(tab_frame(false, true, false), 3);
        assert_eq!(tab_frame(false, false, true), 1);
        assert_eq!(tab_frame(false, false, false), 0);
        // Sell and repair: active 1, else 0.
        assert_eq!(mode_frame(true), 1);
        assert_eq!(mode_frame(false), 0);
        // Scroll: disabled 2, even while pressed; pressed with the pointer over it 1; else 0.
        assert_eq!(scroll_frame(true, true), 2);
        assert_eq!(scroll_frame(true, false), 2);
        assert_eq!(scroll_frame(false, true), 1);
        assert_eq!(scroll_frame(false, false), 0);
    }

    /// Synthetic button images: file *k* has id `900 + k`, the frames of
    /// `frames` and the canvas `size`.
    fn art(size: (i32, i32), frames: u32) -> ButtonArt {
        let image = |k: u32| {
            Some(ButtonImage {
                id: 900 + k,
                size,
                frames,
            })
        };
        ButtonArt {
            sell: image(0),
            repair: image(1),
            tabs: [image(2), image(3), image(4), image(5)],
            scroll: [image(6), image(7)],
        }
    }

    fn sprites(d: &DrawList) -> Vec<(u32, u32, i32, i32)> {
        d.prims
            .iter()
            .filter_map(|p| match p {
                Prim::Sprite {
                    id, frame, x, y, ..
                } => Some((*id, *frame, *x, *y)),
                _ => None,
            })
            .collect()
    }

    fn frame_of(d: &DrawList, id: u32) -> Option<u32> {
        sprites(d).into_iter().find(|s| s.0 == id).map(|s| s.1)
    }

    // design: S-0411 claim 19 (Layout: without images every rectangle is our earlier layout's)
    #[test]
    fn claim19_layout_without_images_is_our_earlier_layout() {
        for (w, h) in [(800, 600), (1024, 768), (640, 400)] {
            let l = Layout::with_buttons((w, h), &ButtonArt::default());
            assert_eq!(l, Layout::new((w, h)));
            let x = w - SIDEBAR_W + 14;
            let my = 22 + 108 + 4 + 14 + 2 + 8 + 6;
            for i in 0..3 {
                assert_eq!(l.modes[i as usize], Rect::new(x + i * 48, my, 44, 18));
            }
            let ty = my + 22;
            for i in 0..4 {
                assert_eq!(l.tabs[i as usize], Rect::new(x + i * 36, ty, 32, 18));
            }
            let gy = ty + 24;
            let rows = ((h - gy - 26) / (CAMEO.1 + 4)).max(1);
            assert_eq!(l.grid, Rect::new(x, gy, 140, rows * (CAMEO.1 + 4)));
            let sy = l.grid.bottom() + 2;
            assert_eq!(
                l.scroll,
                [Rect::new(x, sy, 68, 18), Rect::new(x + 72, sy, 68, 18)]
            );
        }
    }

    // design: S-0411 claim 19 (Layout: an image's canvas is its cell, each row centred in the sidebar)
    #[test]
    fn claim19_layout_with_images_takes_their_canvases() {
        let size = (800, 600);
        // (canvas, the power button on a line of its own, the tabs on more than one line)
        for (cell, power_own_row, tabs_wrap) in [
            ((30, 14), false, false),
            ((50, 30), false, true),
            ((70, 26), false, true),
            ((72, 26), true, true),
            ((200, 20), true, true),
        ] {
            let a = art(cell, 0b11111);
            let l = Layout::with_buttons(size, &a);
            let cells = [l.modes[0], l.modes[1]]
                .into_iter()
                .chain(l.tabs)
                .chain(l.scroll);
            for r in cells.clone() {
                // A canvas wider than the sidebar is narrowed to it.
                assert_eq!((r.w, r.h), (cell.0.min(SIDEBAR_W), cell.1), "{r:?}");
            }
            assert_eq!(l.tabs[3].y > l.tabs[0].y, tabs_wrap);
            let all: Vec<Rect> = cells.chain([l.modes[2]]).collect();
            for (i, r) in all.iter().enumerate() {
                assert!(l.bar.intersect(r) == *r, "{r:?} inside the bar");
                for s in &all[i + 1..] {
                    assert!(
                        r.intersect(s).w <= 0 || r.intersect(s).h <= 0,
                        "{r:?} {s:?}"
                    );
                }
            }
            // The power button keeps our symbol's room, beside them or on its own row.
            assert!(l.modes[2].w >= POWER_MIN_W);
            assert_eq!(l.modes[2].y >= l.modes[0].bottom(), power_own_row);
            // The rows stack downwards and the grid holds whole rows of icons.
            assert!(l.tabs[0].y >= l.modes[2].bottom());
            assert!(l.grid.y > l.tabs[0].bottom());
            assert_eq!(l.grid.h, l.rows() as i32 * (CAMEO.1 + 4));
            assert!(l.scroll[0].y > l.grid.bottom());
            assert!(l.scroll[1].bottom() <= size.1);
        }
    }

    // spec: S-0411 claim 19 (each button draws the frame of its state; the power button has no image)
    #[test]
    fn claim19_the_view_draws_each_buttons_frame() {
        let size = (800, 600);
        let mut m = HudModel {
            buttons: art((40, 20), 0b11111),
            flash_extra: true,
            ..HudModel::default()
        };
        // Tab 0 shown, tab 1 lit, tabs 2 and 3 without icons.
        m.tabs[0].count = 30;
        m.tabs[1] = TabView {
            count: 1,
            waiting: true,
            flash_start: Some(23),
            sw_ready: false,
        };
        m.tick = 39; // spec: S-0411 claim 11 (start 23: lit from tick 39)
        m.cameos = (0..30).map(cameo_view).collect();
        let (d, _) = view(&m, size, None, None);
        assert_eq!(frame_of(&d, 900), Some(0), "sell, no mode");
        assert_eq!(frame_of(&d, 901), Some(0), "repair, no mode");
        assert_eq!(frame_of(&d, 902), Some(1), "shown");
        assert_eq!(frame_of(&d, 903), Some(3), "lit");
        assert_eq!(frame_of(&d, 904), Some(2), "unavailable");
        assert_eq!(frame_of(&d, 905), Some(2), "unavailable");
        assert_eq!(frame_of(&d, 906), Some(0), "scroll up, enabled");
        assert_eq!(frame_of(&d, 907), Some(0), "scroll down, enabled");
        assert_eq!(sprites(&d).len(), 8, "no image for the power button");
        m.shown = 1;
        m.mode = Some(Mode::Repair);
        m.scroll_pressed = Some(1);
        let (d, _) = view(&m, size, None, None);
        assert_eq!(frame_of(&d, 900), Some(0));
        assert_eq!(frame_of(&d, 901), Some(1), "repair mode active");
        assert_eq!(
            frame_of(&d, 902),
            Some(0),
            "available, neither lit nor shown"
        );
        assert_eq!(frame_of(&d, 903), Some(4), "lit and shown");
        assert_eq!(frame_of(&d, 906), Some(0));
        assert_eq!(
            frame_of(&d, 907),
            Some(1),
            "pressed with the pointer over it"
        );
        // All icons fit: both scroll buttons disabled, frame 2 even while pressed.
        m.cameos.truncate(2);
        m.mode = Some(Mode::Sell);
        let (d, t) = view(&m, size, None, None);
        assert_eq!(frame_of(&d, 900), Some(1), "sell mode active");
        assert_eq!(frame_of(&d, 901), Some(0));
        assert_eq!(frame_of(&d, 906), Some(2));
        assert_eq!(frame_of(&d, 907), Some(2));
        // The image is centred in its cell, and the hit targets are kept.
        let l = Layout::with_buttons(size, &m.buttons);
        let s = sprites(&d).into_iter().find(|s| s.0 == 900).unwrap();
        assert_eq!((s.2, s.3), (l.modes[0].x, l.modes[0].y));
        assert_eq!(t.hit(l.modes[0].centre()), Some(HudHit::Mode(Mode::Sell)));
        assert_eq!(t.hit(l.modes[2].centre()), Some(HudHit::Mode(Mode::Power)));
        assert_eq!(t.hit(l.tabs[1].centre()), Some(HudHit::Tab(1)));
    }

    // spec: S-0411 claim 19 (a resting pointer changes no frame; a press shows only on a scroll button with the pointer over it)
    #[test]
    fn claim19_hover_changes_no_frame() {
        let size = (800, 600);
        let mut m = HudModel {
            buttons: art((40, 20), 0b11111),
            ..HudModel::default()
        };
        m.tabs[0].count = 30;
        m.cameos = (0..30).map(cameo_view).collect();
        let l = Layout::with_buttons(size, &m.buttons);
        let plain = sprites(&view(&m, size, None, None).0);
        for r in l.modes.iter().chain(&l.tabs).chain(&l.scroll) {
            let (d, _) = view(&m, size, None, Some(r.centre()));
            assert_eq!(sprites(&d), plain, "hover over {r:?}");
        }
        // Hover still names the button.
        let (d, _) = view(&m, size, None, Some(l.modes[0].centre()));
        assert!(d.texts().contains(&"Sell"));
    }

    // spec: S-0411 claim 19 (a file or frame that cannot be resolved: our own symbol, for that state only)
    #[test]
    fn claim19_a_missing_frame_falls_back_to_the_symbol() {
        let size = (800, 600);
        // Only frame 0 of every file.
        let mut m = HudModel {
            buttons: art((40, 20), 0b1),
            ..HudModel::default()
        };
        m.tabs[0].count = 1;
        m.tabs[1].count = 1;
        m.shown = 1;
        let l = Layout::with_buttons(size, &m.buttons);
        let ink_rows_in = |d: &DrawList, r: Rect| {
            d.prims
                .iter()
                .filter(|p| {
                    matches!(p, Prim::Fill { rect, .. } if r.intersect(rect) == *rect && rect.h == 1)
                })
                .count()
        };
        let (d, _) = view(&m, size, None, None);
        assert_eq!(frame_of(&d, 900), Some(0), "sell at rest has its frame");
        assert_eq!(frame_of(&d, 902), Some(0), "tab 0 at rest has its frame");
        assert_eq!(frame_of(&d, 903), None, "tab 1 shown: frame 1 missing");
        assert!(ink_rows_in(&d, l.tabs[1]) >= 11, "our symbol instead");
        assert_eq!(frame_of(&d, 904), None, "unavailable: frame 2 missing");
        assert_eq!(ink_rows_in(&d, l.tabs[0]), 0, "no symbol over an image");
        m.mode = Some(Mode::Sell);
        let (d, _) = view(&m, size, None, None);
        assert_eq!(frame_of(&d, 900), None, "sell active: frame 1 missing");
        assert!(ink_rows_in(&d, l.modes[0]) >= 11);
        assert_eq!(frame_of(&d, 901), Some(0), "repair unaffected");
        // No images at all: every button is ours, no sprite drawn.
        m.buttons = ButtonArt::default();
        let (d, _) = view(&m, size, None, None);
        assert!(sprites(&d).is_empty());
    }

    // spec: S-0405 claim 19 and F4 (the three parts' extents are unchanged); design: M8 A4 round 7 note 43 (a drain mark within the supplied part while 0 < D < O, the totals on hover)
    #[test]
    fn the_power_bar_marks_the_drain_and_names_the_totals() {
        assert_eq!(drain_mark((100, 50), 120), Some(60));
        assert_eq!(drain_mark((300, 25), 120), Some(10));
        assert_eq!(drain_mark((100, 0), 120), None, "no drain");
        assert_eq!(drain_mark((100, 100), 120), None, "all of it used");
        assert_eq!(drain_mark((0, 25), 0), None, "short of power");
        let size = (800, 600);
        let l = Layout::new(size);
        let m = HudModel {
            power: (200, 50),
            meter: [100, 0, 0],
            ..HudModel::default()
        };
        let (d, _) = view(&m, size, None, None);
        let mark = Rect::new(l.power.x + 1 + 25, l.power.y, 1, l.power.h);
        assert!(d.prims.contains(&Prim::Fill {
            rect: mark,
            rgba: colour::TEXT
        }));
        let parts: Vec<Rect> = d
            .prims
            .iter()
            .filter_map(|p| match p {
                Prim::Fill { rect, rgba } if *rgba == colour::GOOD => Some(*rect),
                _ => None,
            })
            .collect();
        assert_eq!(
            parts,
            [Rect::new(l.power.x + 1, l.power.y + 1, 100, l.power.h - 2)]
        );
        let tip = view(&m, size, None, Some(l.power.centre())).0;
        let t = tip.texts();
        assert!(t.contains(&"Output 200") && t.contains(&"Drain 50"));
        assert!(!d.texts().contains(&"Output 200"));
    }

    #[test]
    fn countdown_time_switches_to_hours_at_an_hour() {
        // spec: S-0512 claim 18 (hours, minutes and seconds when s is an hour or more, minutes and seconds otherwise)
        assert_eq!(countdown_time(0), "0:00");
        assert_eq!(countdown_time(59), "0:59");
        assert_eq!(countdown_time(60), "1:00");
        assert_eq!(countdown_time(3599), "59:59");
        assert_eq!(countdown_time(3600), "1:00:00");
        assert_eq!(countdown_time(3661), "1:01:01");
    }

    #[test]
    fn countdown_at_zero_blinks_once_a_real_second() {
        let map = Rect::new(0, 0, 800, 600);
        let row = |seconds| CountdownRow {
            name: "Bolt".into(),
            colour: [200, 0, 0],
            seconds,
        };
        // spec: S-0512 claim 18 (at s = 0 the time text blinks, changing state once per second of real time; the name stays)
        assert!(countdown_blink_on(0) && countdown_blink_on(999));
        assert!(!countdown_blink_on(1000) && !countdown_blink_on(1999));
        assert!(countdown_blink_on(2000));
        let on = countdown_view(&[row(0)], map, 0);
        let off = countdown_view(&[row(0)], map, 1500);
        assert!(on.texts().contains(&"0:00"));
        assert!(!off.texts().contains(&"0:00"));
        assert!(off.texts().contains(&"Bolt "));
        // A running countdown never blinks.
        assert!(
            countdown_view(&[row(5)], map, 1500)
                .texts()
                .contains(&"0:05")
        );
        // spec: S-0512 claim 18 (rows in grant order)
        let two = countdown_view(
            &[
                row(5),
                CountdownRow {
                    name: "Rain".into(),
                    ..row(7)
                },
            ],
            map,
            0,
        );
        let t = two.texts();
        let (a, b) = (
            t.iter().position(|s| *s == "Bolt ").unwrap(),
            t.iter().position(|s| *s == "Rain ").unwrap(),
        );
        assert!(a < b);
    }
}
