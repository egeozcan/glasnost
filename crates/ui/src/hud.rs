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
                }
            }
        })
        .collect();
    m
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
    /// The layout for a window of `size`.
    pub fn new(size: (i32, i32)) -> Self {
        let (w, h) = size;
        let bar = Rect::new(w - SIDEBAR_W, 0, SIDEBAR_W, h);
        let x = bar.x + (SIDEBAR_W - RADAR.0) / 2;
        let radar = Rect::new(x, 22, RADAR.0, RADAR.1);
        let options = Rect::new(x, 2, RADAR.0, 16);
        let credits = Rect::new(x, radar.bottom() + 4, RADAR.0, 14);
        let power = Rect::new(x, credits.bottom() + 2, RADAR.0, 8);
        let my = power.bottom() + 6;
        let modes = [0, 1, 2].map(|i| Rect::new(x + i * 48, my, 44, 18));
        let ty = my + 22;
        let tabs = [0, 1, 2, 3].map(|i| Rect::new(x + i * 36, ty, 32, 18));
        let gy = ty + 24;
        let rows = ((h - gy - 26) / (CAMEO.1 + 4)).max(1);
        let grid = Rect::new(x, gy, RADAR.0, rows * (CAMEO.1 + 4));
        let sy = grid.bottom() + 2;
        let scroll = [Rect::new(x, sy, 68, 18), Rect::new(x + 72, sy, 68, 18)];
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

const TAB_LABELS: [&str; TABS] = ["Bld", "Def", "Inf", "Veh"];
const MODE_LABELS: [(Mode, &str); 3] = [
    (Mode::Sell, "Sell"),
    (Mode::Repair, "Fix"),
    (Mode::Power, "Pwr"),
];

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
    let l = Layout::new(size);
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
    d.frame(l.power, colour::EDGE);

    for (r, (mode, label)) in l.modes.iter().zip(MODE_LABELS) {
        widget::button(&mut d, *r, label, hot(*r) || m.mode == Some(mode), true);
        t.push(*r, HudHit::Mode(mode));
    }
    for (i, r) in l.tabs.iter().enumerate() {
        let tv = m.tabs[i];
        // spec: S-0411 claim 11 (the flash state from the tick that started it; the lit look is implementer design)
        let lit = tv.flashes()
            && tv
                .flash_start
                .is_some_and(|t| tab_lit(t, m.tick, m.flash_extra));
        let active = i == m.shown;
        widget::button(
            &mut d,
            *r,
            TAB_LABELS[i],
            active || lit || hot(*r),
            tv.count > 0,
        );
        if active {
            d.fill(Rect::new(r.x, r.bottom() - 2, r.w, 2), colour::ACCENT);
        }
        if tv.count > 0 {
            t.push(*r, HudHit::Tab(i));
        }
    }

    d.fill(l.grid, colour::BACKDROP);
    let rows = l.rows();
    let first = m.scroll.min(max_scroll(m.cameos.len(), rows)) * 2;
    for (n, c) in m.cameos.iter().enumerate().skip(first).take(rows * 2) {
        let r = l.cameo(n - first);
        cameo(&mut d, r, c, hot(r));
        t.push(r, HudHit::Cameo(n));
    }
    // spec: S-0411 claim 18 (both page buttons disabled when all icons fit)
    let enabled = overflows(m.cameos.len(), rows);
    for (k, (r, label)) in l.scroll.iter().zip(["Up", "Down"]).enumerate() {
        widget::button(&mut d, *r, label, enabled && hot(*r), enabled);
        if enabled {
            t.push(*r, HudHit::Scroll(if k == 0 { -1 } else { 1 }));
        }
    }
    (d, t)
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
    /// Give up and see the end screen.
    Surrender,
    /// Open the settings screen.
    Settings,
    /// Leave to the main menu.
    Leave,
}

/// Draws the options menu over the game. `speed` is the game-speed index.
pub fn pause_view(
    size: (i32, i32),
    speed: u8,
    settings: bool,
    hover: Option<Point>,
) -> (DrawList, Targets<PauseHit>) {
    let (w, h) = size;
    let mut d = DrawList::new();
    let mut t = Targets::new();
    d.fill(Rect::new(0, 0, w, h), [0, 0, 0, 120]);
    let r = Rect::new((w - 240) / 2, (h - 186) / 2, 240, 186);
    widget::panel(&mut d, r, Some("OPTIONS (PAUSED)"));
    let hot = |x: Rect| hover.is_some_and(|p| x.contains(p));
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
    let mut y = r.y + 50;
    let mut items = vec![(PauseHit::Resume, "Resume")];
    if settings {
        items.push((PauseHit::Settings, "Settings"));
    }
    items.extend([
        (PauseHit::Surrender, "Surrender"),
        (PauseHit::Leave, "Leave game"),
    ]);
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
        let (d, t) = pause_view((800, 600), 4, true, None);
        assert!(d.texts().contains(&"3 of 7"));
        let r = t.rect_of(&PauseHit::Resume).unwrap();
        assert_eq!(t.hit(r.centre()), Some(PauseHit::Resume));
        assert!(t.rect_of(&PauseHit::Settings).is_some());
        assert!(
            pause_view((800, 600), 4, false, None)
                .1
                .rect_of(&PauseHit::Settings)
                .is_none()
        );
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
}
