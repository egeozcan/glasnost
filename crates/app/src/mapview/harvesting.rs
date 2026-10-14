//! The vehicle harvester's harvesting sprite (S-0402 claim 5a, F8): a fixed
//! game-wide sprite drawn under a loading harvester's body, its image frame
//! and its anchor. When it shows is the simulation's
//! ([`glasnost_sim::econ::harvest::display_shown`]); the drawing is
//! [`super::build`]'s.

use std::sync::OnceLock;

use glasnost_num::geom::n_step;
use glasnost_num::{F64, Rounding, trig};
use glasnost_sim::econ::harvest;
use glasnost_sim::world::{ObjectId, World};

/// The harvesting sprite, loaded by its fixed file name with the ordinary
/// asset precedence (S-0005 claim 1).
pub const SPRITE_FILE: &[u8] = b"OREGATH.SHP"; // spec: S-0402 Import mapping (fixed game asset OREGATH.SHP)

/// Facing groups of the sprite.
const FACING_GROUPS: u32 = 8; // spec: S-0402 Constants (harvesting sprite facing groups: 8)

/// Image frames per facing group.
const FRAMES_PER_FACING: i64 = 15; // spec: S-0402 Constants (harvesting sprite frames per facing: 15)

/// The anchor's displacement from the vehicle's position, leptons.
const DISPLACEMENT: i64 = 30; // spec: S-0402 Constants (harvesting sprite anchor displacement: 30 leptons)

/// The harvesting image to draw for one vehicle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Harvesting {
    /// Image frame of the sprite (F8).
    pub frame: usize,
    /// World point the image is centred on, before projection (F8).
    pub anchor: (i32, i32, i32),
}

/// The image frame of F8 for 8-step body facing `q` (0 … 7), tick number
/// `t` and walk-animation counter `nu`: 15(7 − q) + ((t + ν) mod 15).
pub fn frame(q: u32, t: i32, nu: i32) -> i64 {
    let q = i64::from(q.min(FACING_GROUPS - 1));
    // spec: S-0402 F8 (15(7 − q) + ((t + ν) mod 15), integer arithmetic with a non-negative remainder)
    FRAMES_PER_FACING * (i64::from(FACING_GROUPS) - 1 - q)
        + (i64::from(t) + i64::from(nu)).rem_euclid(FRAMES_PER_FACING)
}

/// The offsets RZ₅₃(30 × cosT θ) and RZ₅₃(30 × sinT θ) of F8 for 8-step
/// facing `q`, with `quirks.table_direction` on (`table`) or off.
pub fn offset(q: u32, table: bool) -> (F64, F64) {
    static TABLES: OnceLock<[[(F64, F64); 8]; 2]> = OnceLock::new();
    let t = TABLES.get_or_init(|| {
        let mut out = [[(F64::ZERO, F64::ZERO); 8]; 2];
        for (i, row) in out.iter_mut().enumerate() {
            for (q, cell) in row.iter_mut().enumerate() {
                *cell = compute_offset(q as u32, i == 1);
            }
        }
        out
    });
    t[usize::from(table)][(q % FACING_GROUPS) as usize]
}

/// [`offset`], computed.
fn compute_offset(q: u32, table: bool) -> (F64, F64) {
    // spec: S-0402 F8 (products and sums round toward zero)
    let rz = Rounding::TowardZero;
    // spec: S-0402 F8 (f̂: 8192q reduced to the signed facing range −32768 … 32767)
    let f = 8192 * i64::from(q);
    let fh = if f <= 32767 { f } else { f - 65536 };
    let theta = if table {
        // spec: S-0402 F8 (θ = RZ₅₃((f̂ − 16383) × c), c the binary64 value nearest −2π / 65534)
        F64::from_i64(fh - 16383, rz).mul(glasnost_sim::consts::WALK_C2, rz)
    } else {
        // spec: S-0402 F8 (`quirks.table_direction` off: 16384 and the binary64 value nearest −2π / 65536)
        F64::from_i64(fh - 16384, rz).mul(glasnost_sim::consts::WALK_C, rz)
    };
    // spec: S-0402 F8 (the table sine and cosine of S-0105 F1 required by S-0205 claim 6, with the table's
    // negative-angle quirk; single-precision results widened exactly)
    let (c, s) = (trig::cos_t(theta, rz), trig::sin_t(theta, rz));
    let d = F64::from_i64(DISPLACEMENT, rz);
    // spec: S-0402 F8 (the cosine contributes to X and the sine to Y)
    (d.mul(c, rz), d.mul(s, rz))
}

