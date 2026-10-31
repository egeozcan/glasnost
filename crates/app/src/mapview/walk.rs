//! The vehicles' walk counters (S-0102 claims 22 and 22a; S-0105 claim 16),
//! kept by the view beside the game.
//!
//! S-0102 claim 22a (Q-2176): each vehicle, drawn as a sprite or from a
//! voxel model, has one walk counter, 0 at creation, advanced at most once
//! per tick in its own turn. It picks the walking image frame of a sprite
//! vehicle (claim 22), the body frame ν of a voxel vehicle (S-0105 claim 16)
//! and the harvesting image (S-0402 F8). No rule reads it, so the view keeps
//! it, advanced once per game tick (never per drawn frame) from the state
//! the tick left (claim 22a *Reading time*). As the claim says for our
//! engine, it is outside the game state: not saved, not hashed, and a save,
//! a load or a joined network game starts every counter again at 0.

use std::collections::BTreeMap;

use glasnost_sim::rules::Kind as SimKind;
use glasnost_sim::world::{ObjectId, World};

/// A vehicle type's walk-counter data: S-0102 Import mapping `WalkRate`,
/// `IdleRate`, `HoverAttack`, `DeployToLand`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rates {
    /// The walk rate *R*<sub>w</sub>.
    pub walk: i32,
    /// The idle rate *R*<sub>i</sub>; 0 never steps a vehicle that is not moving now.
    pub idle: i32,
    /// A hover-attack type (test (a)'s alternative).
    pub hover_attack: bool,
    /// The deploy-to-land flag (test (c)).
    pub deploy_to_land: bool,
}

impl Default for Rates {
    fn default() -> Self {
        Self {
            walk: 1,               // spec: S-0102 Import mapping (WalkRate absent: 1)
            idle: 0,               // spec: S-0102 Import mapping (IdleRate absent: 0)
            hover_attack: false,   // spec: S-0102 Import mapping (HoverAttack absent: no)
            deploy_to_land: false, // spec: S-0102 Import mapping (DeployToLand absent: no)
        }
    }
}

/// What the drawing reads of a vehicle's walking: its walk counter and
/// whether it has movement in progress in the shown tick.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Walk {
    /// The walk counter *w* (S-0102 claim 22a), the walk-animation counter
    /// ν of S-0105 claim 16.
    pub counter: u32,
    /// Has movement in progress (S-0107 Terms): S-0102 claim 22's walking
    /// row test (not *moving now*).
    pub in_progress: bool,
}

/// What a tick's test of a vehicle's counter reads of the vehicle (claim
/// 22a *The advance*).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Situation {
    /// Moving now, by its locomotor's spec.
    pub moving_now: bool,
    /// Holds a target, is not deployed and is of a hover-attack type:
    /// test (a)'s alternative to moving now.
    pub hover_attack_active: bool,
    /// A deploy-to-land type with its height above the surface above 0
    /// (test (c)).
    pub airborne_lander: bool,
}

/// Whether `tick` is a multiple of `rate`; the multiples of a rate are
/// those of its absolute value, and a rate of 0 is used as 1.
fn multiple(tick: i32, rate: i32) -> bool {
    // spec: S-0102 claim 22a (Rates: the multiples of a rate are those of its absolute value; a walk rate of 0 is treated as 1)
    let r = rate.unsigned_abs().max(1);
    tick.unsigned_abs().is_multiple_of(r)
}

/// One tick of a counter (S-0102 claim 22a *The advance*): the tick number
/// `tick` (0 while the first tick runs), the vehicle's situation, its
/// rates.
pub fn step(counter: u32, tick: i32, s: Situation, rates: Rates) -> u32 {
    // spec: S-0102 claim 22a (a: tick a multiple of the walk rate, and moving now or the hover-attack alternative)
    let a = multiple(tick, rates.walk) && (s.moving_now || s.hover_attack_active);
    // spec: S-0102 claim 22a (b: not moving now, idle rate not 0, tick a multiple of it)
    let b = !s.moving_now && rates.idle != 0 && multiple(tick, rates.idle);
    // spec: S-0102 claim 22a (c: deploy-to-land type above the surface, every tick)
    let c = s.airborne_lander;
    if a || b || c {
        counter.wrapping_add(1)
    } else {
        counter
    }
}

