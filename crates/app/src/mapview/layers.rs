//! The render layers of the mobile objects (S-0102 claims 3, 4 and 7):
//! which layer the table of *State and ordering* gives an object, the
//! moments at which the drawing evaluates it, and the insertion-ordered
//! lists of the layers other than ground.
//!
//! All of this is presentation state of the game client: it reads the
//! simulation and the simulation never reads it.

use std::collections::BTreeMap;

use glasnost_sim::jump::{self, Phase};
use glasnost_sim::rules::Locomotor;
use glasnost_sim::world::{ObjectId, World};

pub use super::slots::Layer;

/// The bridge height of the jumpjet two-height rule: *r*′ = *r* − 416
/// leptons over a high bridge (S-0102 *State and ordering*).
const BRIDGE_HEIGHT: i32 = 416; // spec: S-0102 State and ordering (r′ = r − 416 leptons)

/// The raw height below which a unit with jumpjet movement evaluates to
/// ground (S-0102 *State and ordering*).
const JUMPJET_GROUND_HEIGHT: i32 = 208; // spec: S-0102 State and ordering (r < 208 leptons: ground)

/// The layer S-0102's table gives mobile object `id` for its state and
/// height now (claim 3, *State and ordering*), `None` for an object that
/// is not a mobile object on the map.
pub fn evaluate(w: &World, id: ObjectId) -> Option<Layer> {
    let o = w.obj(id)?;
    if !o.kind.is_mobile() || !o.on_map {
        return None;
    }
    let loco = w.rules().unit(o.kind, o.type_index).map(|u| u.locomotor);
    // spec: S-0102 State and ordering (a unit with rocket movement is always in the air layer)
    if o.rocket.is_some() || loco == Some(Locomotor::Rocket) {
        return Some(Layer::Air);
    }
    // spec: S-0102 State and ordering (the layer follows the movement type in use; a seized unit moves with the jumpjet locomotor, S-0607 claim 5)
    if jump::is_jumpjet(w, id) {
        return Some(jumpjet_layer(w, id));
    }
    if o.kind == glasnost_sim::rules::Kind::Aircraft || loco == Some(Locomotor::Fly) {
        // spec: S-0102 State and ordering and S-0501 claim 7 (flying movement: top at any positive height above the surface, ground at 0 or below)
        return Some(if w.height_above_surface(id) > 0 {
            Layer::Top
        } else {
            Layer::Ground
        });
    }
    // spec: S-0102 State and ordering (walking, driving, hovering, ship, mech and teleport movement: always ground; tunnel and drop-pod movement are not modelled)
    Some(Layer::Ground)
}

/// The two-height rule of a unit with jumpjet movement (S-0102 *State and
/// ordering*).
fn jumpjet_layer(w: &World, id: ObjectId) -> Layer {
    let Some(o) = w.obj(id) else {
        return Layer::Ground;
    };
    let on_deck = o.jump.as_ref().is_some_and(|j| j.on_deck);
    // spec: S-0102 State and ordering (r: the height above the surface, above the deck with the on-deck state set)
    let r = w.height_above_surface(id);
    // spec: S-0102 State and ordering (r′ = r − 416 only over a high-bridge cell, with on-deck state unset, r ≥ 416 and no bridge/parachute falling state; otherwise r′ = r)
    let adjusted = if w.high_bridge(o.pos.cell())
        && !on_deck
        && r >= BRIDGE_HEIGHT
        && !glasnost_sim::fall::falling(w, id)
    {
        r - BRIDGE_HEIGHT
    } else {
        r
    };
    // spec: S-0102 State and ordering (ground when r < 208 or r′ = 0; the 208 test reads r)
    if r < JUMPJET_GROUND_HEIGHT || adjusted == 0 {
        return Layer::Ground;
    }
    // spec: S-0102 State and ordering (the type's jumpjet cruise height after its minimum, S-0502 claim 1)
    let cruise = jump::fields(w, id).map_or(0, |f| f.height);
    // spec: S-0102 State and ordering (top at or above the jumpjet cruise height, compared with r′; air below it)
    if adjusted >= cruise {
        Layer::Top
    } else {
        Layer::Air
    }
}

/// What the drawing remembers of a mobile object between ticks, to see
/// claim 4's moments.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Track {
    /// The layer it was last put into.
    pub layer: Layer,
    /// A flying object's height above the surface was positive.
    airborne: bool,
    /// A jumpjet's state.
    phase: Option<Phase>,
}

