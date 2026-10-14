//! The end screen: every player's result (S-0414 claim 7) and the counts
//! the world keeps.

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
    /// Objects built: the sum of the ever-built counts (S-0407 Terms).
    pub built: i32,
    /// Objects destroyed, credited (S-0302 claim 16).
    pub kills: i32,
    /// Objects lost.
    pub losses: i32,
    /// Score from kill values (S-0311 claim 7).
    pub score: i32,
}

/// The end screen's model.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct End {
    /// The tick in which the game is over (S-0414 claims 8 and 9).
    pub ticks: i32,
    /// The game time in milliseconds at the game's paced period.
    pub millis: i64,
    /// The rows in house order (S-0414 claim 7).
    pub rows: Vec<EndRow>,
}

impl End {
    /// Reads the end screen from the world and its outcome. `period_ms` is
    /// the paced period of one tick (S-0201 claim 4).
    pub fn from_world(
        w: &World,
        outcome: &Outcome,
        local: Option<HouseId>,
        period_ms: i64,
    ) -> Self {
        let rows = outcome
            .results
            .iter()
            .map(|&(h, v)| {
                let house = &w.houses[h.0 as usize];
                EndRow {
                    name: house.name.clone(),
                    local: Some(h) == local,
                    won: v == Verdict::Win,
                    built: house.econ.tech.ever_built.values().sum(),
                    kills: house.combat.kills,
                    losses: house.combat.losses,
                    score: house.combat.score,
                }
            })
            .collect();
        Self {
            ticks: outcome.over,
            millis: i64::from(outcome.over.max(0)) * period_ms,
            rows,
        }
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
        let secs = self.millis / 1000;
        let time = format!(
            "Game time {}:{:02} ({} ticks)",
            secs / 60,
            secs % 60,
            self.ticks
        );
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
            millis: 900 * 32,
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
        assert!(texts.contains(&"Game time 0:28 (900 ticks)"));
        let r = t.list[0].0;
        assert_eq!(t.hit(r.centre()), Some(()));
    }
}
