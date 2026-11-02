//! The watch panel: the sidebar of a game in which no house is played, only
//! computer players (a watch game).
//!
//! design: watch mode (lead request 2026-11-02, an extra not in the
//! original). In the sidebar's place: the options button and the radar
//! where the ordinary sidebar has them, then one row for each player house
//! in house order (its colour, name and credits, or "defeated"), then what
//! the selected object is. There are no tabs, icons or mode buttons: a
//! watch game takes no orders.

use glasnost_sim::econ::{production, victory};
use glasnost_sim::world::{HouseId, HouseRole, World};

use crate::draw::{DrawList, colour};
use crate::font;
use crate::geom::{Point, Rect};
use crate::hud::{HudHit, Layout, VIEW_BOX_INK, ViewBox, view_box_lines};
use crate::widget::{self, Targets};

/// One player house's row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HouseRow {
    /// The house's name.
    pub name: String,
    /// Its colour (S-0103).
    pub colour: [u8; 3],
    /// Its available credits (S-0416).
    pub credits: i32,
    /// Whether it is defeated (S-0414).
    pub defeated: bool,
}

/// The watch panel's model.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WatchModel {
    /// The player houses, in house order.
    pub houses: Vec<HouseRow>,
    /// The battlefield view's outline on the radar (S-0410 claim 11).
    pub view_box: Option<ViewBox>,
    /// The lines that say what the selected object is, empty with none.
    pub selected: Vec<String>,
}

/// The rows of the player houses of `w`, in house order, with each house's
/// colour from `colour`.
pub fn house_rows(w: &World, colour: impl Fn(HouseId) -> [u8; 3]) -> Vec<HouseRow> {
    (0..w.houses.len())
        .filter_map(|i| {
            let h = HouseId(u8::try_from(i).ok()?);
            let house = &w.houses[i];
            // spec: S-0414 claim 7 (the houses that get a result: player houses, no neutral, special or observer's house)
            if house.role != HouseRole::Player || house.removed || house.econ.setup.observer {
                return None;
            }
            Some(HouseRow {
                name: house.name.clone(),
                colour: colour(h),
                credits: production::available_credits(w, h),
                defeated: victory::defeated(w, h),
            })
        })
        .collect()
}

/// The height of one house row (our layout).
const ROW_H: i32 = 16;

/// Draws the watch panel for a window of `size`; `radar_image` is the
/// radar's sprite id. Its targets are the battlefield, the radar and the
/// options button only.
pub fn view(
    m: &WatchModel,
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
    if let Some(id) = radar_image {
        d.image(id, l.radar);
        // spec: S-0410 claim 11 (the outline that marks the battlefield view's part of the map is drawn over the radar map)
        if let Some(b) = m.view_box {
            for (a, c) in view_box_lines(b, Point::new(l.radar.x, l.radar.y)) {
                d.line(a, c, VIEW_BOX_INK);
            }
        }
        t.push_hit(l.radar, HudHit::Radar);
    } else {
        d.text_centred(l.radar, "No radar", colour::DIM);
    }

    let x = l.radar.x;
    let width = l.radar.w;
    let mut y = l.radar.bottom() + 8;
    d.text(x, y, "Watching", colour::DIM, l.bar);
    y += ROW_H;
    for r in &m.houses {
        if y + ROW_H > l.bar.bottom() {
            break;
        }
        let [cr, cg, cb] = r.colour;
        d.fill(Rect::new(x, y + 1, 10, 10), [cr, cg, cb, 255]);
        d.frame(Rect::new(x, y + 1, 10, 10), colour::EDGE);
        let (right, ink) = if r.defeated {
            ("defeated".to_string(), colour::DIM)
        } else {
            (format!("$ {}", r.credits), colour::TEXT)
        };
        let rw = font::text_width(&right);
        d.text(x + width - rw, y, right, ink, l.bar);
        let name = font::fit(&r.name, width - rw - 18);
        d.text(x + 14, y, name, ink, l.bar);
        y += ROW_H;
    }
    if !m.selected.is_empty() {
        y += 6;
        d.fill(Rect::new(x, y, width, 1), colour::EDGE);
        y += 6;
        for line in &m.selected {
            if y + ROW_H > l.bar.bottom() {
                break;
            }
            d.text(x, y, font::fit(line, width), colour::TEXT, l.bar);
            y += ROW_H;
        }
    }
    (d, t)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model() -> WatchModel {
        WatchModel {
            houses: vec![
                HouseRow {
                    name: "Computer 1".into(),
                    colour: [200, 40, 40],
                    credits: 7000,
                    defeated: false,
                },
                HouseRow {
                    name: "Computer 2".into(),
                    colour: [40, 40, 200],
                    credits: 0,
                    defeated: true,
                },
            ],
            view_box: None,
            selected: vec![
                "Tank".into(),
                "Computer 1".into(),
                "Health 300 / 300".into(),
            ],
        }
    }

    // design: watch mode (lead request 2026-11-02, an extra not in the original): each house's name, credits or defeat, and the selected object
    #[test]
    fn the_panel_lists_each_house_and_the_selection() {
        let (d, _) = view(&model(), (800, 600), Some(1), None);
        let texts = d.texts();
        for s in [
            "Computer 1",
            "$ 7000",
            "Computer 2",
            "defeated",
            "Tank",
            "Health 300 / 300",
        ] {
            assert!(texts.contains(&s), "{s} missing from {texts:?}");
        }
    }

    // design: watch mode (lead request 2026-11-02, an extra not in the original): no orders, so the only targets are the battlefield, the radar and the options button
    #[test]
    fn the_panel_offers_no_order() {
        let (_, t) = view(&model(), (800, 600), Some(1), None);
        for (_, hit) in &t.list {
            assert!(
                matches!(hit, HudHit::Map | HudHit::Radar | HudHit::Options),
                "{hit:?}"
            );
        }
        let l = Layout::new((800, 600));
        assert_eq!(t.hit(l.radar.centre()), Some(HudHit::Radar));
        assert_eq!(t.hit(l.options.centre()), Some(HudHit::Options));
    }
}