/// Whether a vehicle's counter is held at this tick's test (S-0102 claim
/// 22a *No advance*), for the states the simulation answers.
pub fn held(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return true };
    // spec: S-0102 claim 22a (No advance: off the map, which covers a passenger of a transport and a vehicle carried by a carrier aircraft)
    !o.on_map
        // spec: S-0102 claim 22a (No advance: a dropped object still falling)
        || glasnost_sim::fall::falling(w, id)
        // spec: S-0102 claim 22a (No advance: a sinking vessel)
        || glasnost_sim::naval::sinking(w, id)
        // spec: S-0102 claim 22a (No advance: being erased or phasing in)
        || glasnost_sim::superweapons::being_erased(w, id)
        || glasnost_sim::teleport::phasing_in(w, id)
        // spec: S-0102 claim 22a (No advance: seized, until a restoration clears it)
        || glasnost_sim::lift::seized(w, id)
        // spec: S-0102 claim 22a (No advance: a chronosphere warp member from its first pending update; our engine's design rule)
        || glasnost_sim::superweapons::warp_pending_or_leaving(w, id)
    // seam: melee-linked without the keep-acting flag, and travel through a tunnel (S-0102 claim 22a): the simulation keeps no answer the view can read
}

/// What the test reads of vehicle `id` in the world the tick left.
fn situation(w: &World, id: ObjectId, rates: Rates) -> Situation {
    let Some(o) = w.obj(id) else {
        return Situation::default();
    };
    Situation {
        // spec: S-0102 claim 22a (Moving now: the locomotor's own answer)
        moving_now: glasnost_sim::occupancy::locomotor_moving_now(w, id),
        // spec: S-0102 claim 22a (a: holds a target, not deployed (S-0517), hover-attack type)
        hover_attack_active: rates.hover_attack
            && o.target.is_some()
            && !glasnost_sim::econ::inplace::deployed(w, id),
        // spec: S-0102 claim 22a (c: deploy-to-land flag and height above the surface above 0)
        airborne_lander: rates.deploy_to_land && glasnost_sim::jump::height_above(w, id) > 0,
    }
}

/// The walk counters of every vehicle in play.
#[derive(Clone, Debug, Default)]
pub struct Counters {
    walks: BTreeMap<ObjectId, Walk>,
}

impl Counters {
    /// Advances every vehicle's counter by the tick the world has just
    /// completed (its tick number is `w.tick − 1`), reading each type's
    /// rates from `rates` (by vehicle type index; a missing entry takes the
    /// absent-key defaults). A held vehicle keeps its value, so a passenger
    /// goes on from it when unloaded (claim 22a *Leaving and entering
    /// play*); vehicles that no longer exist are forgotten.
    pub fn advance(&mut self, w: &World, rates: &[Rates]) {
        // spec: S-0102 claim 22a and glossary (the tick number: 0 while the first tick runs)
        let tick = w.tick - 1;
        let mut next = BTreeMap::new();
        for o in w.objects.values() {
            if o.kind != SimKind::Vehicle {
                continue;
            }
            let r = rates
                .get(o.type_index as usize)
                .copied()
                .unwrap_or_default();
            let was = self.walks.get(&o.id).copied().unwrap_or_default();
            let counter = if held(w, o.id) {
                was.counter
            } else {
                step(was.counter, tick, situation(w, o.id, r), r)
            };
            next.insert(
                o.id,
                Walk {
                    counter,
                    // spec: S-0102 claim 22 (the walking row tests has movement in progress, S-0107 Terms)
                    in_progress: o.on_map && glasnost_sim::occupancy::in_progress(w, o.id),
                },
            );
        }
        self.walks = next;
    }