/// The moment state of `id` now.
fn moment_state(w: &World, id: ObjectId) -> (bool, Option<Phase>) {
    let flying = w.obj(id).is_some_and(|o| {
        o.kind == glasnost_sim::rules::Kind::Aircraft
            || w.rules()
                .unit(o.kind, o.type_index)
                .is_some_and(|u| u.locomotor == Locomotor::Fly)
    });
    (flying && w.height_above_surface(id) > 0, jump::phase(w, id))
}

/// The render layers' membership of the mobile objects, the animations,
/// the particles and the static layered entries (S-0102 claims 3, 4 and 7).
///
/// The ground layer itself is [`super::live::GroundLayer`], sorted in
/// every drawn frame (claim 6); this keeps which mobile objects are in it
/// and the lists of the other four layers, each in entering order.
#[derive(Clone, Debug, Default)]
pub struct Layers {
    /// Each mobile object's layer and moment state.
    tracks: BTreeMap<ObjectId, Track>,
    /// The underground, surface, air and top lists, in entering order.
    lists: BTreeMap<Layer, Vec<super::live::Entrant>>,
}

impl Layers {
    /// The layer mobile object `id` was last put into.
    pub fn of(&self, id: ObjectId) -> Option<Layer> {
        self.tracks.get(&id).map(|t| t.layer)
    }

    /// The list of a layer other than ground, in entering order.
    pub fn list(&self, layer: Layer) -> &[super::live::Entrant] {
        self.lists.get(&layer).map_or(&[], Vec::as_slice)
    }

    /// The layers other than ground that hold something, in drawing order.
    pub fn upper(&self) -> impl Iterator<Item = Layer> + '_ {
        self.lists
            .iter()
            .filter(|(l, v)| **l != Layer::Ground && !v.is_empty())
            .map(|(l, _)| *l)
    }

    /// Puts `e` into `layer`: the end of that list for a layer other than
    /// ground (claims 4 and 7), after taking it out of every list.
    pub fn place(&mut self, e: super::live::Entrant, layer: Layer) {
        for v in self.lists.values_mut() {
            v.retain(|x| *x != e);
        }
        if layer != Layer::Ground {
            // spec: S-0102 claims 4 and 7 (an object entering a layer other than ground becomes the last of its list)
            self.lists.entry(layer).or_default().push(e);
        }
    }

    /// Takes `e` out of every list.
    pub fn remove(&mut self, e: super::live::Entrant) {
        for v in self.lists.values_mut() {
            v.retain(|x| *x != e);
        }
    }

    /// Keeps only the entrants for which `keep` holds.
    pub fn retain(&mut self, keep: impl Fn(super::live::Entrant) -> bool) {
        for v in self.lists.values_mut() {
            v.retain(|x| keep(*x));
        }
    }

    /// Whether `e` is in a list of a layer other than ground.
    pub fn contains(&self, e: super::live::Entrant) -> bool {
        self.lists.values().any(|v| v.contains(&e))
    }

    /// Sorts every list other than ground stably by `key` (the alternative
    /// of `quirks.unsorted_upper_layers`, S-0102 Edge cases).
    pub fn sort_by_key(&mut self, key: impl Fn(super::live::Entrant) -> i64) {
        for v in self.lists.values_mut() {
            v.sort_by_key(|e| key(*e));
        }
    }

    /// Evaluates the mobile objects at claim 4's moments after a tick (or
    /// at the start, `tick` false: placement only). Objects that left the
    /// map leave their layer. Objects are taken in the update order
    /// (S-0203 claim 3), the order of their turns in the tick.
    ///
    /// - **Placement** (an object the drawing has not seen on the map):
    ///   always.
    /// - **Take-off and landing:** a flying object whose height above the
    ///   surface turns positive or returns to 0 or below (S-0501 claim 7);
    ///   a jumpjet leaving or entering its grounded state (S-0502 claims 5
    ///   and 15).
    /// - **Falling:** every tick while it falls, changed only when the
    ///   layer differs.
    /// - With `events_only` off (`quirks.layer_change_on_events_only`):
    ///   every tick, changed only when the layer differs.
    ///
    /// Returns the objects that were put into a layer, in that order, with
    /// the layer.
    pub fn evaluate_mobiles(
        &mut self,
        w: &World,
        tick: bool,
        events_only: bool,
    ) -> Vec<(ObjectId, Layer)> {
        let on_map = |id: ObjectId| w.obj(id).is_some_and(|o| o.kind.is_mobile() && o.on_map);
        let gone: Vec<ObjectId> = self
            .tracks
            .keys()
            .copied()
            .filter(|&id| !on_map(id))
            .collect();
        for id in gone {
            self.tracks.remove(&id);
            self.remove(super::live::Entrant::Object(id));
        }
        // Objects in their turn order; objects on the map that are not in
        // play (none in practice) after them, by ID.
        let mut ids: Vec<ObjectId> = w
            .update
            .order
            .iter()
            .copied()
            .filter(|&id| on_map(id))
            .collect();
        let mut rest: Vec<ObjectId> = w
            .objects
            .values()
            .filter(|o| o.kind.is_mobile() && o.on_map && !ids.contains(&o.id))
            .map(|o| o.id)
            .collect();
        ids.append(&mut rest);
        let mut placed = Vec::new();
        for id in ids {
            let Some(layer) = evaluate(w, id) else {
                continue;
            };
            let (airborne, phase) = moment_state(w, id);
            let put = match self.tracks.get(&id) {
                // spec: S-0102 claim 4 (evaluated when placed on the map)
                None => true,
                Some(_) if !tick => false,
                Some(t) => {
                    // spec: S-0102 claim 4 and S-0501 claim 7 (take-off and landing of flying movement: the height above the surface leaves or returns to 0)
                    let flying_moment = t.airborne != airborne;
                    // spec: S-0102 claim 4, S-0502 claims 5 and 15 (take-off and landing of jumpjet movement: leaving or entering the grounded state)
                    let jumpjet_moment = (t.phase == Some(Phase::Grounded))
                        != (phase == Some(Phase::Grounded))
                        && t.phase.is_some()
                        && phase.is_some();
                    // spec: S-0102 claim 4 (while falling, re-evaluated every tick and changed as soon as it differs)
                    let falling = glasnost_sim::fall::falling(w, id) && t.layer != layer;
                    // spec: S-0102 Edge cases (quirks.layer_change_on_events_only off: re-evaluated every tick)
                    let every_tick = !events_only && t.layer != layer;
                    flying_moment || jumpjet_moment || falling || every_tick
                }
            };
            let track = self.tracks.entry(id).or_insert(Track {
                layer,
                airborne,
                phase,
            });
            track.airborne = airborne;
            track.phase = phase;
            if put {
                track.layer = layer;
                self.place(super::live::Entrant::Object(id), layer);
                placed.push((id, layer));
            }
        }
        placed
    }
}

