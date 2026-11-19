//! S-0109 r45 claim 22 (Q-3240, #291): an attached animation ends with its
//! owner at each event of the *End with the owner* list. It is not drawn
//! from the event on and is deleted at its next update, which does nothing
//! else; S-0611 claim 15's rules for that end; the events that do not end
//! it; and the control ring shown again (S-0109 Changes for other specs,
//! the S-0601 claim 18 entry). Synthetic rules only, on the `sw_common`
//! superweapon ruleset (and `jump_common` for the jumpjet), with:
//! - STAY: an animation type of a thousand frames at one update a frame, so
//!   it outlasts every test by itself;
//! - MAKER: twenty frames at one update a frame, make-infantry index 0, the
//!   animation-to-infantry list `[GROWN]` and a neutral civilian-side house
//!   Civ (house 2), as in claims_s0611_moving.rs;
//! - THIEF: the WALKER infantry type with the vehicle-thief flag;
//! - the permanent control ring type name PERMA.
//!
//! Every name and number is made up.

mod jump_common;
mod sw_common;

use std::collections::BTreeSet;

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::AnimationType;
use glasnost_sim::anims::{self, Role, Spawn};
use glasnost_sim::cells::Marker;
use glasnost_sim::cloak::{self, CloakState};
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{Layer, ObjectId, Pos, Target, World};
use glasnost_sim::{air, jump, mind, naval, objects, testkit};
use sw_common::{ALPHA, BETA, at};

