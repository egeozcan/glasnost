//! The render layers of the mobile objects (S-0102 claims 3, 4 and 7):
//! which layer the table of *State and ordering* gives an object, the
//! moments at which the drawing evaluates it, and the insertion-ordered
//! lists of the layers other than ground.
//!
//! All of this is presentation state of the game client: it reads the
//! simulation and the simulation never reads it.

use std::collections::BTreeMap;

use glasnost_sim::combat::effects::LayerHeights;
use glasnost_sim::jump;
use glasnost_sim::rocket::Phase as RocketPhase;
use glasnost_sim::rules::Locomotor;
use glasnost_sim::spawn::RocketSetId;
use glasnost_sim::world::{ObjectId, World};

pub use super::slots::Layer;

/// The raw height below which a unit with jumpjet movement evaluates to
/// ground (S-0102 *State and ordering*).
const JUMPJET_GROUND_HEIGHT: i32 = 208; // spec: S-0102 State and ordering (r < 208 leptons: ground)

/// A moment of S-0102 claim 4 the simulation reported for an object
/// (`Effect::LayerMoment`): a re-file (`None`) or a check with the heights
/// before and after the object's step.
pub type Moment = (ObjectId, Option<(LayerHeights, LayerHeights)>);

/// The layer S-0102's table gives mobile object `id` for its state and
/// height now (claim 3, *State and ordering*), `None` for an object that
/// is not a mobile object on the map.
pub fn evaluate(w: &World, id: ObjectId) -> Option<Layer> {
    evaluate_at(w, id, jump::layer_heights(w, id))
}

/// The layer of mobile object `id` for its state now and the heights `h`
/// (*r* and *r*′, S-0102 *State and ordering*).
pub fn evaluate_at(w: &World, id: ObjectId, h: LayerHeights) -> Option<Layer> {
    let o = w.obj(id)?;
    if !o.kind.is_mobile() || !o.on_map {
        return None;
    }
    // spec: S-0102 claim 3 and State and ordering (a superweapon warp member, any kind, is in the ground layer from its warp's launch until at least the end of its arrival hold, whatever its height; this row comes before every other row, S-0306 claim 17)
    if glasnost_sim::superweapons::warp::in_ground_layer(w, id) {
        return Some(Layer::Ground);
    }
    let loco = w.rules().unit(o.kind, o.type_index).map(|u| u.locomotor);
    // spec: S-0102 State and ordering (a unit with rocket movement is always in the air layer)
    if o.rocket.is_some() || loco == Some(Locomotor::Rocket) {
        return Some(Layer::Air);
    }
    // spec: S-0102 State and ordering (the layer follows the movement type in use; a seized unit moves with the jumpjet locomotor, S-0607 claim 5)
    if jump::is_jumpjet(w, id) {
        return Some(jumpjet_layer(w, id, h));
    }
    if o.kind == glasnost_sim::rules::Kind::Aircraft || loco == Some(Locomotor::Fly) {
        // spec: S-0102 State and ordering and S-0501 claim 7 (flying movement: top at any positive height above the surface, ground at 0 or below)
        return Some(if h.r > 0 { Layer::Top } else { Layer::Ground });
    }
    // spec: S-0102 State and ordering (walking, driving, hovering, ship, mech and teleport movement: always ground; tunnel and drop-pod movement are not modelled)
    Some(Layer::Ground)
}

/// The two-height rule of a unit with jumpjet movement (S-0102 *State and
/// ordering*), from *r* and *r*′ as the simulation gives them
/// (`jump::layer_heights`: S-0502 claim 4's same predicate, with the
/// on-deck, high-bridge and falling conditions of *r*′).
fn jumpjet_layer(w: &World, id: ObjectId, h: LayerHeights) -> Layer {
    // spec: S-0102 State and ordering (ground when r < 208 or r′ = 0; the 208 test reads r)
    if h.r < JUMPJET_GROUND_HEIGHT || h.adjusted == 0 {
        return Layer::Ground;
    }
    // spec: S-0102 State and ordering (the type's jumpjet cruise height after its minimum, S-0502 claim 1)
    let cruise = jump::fields(w, id).map_or(0, |f| f.height);
    // spec: S-0102 State and ordering (top at or above the jumpjet cruise height, compared with r′; air below it)
    if h.adjusted >= cruise {
        Layer::Top
    } else {
        Layer::Air
    }
}