#[cfg(test)]
mod tests {
    //! Synthetic rules and an empty flat map; every name and number is
    //! invented.

    use glasnost_rules::sim::{Decimal, LocomotionClass, Locomotor as L, MovementZone, UnitType};
    use glasnost_sim::jump::Phase;
    use glasnost_sim::rules::Kind;
    use glasnost_sim::testkit;
    use glasnost_sim::world::{ObjectId, World};

    use super::super::live::Entrant;
    use super::{Layer, Layers, evaluate};

    const SIZE: i32 = 20;

    /// The test ruleset: the synthetic one plus aircraft JET (flying
    /// movement) and infantry JJ (jumpjet movement, cruise height 500).
    fn world() -> World {
        let mut r = testkit::ruleset();
        r.registries.insert("aircraft".into(), vec!["JET".into()]);
        r.registries.get_mut("infantry").unwrap().push("JJ".into());
        r.sim.aircraft.insert(
            "JET".into(),
            UnitType {
                strength: Some(100),
                speed: Some(10),
                locomotor: Some(L::Fly),
                movement_zone: Some(MovementZone::Fly),
                ..Default::default()
            },
        );
        r.sim.infantry.insert(
            "JJ".into(),
            UnitType {
                strength: Some(100),
                speed: Some(4),
                locomotion_class: Some(LocomotionClass::Foot),
                movement_zone: Some(MovementZone::Fly),
                locomotor: Some(L::Jumpjet),
                jumpjet: Some(true),
                jumpjet_speed: Some(10),
                jumpjet_climb: Some(Decimal("5.0".into())),
                jumpjet_height: Some(500),
                ..Default::default()
            },
        );
        testkit::game_with(&r, SIZE, 1)
    }

    fn spawn(w: &mut World, kind: Kind, name: &str, dx: i32) -> ObjectId {
        testkit::spawn(w, kind, name, 0, testkit::mid(SIZE, dx, 0), 0)
    }

