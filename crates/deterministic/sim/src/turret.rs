//! Where a vehicle's turret points (S-0206 claims 8 and 9), for the rows
//! that need no combat: the spinning turret and the two rest rows. The rows
//! for a vehicle with a target need the turret weapon's flags and the firing
//! check (combat, milestone M4); until then a vehicle with a target leaves
//! its turret as it is, which is the table's row for a vehicle without a
//! turret weapon.

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
    let Some(o) = w.obj(id).cloned() else { return };
    if o.kind != Kind::Vehicle {
        return;
    }
    let Some(u) = r.unit(o.kind, o.type_index) else {
        return;
    };
    // spec: S-0206 claim 7 (the turret turns at the type's turn rate, the same as the body)
    let omega = u.turn_rate;
    let target = if u.turret_spins {
        // spec: S-0206 claim 9 (a spinning turret: the current facing rounded to a 256th of a turn, plus 2048)
        Some(spin_target(o.turret.value(t)))
    } else if !u.turret {
        None
    } else if o.target.is_some() {
        // spec: S-0206 claim 9 (the target rows, with the firing check's facing error of this tick)
        crate::combat::behave::turret_target(w, id, facing_error)
    } else {
        let g = r.general.guard_area_targeting_delay;
        // spec: S-0206 claim 9 (rest rows: not turning, at least G + 5 ticks since the vehicle last fired)
        let rested = !o.turret.turning(t) && t.wrapping_sub(o.last_fired) >= g.wrapping_add(5);
        if !rested {
            None
        } else if let Some(dest) = o.dest {
            let p = match dest {
                Target::Cell(c) => Pos::centre(c, 0),
                Target::Object(x) => w.obj(x).map_or(o.pos, |d| d.pos),
            };
            // spec: S-0206 claim 9 (a movement destination: its direction, S-0204 claim 8)
            Some(direction(
                i64::from(p.x - o.pos.x),
                i64::from(p.y - o.pos.y),
                r.quirks.table_direction,
            ))
        } else {
            // spec: S-0206 claim 9 (no movement destination: the body's current facing)
            Some(o.body.value(t))
        }
    };
    if let Some(f) = target
        && let Some(ob) = w.obj_mut(id)
    {
        // spec: S-0206 claim 3 (the same target again changes nothing)
        ob.turret.turn_to(f, t, omega);
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
}