/// What the drawing remembers of a mobile object between ticks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Track {
    /// The layer it was last put into.
    pub layer: Layer,
    /// A set-C rocket missile's phase after the last tick (S-0506).
    rocket: Option<RocketPhase>,
    /// Its first ascent update has re-filed it.
    ascended: bool,
}

/// A set-C rocket missile's phase (S-0506), `None` for anything else.
fn set_c_phase(w: &World, id: ObjectId) -> Option<RocketPhase> {
    let r = w.obj(id)?.rocket.as_ref()?;
    if r.set == RocketSetId::C {
        r.phase
    } else {
        None
    }
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

    /// Re-files `id` (S-0102 claim 4): takes it out of its list and puts it
    /// into the list of `layer`, at the end for a layer other than ground.
    fn refile(&mut self, id: ObjectId, layer: Layer, placed: &mut Vec<(ObjectId, Layer)>) {
        if let Some(t) = self.tracks.get_mut(&id) {
            t.layer = layer;
        }
        self.place(super::live::Entrant::Object(id), layer);
        placed.push((id, layer));
    }

    /// Evaluates the mobile objects at claim 4's moments after a tick (or
    /// at the start, `tick` false: placement only). Objects that left the
    /// map leave their layer. Every evaluation reads the state after the
    /// tick, except that a check compares the heights the simulation
    /// reported around the object's step.
    ///
    /// - **Placement** (an object the drawing has not seen on the map): a
    ///   re-file, in the update order (S-0203 claim 3). An aircraft that a
    ///   factory release left above its pad in the tick of its ground
    ///   placement, not re-filed since (`air::ground_filed`), is filed in
    ///   the ground layer its placement gave it (S-0501 claim 48 (a)).
    /// - **The simulation's moments** (`moments`, `Effect::LayerMoment`, in
    ///   the order they happened): a jumpjet's movement step and a fall are
    ///   checks, re-filed only when the layers before and after the step
    ///   differ; the "before" layer is the one evaluated at the step's
    ///   start, not the list the object is in. A jumpjet's or an aircraft's
    ///   crash tick, a landable aircraft's take-off and landing updates and
    ///   the start of a dropped explosive's fall are re-files.
    /// - **Set-C rocket missiles:** re-filed at their first pause update
    ///   and at their first ascent update, seen from their phase after each
    ///   tick (a missile takes one update per tick).
    /// - **Superweapon warp members** (S-0102 claim 3): one that is in a
    ///   list other than ground once its warp has launched is put into the
    ///   ground layer (interim, Q-2346). After its arrival hold, it stays
    ///   there until one of claim 4's moments re-files it (Q-2346).
    /// - With `events_only` off (`quirks.layer_change_on_events_only`):
    ///   the checks give way to a sweep after the re-files: every object
    ///   whose evaluated layer differs from the list it is in moves to that
    ///   layer's list.
    ///
    /// Returns the objects that were put into a layer, in that order, with
    /// the layer.
    pub fn evaluate_mobiles(
        &mut self,
        w: &World,
        tick: bool,
        events_only: bool,
        moments: &[Moment],
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
        for &id in &ids {
            if self.tracks.contains_key(&id) {
                continue;
            }
            let Some(mut layer) = evaluate(w, id) else {
                continue;
            };
            if glasnost_sim::air::ground_filed(w, id) {
                // spec: S-0501 claim 48 (a) and S-0102 claim 4 (an aircraft that a factory release raised above its pad in the tick of its ground placement was filed in the ground render layer by that placement, and the release does not re-file it)
                layer = Layer::Ground;
            }
            self.tracks.insert(
                id,
                Track {
                    layer,
                    rocket: set_c_phase(w, id),
                    ascended: false,
                },
            );
            // spec: S-0102 claim 4 (placement re-files the object)
            self.refile(id, layer, &mut placed);
        }
        if !tick {
            return placed;
        }
        for &(id, check) in moments {
            if !self.tracks.contains_key(&id) {
                continue;
            }
            match check {
                None => {
                    // spec: S-0102 claim 4 (a re-file moment: a crash tick, a landable aircraft's take-off or landing update, the start of a dropped explosive's fall)
                    if let Some(layer) = evaluate(w, id) {
                        self.refile(id, layer, &mut placed);
                    }
                }
                Some((before, after)) if events_only => {
                    // spec: S-0102 claim 4 and Edge cases (quirks.layer_change_on_events_only: a check compares the layers evaluated just before and just after the object's own step, not the list it is in)
                    let (Some(b), Some(a)) =
                        (evaluate_at(w, id, before), evaluate_at(w, id, after))
                    else {
                        continue;
                    };
                    if b != a {
                        self.refile(id, a, &mut placed);
                    }
                }
                Some(_) => {}
            }
        }
        for &id in &ids {
            let Some(t) = self.tracks.get(&id).copied() else {
                continue;
            };
            let now = set_c_phase(w, id);
            // spec: S-0102 claim 4 (rocket movement, set C only: its first pause update, the tick its phase becomes pause)
            let first_pause = t.rocket.is_none() && now == Some(RocketPhase::Pause);
            // spec: S-0102 claim 4 (and its first ascent update, the tick after its raise ends)
            let first_ascent = t.rocket == Some(RocketPhase::Ascent) && !t.ascended;
            if let Some(tr) = self.tracks.get_mut(&id) {
                tr.rocket = now;
                tr.ascended |= first_ascent;
            }
            if (first_pause || first_ascent)
                && let Some(layer) = evaluate(w, id)
            {
                self.refile(id, layer, &mut placed);
            }
        }
        for &id in &ids {
            let Some(t) = self.tracks.get(&id).copied() else {
                continue;
            };
            // spec: S-0102 claim 3 (a superweapon warp member is in the ground layer from its warp's launch, whatever its height); claim 4 names no moment at the launch, so a member still in an upper list is put into the ground layer after the launch's tick (interim, Q-2346)
            if t.layer != Layer::Ground && glasnost_sim::superweapons::warp::in_ground_layer(w, id)
            {
                self.refile(id, Layer::Ground, &mut placed);
            }
        }
        if !events_only {
            for &id in &ids {
                let (Some(t), Some(layer)) = (self.tracks.get(&id), evaluate(w, id)) else {
                    continue;
                };
                // spec: S-0102 Edge cases (quirks.layer_change_on_events_only off: every tick, an object whose evaluated layer differs from the list it is in moves to that layer; an object whose layer is unchanged is not moved by this check)
                if t.layer != layer {
                    self.refile(id, layer, &mut placed);
                }
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
    use glasnost_sim::combat::effects::LayerHeights;
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

    /// The heights `r` with `r′ = r` (no bridge).
    fn h(r: i32) -> LayerHeights {
        LayerHeights { r, adjusted: r }
    }

    /// spec: S-0102 claims 4 and 7 (placement re-files every object into
    /// its layer, in the update order; between moments an object stays in
    /// its list; an object that leaves the map leaves its layer)
    #[test]
    fn claim4_placement_and_no_moment() {
        let mut w = world();
        let a = spawn(&mut w, Kind::Aircraft, "JET", 0);
        let b = spawn(&mut w, Kind::Aircraft, "JET", 2);
        lift(&mut w, a, 100);
        let mut l = Layers::default();
        let placed = l.evaluate_mobiles(&w, false, true, &[]);
        assert_eq!(placed, [(a, Layer::Top), (b, Layer::Ground)]);
        // Height changes without a moment: nothing moves.
        lift(&mut w, a, 0);
        lift(&mut w, b, 100);
        assert!(l.evaluate_mobiles(&w, true, true, &[]).is_empty());
        assert_eq!(l.of(a), Some(Layer::Top));
        assert_eq!(l.of(b), Some(Layer::Ground));
        w.obj_mut(a).unwrap().on_map = false;
        l.evaluate_mobiles(&w, true, true, &[]);
        assert!(l.list(Layer::Top).is_empty());
        assert_eq!(l.of(a), None);
    }

    /// spec: S-0102 claims 4 and 7 (a re-file moment, here a landable
    /// aircraft's take-off and landing updates or a crash tick, takes the
    /// object out of its list and puts it at the end of its evaluated
    /// layer's list, even when the layer is unchanged; re-files happen with
    /// quirks.layer_change_on_events_only on and off)
    #[test]
    fn claim4_refile_moments_go_to_the_end() {
        let mut w = world();
        let a = spawn(&mut w, Kind::Aircraft, "JET", 0);
        let b = spawn(&mut w, Kind::Aircraft, "JET", 2);
        lift(&mut w, a, 100);
        lift(&mut w, b, 100);
        let mut l = Layers::default();
        l.evaluate_mobiles(&w, false, true, &[]);
        assert_eq!(l.list(Layer::Top), [Entrant::Object(a), Entrant::Object(b)]);
        // A re-file in the same layer (a crash tick in the air).
        l.evaluate_mobiles(&w, true, true, &[(a, None)]);
        assert_eq!(l.list(Layer::Top), [Entrant::Object(b), Entrant::Object(a)]);
        // A landing update that reaches the ground.
        lift(&mut w, b, 0);
        let placed = l.evaluate_mobiles(&w, true, true, &[(b, None)]);
        assert_eq!(placed, [(b, Layer::Ground)]);
        assert_eq!(l.list(Layer::Top), [Entrant::Object(a)]);
        // A take-off with the switch off.
        lift(&mut w, b, 10);
        l.evaluate_mobiles(&w, true, false, &[(b, None)]);
        assert_eq!(l.list(Layer::Top), [Entrant::Object(a), Entrant::Object(b)]);
    }

    /// spec: S-0102 claim 4 (a non-landable aircraft is never re-filed by
    /// take-off or landing: without a moment it stays in its placement
    /// layer while its height changes)
    #[test]
    fn claim4_non_landable_aircraft_stays_in_its_placement_layer() {
        let mut w = world();
        let a = spawn(&mut w, Kind::Aircraft, "JET", 0);
        let mut l = Layers::default();
        l.evaluate_mobiles(&w, false, true, &[]);
        for z in [50, 600, 0] {
            lift(&mut w, a, z);
            l.evaluate_mobiles(&w, true, true, &[]);
            assert_eq!(l.of(a), Some(Layer::Ground), "r = {z}");
        }
    }

    /// spec: S-0102 claim 4 and Edge cases (a jumpjet movement step is a
    /// check: re-filed only when the layers evaluated before and after the
    /// step differ; "before" is the evaluation at the step's start, not
    /// the list it is in)
    #[test]
    fn claim4_jumpjet_check_compares_before_and_after() {
        let mut w = world();
        let jj = spawn(&mut w, Kind::Infantry, "JJ", 0);
        let other = spawn(&mut w, Kind::Infantry, "JJ", 2);
        let mut l = Layers::default();
        l.evaluate_mobiles(&w, false, true, &[]);
        assert_eq!(l.of(jj), Some(Layer::Ground));
        // A step from 300 to 400: air before and after; it stays in the
        // ground list it is in.
        lift(&mut w, jj, 400);
        let placed = l.evaluate_mobiles(&w, true, true, &[(jj, Some((h(300), h(400))))]);
        assert!(placed.is_empty());
        assert_eq!(l.of(jj), Some(Layer::Ground));
        // A step from 100 to 300: ground to air, re-filed.
        lift(&mut w, jj, 300);
        l.evaluate_mobiles(&w, true, true, &[(jj, Some((h(100), h(300))))]);
        assert_eq!(l.of(jj), Some(Layer::Air));
        // The other one reaches the top, then jj; then the other flips
        // top, air, top and re-enters at the end.
        lift(&mut w, other, 600);
        l.evaluate_mobiles(&w, true, true, &[(other, Some((h(0), h(600))))]);
        lift(&mut w, jj, 500);
        l.evaluate_mobiles(&w, true, true, &[(jj, Some((h(300), h(500))))]);
        assert_eq!(
            l.list(Layer::Top),
            [Entrant::Object(other), Entrant::Object(jj)]
        );
        lift(&mut w, other, 499);
        l.evaluate_mobiles(&w, true, true, &[(other, Some((h(600), h(499))))]);
        lift(&mut w, other, 500);
        l.evaluate_mobiles(&w, true, true, &[(other, Some((h(499), h(500))))]);
        assert_eq!(
            l.list(Layer::Top),
            [Entrant::Object(jj), Entrant::Object(other)],
            "the hover flip re-enters at the end"
        );
    }

    /// spec: S-0102 claim 4 (the movement step of a falling object is a
    /// check; the falling state reads r′ = r)
    #[test]
    fn claim4_fall_check() {
        let mut w = world();
        let jj = spawn(&mut w, Kind::Infantry, "JJ", 0);
        lift(&mut w, jj, 600);
        let mut l = Layers::default();
        l.evaluate_mobiles(&w, false, true, &[]);
        assert_eq!(l.of(jj), Some(Layer::Top));
        w.obj_mut(jj).unwrap().fall.falling = true;
        lift(&mut w, jj, 400);
        l.evaluate_mobiles(&w, true, true, &[(jj, Some((h(600), h(400))))]);
        assert_eq!(l.of(jj), Some(Layer::Air));
        lift(&mut w, jj, 100);
        l.evaluate_mobiles(&w, true, true, &[(jj, Some((h(400), h(100))))]);
        assert_eq!(l.of(jj), Some(Layer::Ground));
    }

    /// spec: S-0102 Edge cases (quirks.layer_change_on_events_only off:
    /// the checks give way to a sweep every tick that moves an object whose
    /// evaluated layer differs from the list it is in; an unchanged object
    /// is not moved)
    #[test]
    fn claim4_switch_off_sweeps_every_tick() {
        let mut w = world();
        let jj = spawn(&mut w, Kind::Infantry, "JJ", 0);
        let jet = spawn(&mut w, Kind::Aircraft, "JET", 2);
        let mut l = Layers::default();
        l.evaluate_mobiles(&w, false, false, &[]);
        // No moment at all: the sweep still moves both.
        lift(&mut w, jj, 300);
        lift(&mut w, jet, 50);
        l.evaluate_mobiles(&w, true, false, &[]);
        assert_eq!(l.of(jj), Some(Layer::Air));
        assert_eq!(l.of(jet), Some(Layer::Top));
        // A check that would re-file is ignored; the sweep sees no change.
        let placed = l.evaluate_mobiles(&w, true, false, &[(jj, Some((h(100), h(300))))]);
        assert!(placed.is_empty());
    }

    /// spec: S-0501 claim 48 (a) and (c) and S-0102 claim 4 (an aircraft
    /// that a factory release left above its pad is filed in the ground
    /// layer by its ground placement and stays there with no re-file while
    /// its height gives the top layer; a re-file moment moves it; with
    /// quirks.layer_change_on_events_only off the sweep moves it to the top
    /// layer at once)
    #[test]
    fn claim4_released_aircraft_stays_in_the_ground_layer() {
        let mut w = world();
        let a = spawn(&mut w, Kind::Aircraft, "JET", 0);
        lift(&mut w, a, 100);
        w.obj_mut(a).unwrap().air = Some(glasnost_sim::air::AirState {
            released: true,
            ground_filed: true,
            ..Default::default()
        });
        assert_eq!(evaluate(&w, a), Some(Layer::Top));
        let mut l = Layers::default();
        let placed = l.evaluate_mobiles(&w, false, true, &[]);
        assert_eq!(placed, [(a, Layer::Ground)]);
        lift(&mut w, a, 500);
        assert!(l.evaluate_mobiles(&w, true, true, &[]).is_empty());
        assert_eq!(l.of(a), Some(Layer::Ground), "no re-file while it flies");
        l.evaluate_mobiles(&w, true, true, &[(a, None)]);
        assert_eq!(l.of(a), Some(Layer::Top), "a re-file moment moves it");
        // The switch off: placed in the ground layer, then the sweep.
        let mut off = Layers::default();
        lift(&mut w, a, 100);
        off.evaluate_mobiles(&w, true, false, &[]);
        assert_eq!(off.of(a), Some(Layer::Top), "the sweep of the same tick");
    }

    /// A warp member's state at `stage` (S-0513 T2); the landing point is
    /// where the object stands.
    fn warp(w: &mut World, id: ObjectId, stage: Option<glasnost_sim::superweapons::warp::Stage>) {
        use glasnost_sim::superweapons::warp::Member;
        let landing = w.obj(id).unwrap().pos;
        w.obj_mut(id).unwrap().sw.warp = stage.map(|stage| Member {
            stage,
            house: glasnost_sim::world::HouseId(0),
            landing,
            redirected: false,
            fell_back: false,
            refused: false,
            entry_ok: true,
        });
    }

    /// spec: S-0102 claim 3 and State and ordering (a superweapon warp
    /// member, any kind, is in the ground layer from its warp's launch until
    /// at least the end of its arrival hold, whatever its height; the row
    /// comes before every other, the rocket and jumpjet rows included)
    #[test]
    fn claim3_a_warp_member_is_in_the_ground_layer_whatever_its_height() {
        use glasnost_sim::superweapons::warp::Stage;
        let mut w = world();
        let jj = spawn(&mut w, Kind::Infantry, "JJ", 0);
        let jet = spawn(&mut w, Kind::Aircraft, "JET", 2);
        let cart = spawn(&mut w, Kind::Vehicle, "CART", 4);
        lift(&mut w, jj, 600);
        lift(&mut w, jet, 300);
        w.obj_mut(cart).unwrap().rocket = Some(Default::default());
        assert_eq!(evaluate(&w, jj), Some(Layer::Top));
        assert_eq!(evaluate(&w, jet), Some(Layer::Top));
        assert_eq!(evaluate(&w, cart), Some(Layer::Air));
        for stage in [
            Stage::Pending,
            Stage::Leaving(0),
            Stage::Expired,
            Stage::Placed,
            Stage::Hold(0),
        ] {
            for id in [jj, jet, cart] {
                warp(&mut w, id, Some(stage));
                assert_eq!(evaluate(&w, id), Some(Layer::Ground), "{stage:?}");
            }
        }
        // The hold over: the ordinary rows again.
        for id in [jj, jet, cart] {
            warp(&mut w, id, None);
        }
        assert_eq!(evaluate(&w, jj), Some(Layer::Top));
        assert_eq!(evaluate(&w, jet), Some(Layer::Top));
        assert_eq!(evaluate(&w, cart), Some(Layer::Air));
    }

    /// spec: S-0102 claim 3 (a warp member is in the ground layer from its
    /// warp's launch) and claim 4 (placement re-files; between moments an
    /// object stays in its list), both positions of
    /// quirks.layer_change_on_events_only; interim Q-2346 (launch and the
    /// hold's end name no moment: an upper-list member is put into the
    /// ground layer after the launch's tick, and after the hold it stays
    /// there until a moment re-files it)
    #[test]
    fn claim4_a_warp_member_leaves_its_upper_list_at_the_launch() {
        use glasnost_sim::superweapons::warp::Stage;
        for events_only in [true, false] {
            let mut w = world();
            let jj = spawn(&mut w, Kind::Infantry, "JJ", 0);
            lift(&mut w, jj, 600);
            let mut l = Layers::default();
            l.evaluate_mobiles(&w, false, events_only, &[]);
            assert_eq!(l.list(Layer::Top), [Entrant::Object(jj)]);
            // The launch: the member joins its group.
            warp(&mut w, jj, Some(Stage::Pending));
            let placed = l.evaluate_mobiles(&w, true, events_only, &[]);
            assert_eq!(placed, [(jj, Layer::Ground)], "events only {events_only}");
            assert!(l.list(Layer::Top).is_empty());
            // A jumpjet check during the warp: both sides ground, no re-file.
            assert!(
                l.evaluate_mobiles(&w, true, events_only, &[(jj, Some((h(600), h(600))))])
                    .is_empty()
            );
            // Departure takes it off the map; it leaves every list.
            w.obj_mut(jj).unwrap().on_map = false;
            warp(&mut w, jj, Some(Stage::Away));
            l.evaluate_mobiles(&w, true, events_only, &[]);
            assert_eq!(l.of(jj), None);
            // Its placement at the landing point re-files it, into ground.
            w.obj_mut(jj).unwrap().on_map = true;
            warp(&mut w, jj, Some(Stage::Placed));
            let placed = l.evaluate_mobiles(&w, true, events_only, &[]);
            assert_eq!(placed, [(jj, Layer::Ground)]);
            warp(&mut w, jj, Some(Stage::Hold(3)));
            assert!(l.evaluate_mobiles(&w, true, events_only, &[]).is_empty());
            // The hold over.
            warp(&mut w, jj, None);
            let placed = l.evaluate_mobiles(&w, true, events_only, &[]);
            if events_only {
                assert!(placed.is_empty(), "no moment: it stays in ground");
                assert_eq!(l.of(jj), Some(Layer::Ground));
                l.evaluate_mobiles(&w, true, events_only, &[(jj, None)]);
                assert_eq!(l.of(jj), Some(Layer::Top), "a re-file moment moves it");
            } else {
                assert_eq!(placed, [(jj, Layer::Top)], "the sweep of the same tick");
            }
        }
    }

    /// spec: S-0102 claim 4 (rocket movement, set C only: re-filed at its
    /// first pause update and at its first ascent update; other sets and
    /// later updates are no moment)
    #[test]
    fn claim4_set_c_rocket_pause_and_ascent() {
        use glasnost_sim::rocket::{Phase as R, RocketState};
        use glasnost_sim::spawn::RocketSetId;
        let mut w = world();
        let c = spawn(&mut w, Kind::Vehicle, "CART", 0);
        let a = spawn(&mut w, Kind::Vehicle, "CART", 2);
        let other = spawn(&mut w, Kind::Vehicle, "CART", 4);
        w.obj_mut(c).unwrap().rocket = Some(RocketState {
            set: RocketSetId::C,
            ..Default::default()
        });
        w.obj_mut(a).unwrap().rocket = Some(RocketState::default());
        w.obj_mut(other).unwrap().rocket = Some(RocketState::default());
        let mut l = Layers::default();
        l.evaluate_mobiles(&w, false, true, &[]);
        assert_eq!(
            l.list(Layer::Air),
            [
                Entrant::Object(c),
                Entrant::Object(a),
                Entrant::Object(other)
            ]
        );
        let set = |w: &mut World, id: ObjectId, p: Option<R>| {
            w.obj_mut(id).unwrap().rocket.as_mut().unwrap().phase = p;
        };
        // Set A entering its pause: no moment.
        set(&mut w, a, Some(R::Pause));
        assert!(l.evaluate_mobiles(&w, true, true, &[]).is_empty());
        // Set C's first pause update.
        set(&mut w, c, Some(R::Pause));
        assert_eq!(l.evaluate_mobiles(&w, true, true, &[]), [(c, Layer::Air)]);
        assert_eq!(l.list(Layer::Air).last(), Some(&Entrant::Object(c)));
        // Later pause and raise updates: no moment.
        assert!(l.evaluate_mobiles(&w, true, true, &[]).is_empty());
        set(&mut w, c, Some(R::Raise));
        assert!(l.evaluate_mobiles(&w, true, true, &[]).is_empty());
        // The tick its phase becomes ascent ends the raise; the next tick
        // is its first ascent update, and only the first.
        set(&mut w, c, Some(R::Ascent));
        assert!(l.evaluate_mobiles(&w, true, true, &[]).is_empty());
        assert_eq!(l.evaluate_mobiles(&w, true, true, &[]), [(c, Layer::Air)]);
        assert!(l.evaluate_mobiles(&w, true, true, &[]).is_empty());
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