/// The anchor of F8 for a vehicle at `pos` (leptons) with 8-step facing
/// `q`: Xₐ = trunc(RZ₅₃(X + RZ₅₃(30 cosT θ))), Yₐ = trunc(RZ₅₃(Y −
/// RZ₅₃(30 sinT θ))), Zₐ = Z.
pub fn anchor(pos: (i32, i32, i32), q: u32, table: bool) -> (i32, i32, i32) {
    let rz = Rounding::TowardZero;
    let (dx, dy) = offset(q, table);
    // spec: S-0402 F8 (both coordinate conversions truncate toward zero; Zₐ = Z)
    let x = F64::from_i64(i64::from(pos.0), rz)
        .add(dx, rz)
        .to_i32_common();
    let y = F64::from_i64(i64::from(pos.1), rz)
        .sub(dy, rz)
        .to_i32_common();
    (x, y, pos.2)
}

/// The harvesting image of object `id` now, when it shows one (S-0402
/// claim 5a). `nu` is its walk-animation counter (S-0105 claim 16).
pub fn of(w: &World, id: ObjectId, nu: i32) -> Option<Harvesting> {
    if !harvest::display_shown(w, id) {
        return None;
    }
    let o = w.obj(id)?;
    // spec: S-0402 F8 (q: the body's 8-step facing, S-0204 claim 10 and F4)
    let q = n_step(o.body.value(w.tick), FACING_GROUPS);
    // spec: S-0402 F8 (t: the current tick number). A drawn frame reads
    // the simulation's current tick, the same one the body facing is read at.
    let frame = usize::try_from(frame(q, w.tick, nu)).ok()?;
    let table = w.rules().quirks.table_direction;
    Some(Harvesting {
        frame,
        anchor: anchor((o.pos.x, o.pos.y, o.pos.z), q, table),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // spec: S-0402 F8 (15(7 − q) + ((t + ν) mod 15): facing group, phase by tick and walk-animation counter)
    #[test]
    fn f8_frame_formula() {
        assert_eq!(frame(0, 0, 0), 105);
        assert_eq!(frame(7, 0, 0), 0);
        assert_eq!(frame(3, 17, 0), 60 + 2);
        assert_eq!(frame(2, 10, 9), 75 + 4);
        assert_eq!(frame(5, 0, 14), 30 + 14);
        // A negative sum takes the non-negative remainder.
        assert_eq!(frame(1, -1, 0), 90 + 14);
        // Every frame lies in the facing group's 15 frames.
        for q in 0..8 {
            for t in 0..40 {
                let f = frame(q, t, 3);
                let base = 15 * (7 - i64::from(q));
                assert!((base..base + 15).contains(&f), "q {q} t {t}: {f}");
            }
        }
    }

    // spec: S-0402 claim 5a and F8 (advancing every tick and repeating after 15 ticks at a constant walk-animation counter)
    #[test]
    fn f8_frame_repeats_after_15_ticks() {
        for q in 0..8 {
            for nu in [0, 4, 29] {
                for t in 100..130 {
                    assert_eq!(frame(q, t, nu), frame(q, t + 15, nu));
                    if (t + nu + 1) % 15 != 0 {
                        assert_eq!(frame(q, t + 1, nu), frame(q, t, nu) + 1);
                    }
                }
            }
        }
        // A changed walk-animation counter moves the phase.
        assert_eq!(frame(4, 20, 1), frame(4, 21, 0));
    }

    // spec: S-0402 F8 (X + 30 × cosT θ, Y − 30 × sinT θ in round-toward-zero doubles: the same arithmetic as
    // S-0208 F3's step with s = 30 from facing 8192q), for both `quirks.table_direction` values
    #[test]
    fn f8_anchor_is_the_table_step_of_30_leptons() {
        let rz = Rounding::TowardZero;
        for table in [true, false] {
            for (x, y, z) in [(3000, 2000, 104), (40, 31, 0), (25_611, 9_999, 416)] {
                for q in 0..8 {
                    let (sx, sy) =
                        glasnost_sim::walk::step_point(x, y, (8192 * q) as u16, 30, table, rz);
                    assert_eq!(anchor((x, y, z), q, table), (sx, sy, z), "q {q} {table}");
                }
            }
        }
    }

    // spec: S-0402 F8 (the cosine contributes to X and the sine to Y: facing 16384, map east (S-0402
    // Constants), moves the anchor 30 leptons along +X or just short of it, not along Y)
    #[test]
    fn f8_cosine_on_x_and_sine_on_y() {
        let (x, y) = (3000, 2000);
        for table in [true, false] {
            let (ax, ay, _) = anchor((x, y, 0), 2, table);
            assert!((29..=30).contains(&(ax - x)), "{table}: {ax}");
            assert!((ay - y).abs() <= 1, "{table}: {ay}");
            // The opposite facing goes the other way along X.
            let (bx, by, _) = anchor((x, y, 0), 6, table);
            assert!((-30..=-29).contains(&(bx - x)), "{table}: {bx}");
            assert!((by - y).abs() <= 1, "{table}: {by}");
            // Facing 0 moves along Y, not X.
            let (cx, cy, _) = anchor((x, y, 0), 0, table);
            assert!(
                (cx - x).abs() <= 1 && (cy - y).abs() >= 29,
                "{table}: {cx} {cy}"
            );
        }
    }

    // spec: S-0402 F8 (the table sine and cosine, not exact ones: every offset is 30 times a table value)
    #[test]
    fn f8_offsets_use_the_table_values() {
        let rz = Rounding::TowardZero;
        let d = F64::from_i64(30, rz);
        for table in [true, false] {
            for q in 0..8u32 {
                let f = 8192 * i64::from(q);
                let fh = if f <= 32767 { f } else { f - 65536 };
                let theta = if table {
                    F64::from_i64(fh - 16383, rz).mul(glasnost_sim::consts::WALK_C2, rz)
                } else {
                    F64::from_i64(fh - 16384, rz).mul(glasnost_sim::consts::WALK_C, rz)
                };
                let (dx, dy) = offset(q, table);
                assert!(dx.eq_num(d.mul(trig::cos_t(theta, rz), rz)), "q {q}");
                assert!(dy.eq_num(d.mul(trig::sin_t(theta, rz), rz)), "q {q}");
            }
        }
    }

    /// A game with one loading vehicle harvester (type HARVX, invented)
    /// facing `facing`, its harvest state set directly.
    fn loading_harvester(facing: u16) -> (World, ObjectId) {
        use glasnost_sim::econ::harvest::{HarvestState, Phase};
        use glasnost_sim::rules::{Behaviour, Kind};
        use glasnost_sim::testkit;
        let mut r = testkit::ruleset();
        r.registries
            .get_mut("vehicles")
            .unwrap()
            .push("HARVX".into());
        let mut u = r.sim.vehicles["CART"].clone();
        u.primary = None;
        u.harvester = Some(true);
        r.sim.vehicles.insert("HARVX".into(), u);
        let mut w = testkit::game_with(&r, 20, 1);
        let id = testkit::spawn(
            &mut w,
            Kind::Vehicle,
            "HARVX",
            0,
            testkit::mid(20, 0, 0),
            facing,
        );
        let t = w.tick;
        let o = w.obj_mut(id).unwrap();
        o.beh.force(Behaviour::Harvest, t, true);
        o.econ.harvest = Some(HarvestState {
            phase: Phase::Loading,
            harvest_start: Some(o.beh.start_tick),
            ..Default::default()
        });
        (w, id)
    }

    // spec: S-0402 claim 5a and F8 (a loading harvester's image: F8's frame of its facing group at the current
    // tick and walk-animation counter, and the anchor of its facing)
    #[test]
    fn claim_5a_f8_image_of_a_loading_harvester() {
        let (w, id) = loading_harvester(16384);
        let table = w.rules().quirks.table_direction;
        let o = w.obj(id).unwrap();
        let pos = (o.pos.x, o.pos.y, o.pos.z);
        let h = of(&w, id, 4).expect("shown");
        // 16384 is 8-step facing 2.
        assert_eq!(h.frame as i64, frame(2, w.tick, 4));
        assert_eq!(h.anchor, anchor(pos, 2, table));
    }

    // spec: S-0402 claim 5a (suppressed while the body turns; on stopping its frame follows F8 at the current
    // tick, not a fresh frame 0)
    #[test]
    fn claim_5a_resumes_without_restarting_its_cycle() {
        let (mut w, id) = loading_harvester(0);
        let t = w.tick;
        assert!(w.obj_mut(id).unwrap().body.turn_to(8192, t, 6));
        assert_eq!(of(&w, id, 0), None);
        // Some ticks later the turn is over: 8-step facing 1.
        w.tick += 47;
        let h = of(&w, id, 0).expect("shown again");
        assert_eq!(h.frame as i64, frame(1, w.tick, 0));
        assert_eq!(h.frame, 90 + ((t + 47) % 15) as usize);
    }
}