    /// The walk of vehicle `id` (0 and standing for one never counted).
    pub fn of(&self, id: ObjectId) -> Walk {
        self.walks.get(&id).copied().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MOVING: Situation = Situation {
        moving_now: true,
        hover_attack_active: false,
        airborne_lander: false,
    };
    const STILL: Situation = Situation {
        moving_now: false,
        hover_attack_active: false,
        airborne_lander: false,
    };

    fn rates(walk: i32, idle: i32) -> Rates {
        Rates {
            walk,
            idle,
            ..Rates::default()
        }
    }

    // spec: S-0102 claim 22a and D48 (a) (steps on tick numbers divisible by the walk rate while moving now, then holds)
    #[test]
    fn moving_steps_on_multiples_of_the_walk_rate() {
        let r = rates(3, 0);
        let mut c = 0;
        let mut grew = Vec::new();
        for t in 0..10 {
            let n = step(c, t, MOVING, r);
            if n != c {
                grew.push(t);
            }
            c = n;
        }
        assert_eq!(grew, [0, 3, 6, 9]);
        // The default walk rate steps every tick.
        assert_eq!(step(5, 7, MOVING, Rates::default()), 6);
        // Once it stops, the counter holds (idle rate 0).
        assert_eq!(step(c, 12, STILL, r), c);
    }

    // spec: S-0102 claim 22a and D48 (c) (not moving now: idle rate 0 holds; a positive idle rate steps on its multiples)
    #[test]
    fn standing_steps_only_with_an_idle_rate() {
        for t in 0..20 {
            assert_eq!(step(4, t, STILL, rates(1, 0)), 4);
            assert_eq!(
                step(4, t, STILL, rates(1, 5)),
                if t % 5 == 0 { 5 } else { 4 }
            );
        }
    }

    // spec: S-0102 claim 22a (b: moving now excludes the idle step; a: the walk rate alone decides while moving)
    #[test]
    fn moving_ignores_the_idle_rate() {
        let r = rates(3, 5);
        assert_eq!(step(0, 5, MOVING, r), 0);
        assert_eq!(step(0, 3, MOVING, r), 1);
    }

    // spec: S-0102 claim 22a (a: the hover-attack alternative steps at the walk rate without moving now; b still applies)
    #[test]
    fn a_hover_attack_vehicle_holding_a_target_steps_at_the_walk_rate() {
        let s = Situation {
            hover_attack_active: true,
            ..STILL
        };
        let r = rates(3, 0);
        assert_eq!(step(0, 3, s, r), 1);
        assert_eq!(step(0, 4, s, r), 0);
        // Without the three conditions it holds.
        assert_eq!(step(0, 3, STILL, r), 0);
    }

    // spec: S-0102 claim 22a (c: a deploy-to-land vehicle above the surface steps on every tick, moving or not)
    #[test]
    fn a_deploy_to_land_vehicle_in_the_air_steps_every_tick() {
        let s = Situation {
            airborne_lander: true,
            ..STILL
        };
        for t in 0..7 {
            assert_eq!(step(2, t, s, rates(3, 0)), 3);
        }
    }

    // spec: S-0102 claim 22a (Rates: multiples of the absolute value; a walk rate of 0 is used as 1 by our engine)
    #[test]
    fn rates_use_their_absolute_value_and_a_zero_walk_rate_is_one() {
        assert_eq!(step(0, 6, MOVING, rates(-3, 0)), 1);
        assert_eq!(step(0, 4, MOVING, rates(-3, 0)), 0);
        for t in 0..5 {
            assert_eq!(step(0, t, MOVING, rates(0, 0)), 1);
        }
        assert_eq!(step(0, 10, STILL, rates(1, -5)), 1);
        assert_eq!(step(0, 7, STILL, rates(1, -5)), 0);
    }

    // spec: S-0102 claim 22 (frame table: the walking row while the vehicle has movement in progress or its idle rate is not 0, S_w + d·F_w + (w mod F_w); the standing row otherwise)
    #[test]
    fn a_sprite_vehicle_walks_through_its_row_while_it_has_movement_in_progress() {
        use super::super::build::{SpriteVehicleRows, sprite_vehicle_frame};
        let rows = SpriteVehicleRows {
            walk_start: 0,
            walk_frames: 4,
            stand_start: 40,
            standing_frames: 2,
        };
        let walk = |counter, in_progress| Walk {
            counter,
            in_progress,
        };
        let frames: Vec<i32> = (0..6)
            .map(|w| sprite_vehicle_frame(rows, 3, walk(w, true), 0))
            .collect();
        assert_eq!(frames, [12, 13, 14, 15, 12, 13]);
        // No movement in progress: the standing row, whatever the counter.
        assert_eq!(sprite_vehicle_frame(rows, 3, walk(5, false), 0), 46);
        // A vehicle waiting at a blocked cell still has movement in progress: the walking row, held.
        assert_eq!(sprite_vehicle_frame(rows, 3, walk(5, true), 0), 13);
        // Standing with an idle rate: the walking row with the counter.
        assert_eq!(sprite_vehicle_frame(rows, 3, walk(5, false), 2), 13);
        // No standing frames: the walking row's first frame.
        let bare = SpriteVehicleRows {
            standing_frames: 0,
            ..rows
        };
        assert_eq!(sprite_vehicle_frame(bare, 3, walk(5, false), 0), 12);
    }
}