    fn lift(w: &mut World, id: ObjectId, z: i32) {
        w.obj_mut(id).unwrap().pos.z = z;
    }

    fn phase(w: &mut World, id: ObjectId, p: Phase) {
        w.obj_mut(id).unwrap().jump.as_mut().unwrap().phase = p;
    }

    /// spec: S-0102 claim 3 and State and ordering (driving movement:
    /// ground at any height; flying movement: top above 0, ground at 0,
    /// S-0501 claim 7; rocket movement: air)
    #[test]
    fn claim3_rows_by_movement_type() {
        let mut w = world();
        let cart = spawn(&mut w, Kind::Vehicle, "CART", 0);
        let jet = spawn(&mut w, Kind::Aircraft, "JET", 2);
        lift(&mut w, cart, 300);
        assert_eq!(evaluate(&w, cart), Some(Layer::Ground));
        assert_eq!(evaluate(&w, jet), Some(Layer::Ground));
        lift(&mut w, jet, 1);
        assert_eq!(evaluate(&w, jet), Some(Layer::Top));
        w.obj_mut(cart).unwrap().rocket = Some(Default::default());
        assert_eq!(evaluate(&w, cart), Some(Layer::Air));
        w.obj_mut(cart).unwrap().on_map = false;
        assert_eq!(evaluate(&w, cart), None, "off the map: no layer");
    }

    /// spec: S-0102 State and ordering (jumpjet movement: ground when r <
    /// 208 or r′ = 0, air below the cruise height, top at or above it)
    #[test]
    fn claim3_jumpjet_two_heights() {
        let mut w = world();
        let jj = spawn(&mut w, Kind::Infantry, "JJ", 0);
        for (z, want) in [
            (0, Layer::Ground),
            (100, Layer::Ground),
            (207, Layer::Ground),
            (208, Layer::Air),
            (499, Layer::Air),
            (500, Layer::Top),
            (900, Layer::Top),
        ] {
            lift(&mut w, jj, z);
            assert_eq!(evaluate(&w, jj), Some(want), "r = {z}");
        }
    }

    /// spec: S-0102 State and ordering (over a high bridge without the
    /// on-deck state: r = 416 is ground through r′ = 0, 417 is not; the
    /// falling state suppresses the subtraction)
    #[test]
    fn claim3_jumpjet_bridge_rule() {
        let mut w = world();
        let jj = spawn(&mut w, Kind::Infantry, "JJ", 0);
        let cell = w.obj(jj).unwrap().pos.cell();
        let i = w.cells.idx(cell).unwrap();
        w.cells.deck[i] = true;
        lift(&mut w, jj, 416);
        assert_eq!(evaluate(&w, jj), Some(Layer::Ground));
        lift(&mut w, jj, 417);
        assert_eq!(evaluate(&w, jj), Some(Layer::Air));
        lift(&mut w, jj, 916);
        assert_eq!(evaluate(&w, jj), Some(Layer::Top), "r′ = 500");
        lift(&mut w, jj, 416);
        w.obj_mut(jj).unwrap().fall.falling = true;
        assert_eq!(evaluate(&w, jj), Some(Layer::Air), "falling: r′ = r");
    }

    /// spec: S-0102 claims 4 and 7 (placement puts every object in its
    /// layer; a take-off puts an aircraft at the end of the top list; a
    /// landing brings it back to ground; between moments it stays)
    #[test]
    fn claim4_take_off_and_landing_of_flying_movement() {
        let mut w = world();
        let a = spawn(&mut w, Kind::Aircraft, "JET", 0);
        let b = spawn(&mut w, Kind::Aircraft, "JET", 2);
        let mut l = Layers::default();
        l.evaluate_mobiles(&w, false, true);
        assert_eq!(l.of(a), Some(Layer::Ground));
        lift(&mut w, b, 100);
        l.evaluate_mobiles(&w, true, true);
        lift(&mut w, a, 100);
        l.evaluate_mobiles(&w, true, true);
        assert_eq!(
            l.list(Layer::Top),
            [Entrant::Object(b), Entrant::Object(a)],
            "the later take-off is drawn last"
        );
        // Climbing on is no moment: the list keeps its order.
        lift(&mut w, b, 900);
        l.evaluate_mobiles(&w, true, true);
        assert_eq!(l.list(Layer::Top), [Entrant::Object(b), Entrant::Object(a)]);
        // b lands and takes off again: it re-enters at the end (claim 7).
        lift(&mut w, b, 0);
        l.evaluate_mobiles(&w, true, true);
        assert_eq!(l.of(b), Some(Layer::Ground));
        assert_eq!(l.list(Layer::Top), [Entrant::Object(a)]);
        lift(&mut w, b, 10);
        l.evaluate_mobiles(&w, true, true);
        assert_eq!(l.list(Layer::Top), [Entrant::Object(a), Entrant::Object(b)]);
        // An object that leaves the map leaves its layer.
        w.obj_mut(a).unwrap().on_map = false;
        l.evaluate_mobiles(&w, true, true);
        assert_eq!(l.list(Layer::Top), [Entrant::Object(b)]);
        assert_eq!(l.of(a), None);
    }

