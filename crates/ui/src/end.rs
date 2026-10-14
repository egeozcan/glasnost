//! The end screen: every player's result (S-0414 claim 7) and the counts
//! the world keeps.

use glasnost_sim::econ::stats;
use glasnost_sim::econ::victory::{Outcome, Verdict};
use glasnost_sim::world::{HouseId, World};

use crate::draw::{DrawList, colour};
use crate::geom::{Point, Rect};
use crate::widget::{self, Targets};

/// One player's row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EndRow {
    /// The house's name.
    pub name: String,
    /// Whether it is the local player's house.
    pub local: bool,
    /// Won (S-0414 claim 7).
    pub won: bool,
    /// The built total (S-0414 claim 20).
    pub built: i32,
    /// Kills (S-0414 claim 21).
    pub kills: i32,
    /// Losses (S-0414 claim 21).
    pub losses: i32,
    /// The displayed score (S-0414 claim 23).
    pub score: i32,
}

/// The end screen's model.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct End {
    /// The tick in which the game is over (S-0414 claims 8 and 9).
    pub ticks: i32,
    /// The result duration's active time in 16-ms clock units, *E* of
    /// S-0414's Formulas.
    pub units: i64,
    /// The rows in house order (S-0414 claim 7).
    pub rows: Vec<EndRow>,
}

impl End {
    /// Reads the end screen from the world and its outcome, with `units`
    /// of active play time (S-0414 claim 23).
    pub fn from_world(w: &World, outcome: &Outcome, local: Option<HouseId>, units: i64) -> Self {
        let rows = outcome
            .results
            .iter()
            .map(|&(h, v)| {
                let house = &w.houses[h.0 as usize];
                let st = stats::of(w, h);
                EndRow {
                    name: house.name.clone(),
                    local: Some(h) == local,
                    won: v == Verdict::Win,
                    // spec: S-0414 claim 20 (the built total)
                    built: stats::built_total(w, h),
                    // spec: S-0414 claims 19 and 21 (read after play stops)
                    kills: st.kills,
                    losses: st.losses,
                    // spec: S-0414 claim 23 (the displayed score; b = max(p, 0) before it is fixed)
                    score: st.shown.unwrap_or_else(|| stats::displayed(st.score, None)),
                }
            })
            .collect();
        Self {
            ticks: outcome.over,
            units,
            rows,
        }
    }

    /// The displayed duration as hours, minutes and seconds (S-0414
    /// claim 23 and Formulas).
    pub fn duration(&self) -> (i64, i64, i64) {
        // spec: S-0414 Formulas, result duration (T = min(⌊E/60⌋, 359999); hours ⌊T/3600⌋, minutes ⌊(T mod 3600)/60⌋, seconds T mod 60)
        let t = (self.units.max(0) / 60).min(359_999);
        (t / 3600, (t % 3600) / 60, t % 60) // spec: S-0414 Formulas, result duration
    }

    /// The headline for the local player.
    pub fn headline(&self) -> &'static str {
        match self.rows.iter().find(|r| r.local) {
            Some(r) if r.won => "VICTORY",
            Some(_) => "DEFEAT",
            None => "GAME OVER",
        }
    }

    /// Draws the screen.
    pub fn view(&self, size: (i32, i32), hover: Option<Point>) -> (DrawList, Targets<()>) {
        let (w, h) = size;
        let mut d = DrawList::new();
        let mut t = Targets::new();
        d.fill(Rect::new(0, 0, w, h), colour::BACKDROP);
        let outer = Rect::new(
            (w - 520).max(0) / 2,
            (h - 320).max(0) / 2,
            520.min(w),
            320.min(h),
        );
        widget::panel(&mut d, outer, Some("GAME OVER"));
        let head = self.headline();
        let good = head == "VICTORY";
        d.text_centred(
            Rect::new(outer.x, outer.y + 24, outer.w, 20),
            head,
            if good { colour::GOOD } else { colour::BAD },
        );
        let (hh, mm, ss) = self.duration();
        let time = format!("Game time {hh}:{mm:02}:{ss:02} ({} ticks)", self.ticks);
        d.text_centred(
            Rect::new(outer.x, outer.y + 46, outer.w, 14),
            &time,
            colour::DIM,
        );
        let x0 = outer.x + 12;
        let cols = [
            (0, "Player"),
            (150, "Result"),
            (220, "Built"),
            (280, "Kills"),
            (340, "Lost"),
            (400, "Score"),
        ];
        let mut y = outer.y + 70;
        for (cx, name) in cols {
            d.text(x0 + cx, y, name, colour::DIM, outer);
        }
        y += 14;
        for (i, r) in self.rows.iter().enumerate() {
            let row = Rect::new(outer.x + 6, y - 2, outer.w - 12, 14);
            if i % 2 == 0 {
                d.fill(row, colour::RAISED);
            }
            let ink = if r.local {
                colour::ACCENT
            } else {
                colour::TEXT
            };
            let values = [
                (0, r.name.clone()),
                (150, if r.won { "Won".into() } else { "Lost".into() }),
                (220, r.built.to_string()),
                (280, r.kills.to_string()),
                (340, r.losses.to_string()),
                (400, r.score.to_string()),
            ];
            for (cx, v) in values {
                d.text(x0 + cx, y, crate::font::fit(&v, 140), ink, outer);
            }
            y += 16;
        }
        let go = Rect::new(outer.right() - 130, outer.bottom() - 32, 118, 22);
        widget::button(
            &mut d,
            go,
            "Main menu",
            hover.is_some_and(|p| go.contains(p)),
            true,
        );
        t.push(go, ());
        (d, t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn end(local_won: bool) -> End {
        End {
            ticks: 900,
            units: 0,
            rows: vec![
                EndRow {
                    name: "Player".into(),
                    local: true,
                    won: local_won,
                    built: 4,
                    kills: 3,
                    losses: 1,
                    score: 50,
                },
                EndRow {
                    name: "Computer".into(),
                    local: false,
                    won: !local_won,
                    built: 5,
                    kills: 1,
                    losses: 3,
                    score: 20,
                },
            ],
        }
    }

    // spec: S-0414 claim 23 and Formulas (60 clock units of 16 ms make one displayed second, truncated; capped at 99:59:59)
    #[test]
    fn claim_23_duration() {
        let mut e = end(true);
        e.units = 59;
        assert_eq!(e.duration(), (0, 0, 0));
        e.units = 60;
        assert_eq!(e.duration(), (0, 0, 1));
        e.units = 60 * 3725;
        assert_eq!(e.duration(), (1, 2, 5));
        e.units = i64::MAX;
        assert_eq!(e.duration(), (99, 59, 59));
    }

    #[test]
    fn headline_follows_the_local_result() {
        // spec: S-0414 claim 7 (every player house has a result)
        assert_eq!(end(true).headline(), "VICTORY");
        assert_eq!(end(false).headline(), "DEFEAT");
        let mut e = end(true);
        e.rows[0].local = false;
        assert_eq!(e.headline(), "GAME OVER");
    }

    #[test]
    fn view_lists_rows_and_a_way_back() {
        let (d, t) = end(true).view((800, 600), None);
        let texts = d.texts();
        assert!(texts.contains(&"Computer"));
        assert!(texts.contains(&"Game time 0:00:00 (900 ticks)"));
        let r = t.list[0].0;
        assert_eq!(t.hit(r.centre()), Some(()));
    }
}