fn reg(r: &mut Ruleset, registry: &str, name: &str) {
    let l = r.registries.entry(registry.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

fn add_anim(r: &mut Ruleset, name: &str, t: AnimationType) {
    r.sim.animations.insert(name.into(), t);
    reg(r, "animations", name);
}

fn stay() -> AnimationType {
    AnimationType {
        frame_count: Some(1000),
        rate: Some(900),
        ..Default::default()
    }
}

fn rules(stale: bool) -> Ruleset {
    use glasnost_rules::sim::Country;
    let mut r = sw_common::ruleset();
    r.sim.infantry.get_mut("WALKER").unwrap().primary = None;
    let grown = r.sim.infantry["WALKER"].clone();
    r.sim.infantry.insert("GROWN".into(), grown);
    reg(&mut r, "infantry", "GROWN");
    let mut thief = r.sim.infantry["WALKER"].clone();
    thief.vehicle_thief = Some(true);
    r.sim.infantry.insert("THIEF".into(), thief);
    reg(&mut r, "infantry", "THIEF");
    add_anim(&mut r, "STAY", stay());
    add_anim(&mut r, "PERMA", stay());
    add_anim(
        &mut r,
        "MAKER",
        AnimationType {
            make_infantry: Some(0),
            frame_count: Some(20),
            rate: Some(900),
            ..Default::default()
        },
    );
    r.sim.general.perma_controlled_animation_type = Some("PERMA".into());
    r.sim.general.superweapons.anim_to_infantry = Some(vec!["GROWN".into()]);
    r.sim.countries.insert(
        "Civland".into(),
        Country {
            multiplay_passive: Some(true),
            econ: glasnost_rules::sim_econ::CountryEcon {
                side: Some("Civilian".into()),
                ..Default::default()
            },
            ..Default::default()
        },
    );
    reg(&mut r, "countries", "Civland");
    r.globals.quirks.infantry_animation_stale_marks = stale;
    r
}

/// A game on [`rules`] with the neutral civilian-side house Civ (house 2).
fn game(stale: bool) -> World {
    let r = rules(stale);
    let mut s = testkit::setup(1);
    s.houses.push(glasnost_sim::world::HouseSetup {
        name: "Civ".into(),
        country: "Civland".into(),
        human: false,
        start: None,
        role: glasnost_sim::world::HouseRole::Neutral,
        ai: false,
    });
    let rules = std::sync::Arc::new(glasnost_sim::rules::SimRules::new(&r, "synthetic-test"));
    let mut w = glasnost_sim::load::new_game(
        rules,
        &testkit::empty_map(sw_common::SIZE),
        testkit::catalogue(),
        &s,
    )
    .expect("loads");
    testkit::run(&mut w, 4);
    w
}

/// Animation `name` attached to `owner` with no displacement and role
/// `role`.
fn attach_as(w: &mut World, owner: ObjectId, name: &str, role: Role) -> ObjectId {
    let p = w.obj(owner).unwrap().pos;
    let s = Spawn {
        owner: Some((owner, Pos::new(0, 0, 0))),
        role,
        ..Default::default()
    };
    anims::spawn(w, name, Some(p), s).id.expect("made")
}

fn attach(w: &mut World, owner: ObjectId) -> ObjectId {
    attach_as(w, owner, "STAY", Role::Ordinary)
}

/// Drawn: the view draws the animations `anims::shown` gives.
fn drawn(w: &World, a: ObjectId) -> bool {
    anims::shown(w, a).is_some()
}

/// Left play: neither drawn nor in the update order.
fn gone(w: &World, a: ObjectId) -> bool {
    !w.anims.anims.contains_key(&a) && !w.update.contains(a)
}

/// The checks of an event made between ticks: not drawn at once; still in
/// the update order until its next update, which deletes it.
fn ends_at_the_event(w: &mut World, a: ObjectId, what: &str) {
    assert!(!drawn(w, a), "{what}: not drawn from the event on");
    assert!(
        w.update.contains(a),
        "{what}: deleted at its next update, not at the event"
    );
    sw_common::step(w);
    assert!(gone(w, a), "{what}: deleted at its next update");
}

fn grown(w: &World) -> Vec<ObjectId> {
    let ti = w.rules().index_of(Kind::Infantry, "GROWN").unwrap();
    w.objects
        .values()
        .filter(|o| o.kind == Kind::Infantry && o.type_index == ti && o.on_map)
        .map(|o| o.id)
        .collect()
}

/// Every standing position (cell, record, position) animation `a` holds.
fn marks_of(w: &World, a: ObjectId) -> BTreeSet<(i32, i32, bool, usize)> {
    let mut out = BTreeSet::new();
    for dy in -9..=9 {
        for dx in -9..=9 {
            let c = at(dx, dy);
            for (deck, l) in [(false, Layer::Ground), (true, Layer::Deck)] {
                let Some(o) = w.cells.occ(c, l) else { continue };
                for (p, m) in o.marks.iter().enumerate() {
                    if matches!(m, Some(m) if matches!(m.by, Marker::Anim(x, _) if x == a)) {
                        out.insert((c.x, c.y, deck, p));
                    }
                }
            }
        }
    }
    out
}

fn cloak_state(w: &World, id: ObjectId) -> CloakState {
    w.obj(id).unwrap().cloak.state
}

fn hidden(s: CloakState) -> bool {
    matches!(s, CloakState::Cloaking | CloakState::Cloaked)
}

/// Sets the own cloak flag of `id` and runs to the end of the tick of its
/// start of cloaking.
fn start_cloaking(w: &mut World, id: ObjectId) {
    w.obj_mut(id).unwrap().cloak.own = true;
    sw_common::until(w, 300, |w| cloak_state(w, id) == CloakState::Cloaking);
    assert_eq!(cloak_state(w, id), CloakState::Cloaking, "it starts to cloak");
}

// spec: S-0109 claim 22 (End with the owner: destroyed, at the killing hit, also when the owner stays in play after it; it disappears inside the action and is deleted at its next update)
#[test]
fn the_killing_hit_ends_it() {
    for name in ["CART", "WALKER"] {
        let mut w = game(true);
        let owner = sw_common::spawn(&mut w, name, BETA, 0, 0);
        let a = attach(&mut w, owner);
        sw_common::step(&mut w);
        assert!(drawn(&w, a));
        sw_common::kill(&mut w, owner);
        ends_at_the_event(&mut w, a, name);
    }
}

// spec: S-0109 claim 22 (End with the owner: starts to sink, the owner staying in play)
#[test]
fn the_start_of_sinking_ends_it() {
    let mut w = game(true);
    let car = sw_common::spawn(&mut w, "CART", BETA, 0, 0);
    let a = attach(&mut w, car);
    sw_common::step(&mut w);
    naval::start_sinking(&mut w, car);
    assert!(w.obj(car).is_some(), "the owner stays in play");
    ends_at_the_event(&mut w, a, "sinking");
}

// spec: S-0109 claim 22 (End with the owner: starts to crash, an aircraft)
#[test]
fn an_aircraft_crash_ends_it() {
    let mut w = game(true);
    let plane = sw_common::spawn(&mut w, "SPYP", BETA, 0, 0);
    w.obj_mut(plane).unwrap().pos.z += 600;
    let a = attach(&mut w, plane);
    sw_common::step(&mut w);
    assert!(drawn(&w, a));
    assert!(air::start_crash(&mut w, plane, true), "a crash starts");
    ends_at_the_event(&mut w, a, "aircraft crash");
}

// spec: S-0109 claim 22 (End with the owner: starts to crash, a jumpjet)
#[test]
fn a_jumpjet_crash_ends_it() {
    let mut r = jump_common::ruleset();
    add_anim(&mut r, "STAY", stay());
    let mut w = testkit::game_with(&r, jump_common::SIZE, 7);
    testkit::run(&mut w, 4);
    let jv = jump_common::put(&mut w, "JV", Kind::Vehicle, 0, 0, 0);
    jump_common::go(&mut w, jv, jump_common::at(6, 0));
    assert!(jump_common::until(&mut w, 200, |w| {
        jump_common::height(w, jv) > 0
    }));
    let a = attach(&mut w, jv);
    jump_common::step(&mut w);
    assert!(drawn(&w, a));
    assert!(jump::start_crash(&mut w, jv, None), "a crash starts");
    ends_at_the_event(&mut w, a, "jumpjet crash");
}

// spec: S-0109 claim 22 (End with the owner: goes off the map, for example into a transport or a building)
#[test]
fn going_inside_ends_it() {
    let mut w = game(true);
    let car = sw_common::spawn(&mut w, "CART", BETA, 0, 0);
    let a = attach(&mut w, car);
    sw_common::step(&mut w);
    objects::go_inside(&mut w, car);
    ends_at_the_event(&mut w, a, "going inside");
}

// spec: S-0109 claim 22 (End with the owner: is removed; it disappears at once and is deleted at its next update)
#[test]
fn the_removal_ends_it() {
    let mut w = game(true);
    let car = sw_common::spawn(&mut w, "CART", BETA, 0, 0);
    let a = attach(&mut w, car);
    sw_common::step(&mut w);
    sw_common::remove(&mut w, car);
    ends_at_the_event(&mut w, a, "removal");
}

// spec: S-0109 claim 22 (End with the owner: starts to cloak, whatever house owns it and whatever senses its cell; the deleting update in the tick of the event or the next, as the update order places it)
#[test]
fn the_start_of_cloaking_ends_it() {
    let mut w = game(true);
    let car = sw_common::spawn(&mut w, "CART", BETA, 0, 0);
    let a = attach(&mut w, car);
    sw_common::step(&mut w);
    assert!(drawn(&w, a));
    start_cloaking(&mut w, car);
    assert!(!drawn(&w, a), "not drawn after the start of cloaking");
    sw_common::step(&mut w);
    assert!(gone(&w, a), "deleted by the next tick");
}

/// An animation attached to `id` after its start of cloaking, and whether
/// it is still drawn once `id` is fully cloaked.
fn drawn_after_full_cloak(w: &mut World, id: ObjectId) -> bool {
    start_cloaking(w, id);
    let b = attach(w, id);
    sw_common::step(w);
    assert!(drawn(w, b), "attached after the start of cloaking");
    sw_common::until(w, 600, |w| cloak_state(w, id) == CloakState::Cloaked);
    assert_eq!(cloak_state(w, id), CloakState::Cloaked, "fully cloaked");
    drawn(w, b)
}

// spec: S-0109 claim 22 (End with the owner: becomes fully cloaked, unless it is a building)
#[test]
fn full_cloak_ends_it_except_for_a_building() {
    let mut w = game(true);
    let car = sw_common::spawn(&mut w, "CART", BETA, 0, 0);
    assert!(
        !drawn_after_full_cloak(&mut w, car),
        "a vehicle's full cloak ends it"
    );
    let mut w = game(true);
    let plant = sw_common::spawn(&mut w, "PLANT", BETA, 0, 0);
    assert!(
        drawn_after_full_cloak(&mut w, plant),
        "a building's full cloak does not"
    );
}

// spec: S-0109 claim 22 (End with the owner: a vehicle taken over by a vehicle thief) and S-0210 claim 5 (the crusher is taken over instead)
#[test]
fn a_vehicle_thiefs_takeover_ends_it() {
    let mut w = game(true);
    let crusher = sw_common::spawn(&mut w, "CRUSHER", BETA, 0, 0);
    let thief = sw_common::spawn(&mut w, "THIEF", ALPHA, 0, 0);
    let a = attach(&mut w, crusher);
    sw_common::step(&mut w);
    assert!(drawn(&w, a));
    let p = w.obj(crusher).unwrap().pos;
    let t = w.obj_mut(thief).unwrap();
    t.pos = p;
    t.dest = Some(Target::Object(crusher));
    glasnost_sim::crush::at_cell_centre(&mut w, crusher);
    assert_eq!(w.obj(crusher).unwrap().owner, Some(ALPHA), "taken over");
    ends_at_the_event(&mut w, a, "thief takeover");
}

// spec: S-0109 claim 22 (End with the owner: a vehicle-like building taken over by an infantry unit, S-0509 claim 17)
#[test]
fn a_vehicle_like_building_takeover_ends_it() {
    let mut w = game(true);
    let plant = sw_common::spawn(&mut w, "PLANT", BETA, 0, 0);
    let unit = sw_common::spawn(&mut w, "WALKER", ALPHA, 2, 2);
    let a = attach(&mut w, plant);
    sw_common::step(&mut w);
    assert!(glasnost_sim::structures::engineer::takeover(
        &mut w, unit, plant
    ));
    assert_eq!(w.obj(plant).unwrap().owner, Some(ALPHA), "taken over");
    ends_at_the_event(&mut w, a, "building takeover");
}

// spec: S-0109 claim 22 (No detachment: an owner change that is not one of the takeovers, such as mind control, keeps it attached)
#[test]
fn an_owner_change_by_mind_control_keeps_it() {
    let mut w = game(true);
    let car = sw_common::spawn(&mut w, "CART", BETA, 0, 0);
    let a = attach(&mut w, car);
    sw_common::step(&mut w);
    assert!(mind::dominate(&mut w, car, ALPHA));
    assert_eq!(w.obj(car).unwrap().owner, Some(ALPHA));
    sw_common::run(&mut w, 3);
    assert!(drawn(&w, a));
    assert_eq!(w.anims.anims[&a].owner.map(|(o, _)| o), Some(car));
}

// spec: S-0109 claim 22 (the deleting update makes no infantry) and S-0611 claim 15 (Removal by another rule: the end of an attachment makes no infantry; with quirks.infantry_animation_stale_marks on it frees nothing, with it off it frees the last mark)
#[test]
fn an_infantry_making_animation_ended_with_its_owner() {
    for stale in [true, false] {
        let mut w = game(stale);
        let car = sw_common::spawn(&mut w, "CART", BETA, 0, 0);
        let a = attach_as(&mut w, car, "MAKER", Role::Ordinary);
        sw_common::run(&mut w, 2);
        let held = marks_of(&w, a);
        assert_eq!(held.len(), 1, "it marks its point");
        naval::start_sinking(&mut w, car);
        assert!(!drawn(&w, a));
        sw_common::step(&mut w);
        assert!(gone(&w, a), "deleted at its next update (stale {stale})");
        if stale {
            assert_eq!(marks_of(&w, a), held, "the mark stays taken");
        } else {
            assert!(marks_of(&w, a).is_empty(), "the last mark freed");
        }
        sw_common::run(&mut w, 40);
        assert!(grown(&w).is_empty(), "no infantry (stale {stale})");
    }
}

// spec: S-0109 claim 22 (End with the owner) and S-0522 claims 12 and 13 (the parachute recurrence holds while a parachute animation is attached)
#[test]
fn a_parachute_ended_with_its_trooper() {
    let mut w = game(true);
    let trooper = sw_common::spawn(&mut w, "WALKER", BETA, 0, 0);
    let a = attach_as(&mut w, trooper, "STAY", Role::Parachute);
    w.obj_mut(trooper).unwrap().fall.chute = true;
    sw_common::step(&mut w);
    assert_eq!(anims::parachute_of(&w, trooper), Some(a));
    objects::go_inside(&mut w, trooper);
    assert!(
        !w.obj(trooper).unwrap().fall.chute,
        "no parachute attached from the event on"
    );
    assert!(anims::parachute_of(&w, trooper).is_none());
    ends_at_the_event(&mut w, a, "parachute");
}

// spec: S-0608 claim 14 (the drain animation is removed when the link ends) and S-0109 claim 22 (an ended one too: it then leaves the update order at once)
#[test]
fn another_rules_removal_still_takes_an_ended_one_out_at_once() {
    let mut w = game(true);
    let car = sw_common::spawn(&mut w, "CART", BETA, 0, 0);
    let a = attach_as(&mut w, car, "STAY", Role::Drain(car));
    sw_common::step(&mut w);
    naval::start_sinking(&mut w, car);
    assert_eq!(anims::with_role(&w, Role::Drain(car)), vec![a]);
    anims::remove(&mut w, a);
    assert!(gone(&w, a), "out of the update order at once");
    assert!(anims::with_role(&w, Role::Drain(car)).is_empty());
}

/// Alpha's permanently controlled CART, Beta's before.
fn controlled(w: &mut World) -> ObjectId {
    let car = sw_common::spawn(w, "CART", BETA, 0, 0);
    assert!(mind::dominate(w, car, ALPHA));
    assert_eq!(mind::ring(w, car).map(|r| r.anim), Some("PERMA".into()));
    car
}

// spec: S-0109 Changes for other specs (the S-0601 claim 18 entry: while it is controlled, the ring shows whenever the object is on the map, not destroyed, not sinking or crashing and neither cloaking nor cloaked; shown again when the last of those holds again)
#[test]
fn the_ring_hides_while_cloaked_and_shows_again() {
    let mut w = game(true);
    let car = controlled(&mut w);
    start_cloaking(&mut w, car);
    assert!(
        mind::ring(&w, car).is_none(),
        "no ring from the start of cloaking"
    );
    sw_common::until(&mut w, 600, |w| cloak_state(w, car) == CloakState::Cloaked);
    assert!(mind::ring(&w, car).is_none(), "none while cloaked");
    w.obj_mut(car).unwrap().cloak.own = false;
    assert!(cloak::uncloak(&mut w, car, false));
    sw_common::until(&mut w, 600, |w| !hidden(cloak_state(w, car)));
    assert!(!hidden(cloak_state(&w, car)));
    assert!(mind::permanent(&w, car), "still controlled");
    assert!(
        mind::ring(&w, car).is_some(),
        "shown again once it uncloaks"
    );
}

// spec: S-0109 Changes for other specs (the S-0601 claim 18 entry: no ring off the map; shown again when it is back on the map)
#[test]
fn the_ring_hides_off_the_map_and_shows_again() {
    let mut w = game(true);
    let car = controlled(&mut w);
    objects::take_inside(&mut w, car);
    assert!(mind::ring(&w, car).is_none(), "none off the map");
    let p = Pos::centre(at(0, 3), 0);
    let z = w.surface_at(p.x, p.y);
    objects::place(&mut w, car, Pos::new(p.x, p.y, z), 0);
    assert!(mind::permanent(&w, car), "still controlled");
    assert!(mind::ring(&w, car).is_some(), "shown again on the map");
}

// spec: S-0109 Changes for other specs (the S-0601 claim 18 entry: no ring once destroyed or sinking)
#[test]
fn no_ring_once_sinking_or_destroyed() {
    let mut w = game(true);
    let car = controlled(&mut w);
    naval::start_sinking(&mut w, car);
    assert!(mind::ring(&w, car).is_none(), "none while sinking");
    let mut w = game(true);
    let car = controlled(&mut w);
    w.obj_mut(car).unwrap().combat.killed = true;
    assert!(mind::ring(&w, car).is_none(), "none once destroyed");
}
