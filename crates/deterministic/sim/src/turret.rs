//! Where a vehicle's turret points (S-0206 claims 8 and 9). The rows that
//! apply in a tick act one after another as Formulas F5 says: the target
//! rows of a vehicle with a target (combat::behave), then the spin row.

use glasnost_num::geom::direction;

use crate::rules::Kind;
use crate::world::{ObjectId, Pos, Target, World};

/// The spin step's target (S-0206 F2).
// spec: S-0206 F2 (f₂₅₆ = ⌊(f + 128) / 256⌋ mod 256; target = 256 × ((f₂₅₆ + 8) mod 256))
pub fn spin_target(f: u16) -> u16 {
    let f256 = ((u32::from(f) + 128) / 256) % 256;
    (256 * ((f256 + 8) % 256)) as u16
}

/// One tick of a vehicle's turret (S-0206 claim 9), after its movement.
pub fn tick(w: &mut World, id: ObjectId, facing_error: bool) {
    let t = w.tick;
    let r = w.rules_arc();
    let Some(o) = w.obj(id) else { return };
    if o.kind != Kind::Vehicle {
        return;
    }
    let Some(u) = r.unit(o.kind, o.type_index) else {
        return;
    };
    // spec: S-0206 claim 7 (the turret turns at the type's turn rate, the same as the body)
    let omega = u.turn_rate;
    // spec: S-0206 claim 9 and F5 (the target and facing-error rows, then the spin row, each from the facing the one before left)
    let mut targets: Vec<u16> = Vec::new();
    if u.turret && o.target.is_some() {
        // spec: S-0206 claim 9 (the target rows, with the firing check's facing error of this tick)
        targets = crate::combat::behave::turret_target_rows(w, id, facing_error);
    } else if u.turret && !u.turret_spins && o.target.is_none() {
        let g = r.general.guard_area_targeting_delay;
        // spec: S-0206 claim 9 (rest rows: not turning, at least G + 5 ticks since the vehicle last fired, not inside a tank bunker, S-0606 claim 9, not a deployed simple-deployer vehicle, S-0517; a spinning turret never uses them)
        let rested = !o.turret.turning(t)
            && t.wrapping_sub(o.last_fired) >= g.wrapping_add(5)
            && !crate::structures::bunker::bunkered(w, id)
            && !crate::econ::inplace::deployed(w, id);
        if rested {
            if crate::lift::seized(w, id) {
                // spec: S-0206 claim 9 (no change while the vehicle is seized, S-0607 claims 5 and 20)
            } else if let Some(dest) = o.dest {
                let p = match dest {
                    Target::Cell(c) => Pos::centre(c, 0),
                    Target::Object(x) => w.obj(x).map_or(o.pos, |d| d.pos),
                };
                // spec: S-0206 claim 9 (a movement destination: its direction, S-0204 claim 8)
                targets.push(direction(
                    i64::from(p.x - o.pos.x),
                    i64::from(p.y - o.pos.y),
                    r.quirks.table_direction,
                ));
            } else {
                // spec: S-0206 claim 9 (no movement destination: the body's current facing)
                targets.push(o.body.value(t));
            }
        }
    }
    let Some(ob) = w.obj_mut(id) else { return };
    for f in targets {
        // spec: S-0206 claim 3 and F5 (the same target again changes nothing; any other starts a turn from the facing the preceding row left)
        ob.turret.turn_to(f, t, omega);
    }
    if u.turret_spins && r.quirks.turret_spin_quantised {
        // spec: S-0206 claim 9 and F2 (a spinning turret: the facing the preceding rows left, rounded to a 256th of a turn, plus 2048; always the last row)
        let f = spin_target(ob.turret.value(t));
        ob.turret.turn_to(f, t, omega);
    } else if u.turret_spins && omega != 0 {
        // spec: S-0206 Edge cases (quirks.turret_spin_quantised off: the facing F2 reads plus 256 × min(ω, 8), unrounded; at ω = 0 no spin)
        let step = 256 * omega.min(8);
        let f = ob.turret.value(t).wrapping_add(step as u16);
        ob.turret.turn_to(f, t, omega);
    }
    if u.turret {
        // spec: S-0206 claim 17 (the rotation sample after all rows, the spin row included, by the current-tick turning definition)
        ob.rotation_sample = ob.turret.turning(t);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // spec: S-0206 F2 (the spin step: rounded to a 256th, plus 2048)
    #[test]
    fn spin_targets() {
        assert_eq!(spin_target(0), 2048);
        assert_eq!(spin_target(127), 2048);
        assert_eq!(spin_target(128), 2304);
        assert_eq!(spin_target(63488), 0);
        assert_eq!(spin_target(65535), 2048);
    }

    // spec: S-0206 F5 (several rows in one tick act one after another; the example of scenario R4 run f: τ₁ = 16383 gives s₁ = 3 with Θ₁ = 63 and β₁ = 260, then τ₂ = 0 completes at once)
    #[test]
    fn f5_rows_act_in_turn() {
        let t = 40; // any tick
        let mut f = crate::turning::Turning::at(0);
        f.turn_to(16383, t, 1);
        assert_eq!((f.duration, f.step), (63, 260));
        assert_eq!(f.value(t), 3);
        f.turn_to(0, t, 1);
        assert_eq!(f.value(t), 0);
        assert!(!f.turning(t));
        // The spin row after a facing-error row rounds the facing that row left.
        // With ω = 127 a turn of 32000 completes at once (Θ = 0), so the
        // spin row starts from 32000, not from s₀ = 0.
        let mut g = crate::turning::Turning::at(0);
        g.turn_to(32000, t, 127);
        let s1 = g.value(t);
        assert_eq!(s1, 32000);
        g.turn_to(spin_target(s1), t, 127);
        assert_eq!(g.target, 34048);
    }
}
