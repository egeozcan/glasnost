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

/// One build icon as the sidebar draws it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CameoView {
    /// The type.
    pub t: TypeRef,
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

/// What the sidebar shows.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HudModel {
    /// The credits the display shows (S-0416).
    pub credits: i32,
    /// Power output and drain (S-0405 claim 4).
    pub power: (i32, i32),
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
    let Some(h) = w.houses.get(usize::from(house.0)) else {
        return current;
    };
    let icons = &h.econ.tech.icons;
    if icons.get(current).is_some_and(|v| !v.is_empty()) {
        return current;
    }
    // spec: S-0411 claim 2 (the shown tab without icons: switch to the first tab with icons)
    icons.iter().position(|v| !v.is_empty()).unwrap_or(current)
}

/// Reads the sidebar of `house` from the world. `credits` is the counter's
/// shown value (S-0416); `name` and `art` give each type's name and icon.
pub fn model(
    w: &World,
    house: HouseId,
    shown: usize,
    credits: i32,
    name: &dyn Fn(TypeRef) -> String,
    art: &dyn Fn(TypeRef) -> Option<(u32, u32, i32, i32)>,
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
    m.radar = glasnost_sim::econ::radar::available(w, house);
    for (i, icons) in h.econ.tech.icons.iter().enumerate().take(TABS) {
        m.tabs[i].count = icons.len();
        // spec: S-0411 claim 11 (a done building or defence waiting for placement flashes its tab; tabs 1 and 2 feed those lines, claim 1)
        m.tabs[i].waiting = i < 2
            && icons
                .iter()
                .any(|c| sidebar::icon_state(w, house, c.t).is_some_and(|s| s.ready));
    }
    if let Some(icons) = h.econ.tech.icons.get(m.shown) {
        m.cameos = icons
            .iter()
            .map(|c| {
                let state = sidebar::icon_state(w, house, c.t);
                CameoView {
                    t: c.t,
                    name: name(c.t),
                    price: production::price(w, house, c.t),
                    darkened: sidebar::darkened(w, house, c.t),
                    stage: state.map(|s| s.stage),
                    ready: state.is_some_and(|s| s.ready),
                    on_hold: state.is_some_and(|s| s.on_hold),
                    queued: sidebar::queue_count(w, house, c.t),
                    art: art(c.t),
                }
            })
            .collect();
    }
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
    n.div_ceil(2).saturating_sub(rows)
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
    t.push(l.map, HudHit::Map);
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
            t.push(l.radar, HudHit::Radar);
        }
        _ => d.text_centred(l.radar, "No radar", colour::DIM),
    }

    d.text_centred(l.credits, &format!("$ {}", m.credits), colour::ACCENT);
    // Power: a bar filled to the drain's share of the output (our look);
    // red when the drain exceeds the output (S-0405 claim 4: output and
    // drain).
    let (out, drain) = m.power;
    d.fill(l.power, colour::BACKDROP);
    let full = l.power.w - 2;
    let filled = if out <= 0 {
        if drain > 0 { full } else { 0 }
    } else {
        (full as i64 * drain.min(out) as i64 / out as i64) as i32
    };
    let ok = drain <= out;
    d.fill(
        Rect::new(l.power.x + 1, l.power.y + 1, filled, l.power.h - 2),
        if ok { colour::GOOD } else { colour::BAD },
    );
    d.frame(l.power, colour::EDGE);

    for (r, (mode, label)) in l.modes.iter().zip(MODE_LABELS) {
        widget::button(&mut d, *r, label, hot(*r) || m.mode == Some(mode), true);
        t.push(*r, HudHit::Mode(mode));
    }
    for (i, r) in l.tabs.iter().enumerate() {
        let tv = m.tabs[i];
        // spec: S-0411 claim 11 (the flash state from the tick that started it; the lit look is implementer design)
        let lit = tv.waiting
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
    for (k, (r, label)) in l.scroll.iter().zip(["Up", "Down"]).enumerate() {
        widget::button(&mut d, *r, label, hot(*r), true);
        t.push(*r, HudHit::Scroll(if k == 0 { -1 } else { 1 }));
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
            d.text_centred(
                Rect::new(r.x, r.y + 18, r.w, 12),
                &format!("${}", c.price),
                colour::DIM,
            );
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
    let label = if c.ready {
        Some(("Ready", colour::GOOD))
    } else if c.on_hold {
        Some(("On hold", colour::ACCENT))
    } else {
        None
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
    /// Leave to the main menu.
    Leave,
}

/// Draws the options menu over the game. `speed` is the game-speed index.
pub fn pause_view(
    size: (i32, i32),
    speed: u8,
    hover: Option<Point>,
) -> (DrawList, Targets<PauseHit>) {
    let (w, h) = size;
    let mut d = DrawList::new();
    let mut t = Targets::new();
    d.fill(Rect::new(0, 0, w, h), [0, 0, 0, 120]);
    let r = Rect::new((w - 240) / 2, (h - 160) / 2, 240, 160);
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
    t.push(a, PauseHit::Speed(-1));
    t.push(b, PauseHit::Speed(1));
    let mut y = r.y + 50;
    for (hit, label) in [
        (PauseHit::Resume, "Resume"),
        (PauseHit::Surrender, "Surrender"),
        (PauseHit::Leave, "Leave game"),
    ] {
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
            t: TypeRef {
                kind: Kind::Building,
                index: i,
            },
            name: format!("T{i}"),
            price: 100,
            darkened: false,
            stage: None,
            ready: false,
            on_hold: false,
            queued: None,
            art: None,
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
        let mut m = HudModel::default();
        m.flash_extra = true;
        m.tabs[0] = TabView {
            count: 1,
            waiting: true,
            flash_start: Some(23),
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
        let (d, t) = pause_view((800, 600), 4, None);
        assert!(d.texts().contains(&"3 of 7"));
        let r = t.rect_of(&PauseHit::Resume).unwrap();
        assert_eq!(t.hit(r.centre()), Some(PauseHit::Resume));
    }

    #[test]
    fn shown_tab_falls_back_to_the_first_with_icons() {
        use glasnost_sim::{load, testkit};
        let w = load::new_game(
            glasnost_ai::fixture::rules(),
            &glasnost_ai::fixture::map(),
            testkit::catalogue(),
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