    /// spec: S-0102 claim 4 and Edge cases (quirks.layer_change_on_events_only:
    /// a jumpjet evaluated at its take-off from the ground, r = 0, stays in
    /// the ground layer while it climbs past its cruise height; with the
    /// switch off it follows its height every tick; landing is a moment)
    #[test]
    fn claim4_jumpjet_moments_and_the_events_only_switch() {
        for events_only in [true, false] {
            let mut w = world();
            let jj = spawn(&mut w, Kind::Infantry, "JJ", 0);
            let mut l = Layers::default();
            l.evaluate_mobiles(&w, false, events_only);
            // Take-off: grounded to ascending at r = 0.
            phase(&mut w, jj, Phase::Ascending);
            l.evaluate_mobiles(&w, true, events_only);
            assert_eq!(l.of(jj), Some(Layer::Ground));
            lift(&mut w, jj, 300);
            l.evaluate_mobiles(&w, true, events_only);
            let want = if events_only {
                Layer::Ground
            } else {
                Layer::Air
            };
            assert_eq!(l.of(jj), Some(want), "r = 300, events only: {events_only}");
            lift(&mut w, jj, 600);
            phase(&mut w, jj, Phase::Hovering);
            l.evaluate_mobiles(&w, true, events_only);
            let want = if events_only {
                Layer::Ground
            } else {
                Layer::Top
            };
            assert_eq!(l.of(jj), Some(want), "r = 600, events only: {events_only}");
            // Landing: entering the grounded state.
            lift(&mut w, jj, 0);
            phase(&mut w, jj, Phase::Grounded);
            l.evaluate_mobiles(&w, true, events_only);
            assert_eq!(l.of(jj), Some(Layer::Ground));
            assert!(l.list(Layer::Top).is_empty());
            assert!(l.list(Layer::Air).is_empty());
        }
    }

    /// spec: S-0102 claim 4 (while falling, the layer is re-evaluated every
    /// tick and changed as soon as it differs)
    #[test]
    fn claim4_falling_is_evaluated_every_tick() {
        let mut w = world();
        let jj = spawn(&mut w, Kind::Infantry, "JJ", 0);
        let mut l = Layers::default();
        lift(&mut w, jj, 600);
        phase(&mut w, jj, Phase::Hovering);
        l.evaluate_mobiles(&w, false, true);
        assert_eq!(l.of(jj), Some(Layer::Top));
        w.obj_mut(jj).unwrap().fall.falling = true;
        lift(&mut w, jj, 400);
        l.evaluate_mobiles(&w, true, true);
        assert_eq!(l.of(jj), Some(Layer::Air));
        lift(&mut w, jj, 100);
        l.evaluate_mobiles(&w, true, true);
        assert_eq!(l.of(jj), Some(Layer::Ground));
    }

    /// spec: S-0102 claim 7 and Edge cases (quirks.unsorted_upper_layers
    /// off: the lists are sorted by key, stably)
    #[test]
    fn claim7_lists_keep_entering_order_unless_sorted() {
        let mut l = Layers::default();
        let (a, b, c) = (
            Entrant::Object(ObjectId(1)),
            Entrant::Anim(7),
            Entrant::Particle(ObjectId(3)),
        );
        l.place(a, Layer::Air);
        l.place(b, Layer::Air);
        l.place(c, Layer::Air);
        l.place(a, Layer::Air);
        assert_eq!(l.list(Layer::Air), [b, c, a], "re-entry goes to the end");
        l.place(c, Layer::Ground);
        assert_eq!(l.list(Layer::Air), [b, a], "ground has no list here");
        l.sort_by_key(|e| if e == a { 1 } else { 2 });
        assert_eq!(l.list(Layer::Air), [a, b]);
    }
}
