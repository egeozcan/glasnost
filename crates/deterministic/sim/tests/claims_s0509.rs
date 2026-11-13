//! Claim tests of S-0509 (engineers and capture) on synthetic rules:
//! invented types and numbers, no game data.

mod structures_common;

use glasnost_sim::combat::effects::{Effect, Sound};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::structures::engineer;
use glasnost_sim::structures::entry;
use glasnost_sim::structures::ownership::{self, Cause};
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Target, World};
use structures_common::*;

fn counted() -> World {
    game_on(&ruleset(), triggers())
}

/// Puts an infantry unit of `name` and `house` in the cell of building `b`
/// (its top-left cell) with the building as destination (or target) in
/// `beh`, ready for its arrival update.
fn at_building(
    w: &mut World,
    name: &str,
    house: u8,
    b: ObjectId,
    beh: Behaviour,
    dest: bool,
) -> ObjectId {
    let c = w.obj(b).unwrap().pos.cell();
    let u = testkit::spawn(w, Kind::Infantry, name, house, c, 0);
    let t = w.tick;
    let o = w.obj_mut(u).unwrap();
    if dest {
        o.dest = Some(Target::Object(b));
    } else {
        o.target = Some(Target::Object(b));
    }
    o.beh.force(beh, t, true);
    u
}

fn capture_at(w: &mut World, name: &str, house: u8, b: ObjectId) -> (ObjectId, bool) {
    let u = at_building(w, name, house, b, Behaviour::Capture, true);
    let gone = entry::arrival(w, u);
    (u, gone)
}

fn health(w: &World, b: ObjectId) -> i32 {
    w.obj(b).unwrap().health
}

// spec: S-0509 claim 1 (a building entry: the behaviour is capture, area guard or patrol and the building is the unit's destination or its target; any other behaviour is none) and claim 2 (the general entry raise)
#[test]
fn claim_1_when_a_building_entry_happens() {
    for (beh, dest, entered) in [
        (Behaviour::Capture, true, true),
        (Behaviour::AreaGuard, false, true),
        (Behaviour::Patrol, true, true),
        (Behaviour::Guard, true, false),
        (Behaviour::Move, true, false),
    ] {
        let mut w = counted();
        let b = put(&mut w, Kind::Building, "TARGET", 1, 0, 0);
        tag(&mut w, b, "ENT");
        let u = at_building(&mut w, "ENGI", 0, b, beh, dest);
        let f = w.triggers.fired;
        let gone = entry::arrival(&mut w, u);
        assert_eq!(gone, entered, "{beh:?}");
        assert_eq!(w.triggers.fired > f, entered, "{beh:?}");
    }
}

// spec: S-0509 claim 1 (an occupier or assaulter type in capture whose destination is a building makes S-0508's attempt instead, even for an engineer and for a vehicle-like building: a refusal there does not become a takeover)
#[test]
fn claim_1_the_garrison_route_comes_first() {
    let mut w = counted();
    let enemy = put(&mut w, Kind::Building, "TARGET", 1, 0, 0);
    let (u, gone) = capture_at(&mut w, "ENGOCC", 0, enemy);
    assert!(!gone, "an occupier engineer is refused, not consumed");
    assert!(on_map(&w, u));
    assert_eq!(owner(&w, enemy), Some(B), "no capture");
    let vl = put(&mut w, Kind::Building, "VLIKE", 1, 6, 0);
    let (u, gone) = capture_at(&mut w, "ENGOCC", 0, vl);
    assert!(!gone && on_map(&w, u));
    assert_eq!(owner(&w, vl), Some(B), "no takeover after the refusal");
}

// spec: S-0509 claim 2 (the general entry raise: one raise at every entry whatever the outcome, none for the garrison route's refusal)
#[test]
fn claim_2_one_general_raise_for_every_outcome() {
    for (name, target, house, raises) in [
        ("ENGI", "TARGET", 0u8, 2), // capture: the general raise and the capture raise
        ("ENGI", "NOCAP", 1, 1),    // an own building at full health: refused
        ("SPYX", "TARGET", 1, 1),   // a spy of another type: S-0510 (placeholder: refused)
        ("WALKER", "TARGET", 1, 1), // any other unit: refused
    ] {
        let mut w = counted();
        let owner_house = if target == "NOCAP" { 0 } else { 1 };
        let b = put(&mut w, Kind::Building, target, owner_house, 0, 0);
        tag(&mut w, b, "ENT");
        let f = w.triggers.fired;
        let _ = house;
        let (_, _) = capture_at(&mut w, name, 0, b);
        assert_eq!(w.triggers.fired - f, raises, "{name} at {target}");
    }
}

// spec: S-0509 claim 3 row 2 and claim 4 (an arrival-friendly building at full health: refused; the engineer keeps its destination cleared and scatters, and stays)
#[test]
fn claim_3_row_2_and_claim_4_full_health_refusal() {
    let mut w = game();
    let b = put(&mut w, Kind::Building, "TARGET", 0, 0, 0);
    let (u, gone) = capture_at(&mut w, "ENGI", 0, b);
    assert!(!gone && on_map(&w, u));
    let o = w.obj(u).unwrap();
    assert_ne!(o.dest, Some(Target::Object(b)), "destination cleared");
    assert!(matches!(o.dest, Some(Target::Cell(_))), "scattering");
    assert_eq!(owner(&w, b), Some(A));
    // An allied building (one-way, read from the engineer's house) and a passive garrisonable one are friendly too.
    let mut w = game();
    w.houses[0].allies = vec![A, B];
    let ally = put(&mut w, Kind::Building, "TARGET", 1, 0, 0);
    let e = put(&mut w, Kind::Infantry, "ENGI", 0, 6, 6);
    assert!(engineer::arrival_friendly(&w, e, ally));
    let mut w = game();
    let hut = put(&mut w, Kind::Building, "HUT", 2, 0, 0);
    let target = put(&mut w, Kind::Building, "TARGET", 2, 6, 0);
    let e = put(&mut w, Kind::Infantry, "ENGI", 0, 6, 6);
    assert!(
        engineer::arrival_friendly(&w, e, hut),
        "garrisonable, passive owner"
    );
    assert!(
        !engineer::arrival_friendly(&w, e, target),
        "not garrisonable"
    );
}

// spec: S-0509 claim 3 row 3 and claim 5 (an arrival-friendly damaged building: repaired, health becomes strength, its own repair switch is switched off, the building-repaired sound plays at its centre, the engineer is used up)
#[test]
fn claim_5_repair() {
    let mut w = counted();
    let b = put(&mut w, Kind::Building, "TARGET", 0, 0, 0);
    w.obj_mut(b).unwrap().health = 120;
    w.obj_mut(b).unwrap().econ.building.repairing = true;
    let before = w.houses[0].combat.losses;
    w.effects.clear();
    let eng = at_building(&mut w, "ENGI", 0, b, Behaviour::Capture, true);
    tag(&mut w, eng, "DIE");
    assert!(entry::arrival(&mut w, eng));
    assert_eq!(health(&w, b), 400);
    assert!(!w.obj(b).unwrap().econ.building.repairing);
    assert!(w.obj(eng).is_none(), "used up");
    assert_eq!(
        w.houses[0].combat.losses, before,
        "removed, not killed: no loss"
    );
    assert!(w.triggers.variables[1], "its destroyed raise");
    assert!(w.effects.iter().any(|e| matches!(
        e,
        Effect::Sound {
            sound: Sound::BuildingRepaired,
            pos: Some(_),
            ..
        }
    )));
}

// spec: S-0509 claim 3 row 7 and claim 7 (any other building is captured at any health, full health included, in a skirmish)
#[test]
fn claim_7_capture_at_any_health() {
    for h in [400, 123, 1] {
        let mut w = game();
        let b = put(&mut w, Kind::Building, "TARGET", 1, 0, 0);
        w.obj_mut(b).unwrap().health = h;
        let (u, gone) = capture_at(&mut w, "ENGI", 0, b);
        assert!(gone && w.obj(u).is_none(), "health {h}");
        assert_eq!(owner(&w, b), Some(A), "health {h}");
        assert_eq!(health(&w, b), h, "health is kept");
    }
}

// spec: S-0509 claim 3 row 5 and Edge cases (an entry into a building that is neither arrival-friendly nor capturable: the engineer is used up and nothing changes; quirks.engineer_lost_on_uncapturable off: refused and stays)
#[test]
fn claim_3_row_5_and_its_quirk() {
    for on in [true, false] {
        let mut w = game();
        let mut rules = (*w.rules_arc()).clone();
        rules.quirks.engineer_lost_on_uncapturable = on;
        w.rules = Some(std::sync::Arc::new(rules));
        let b = put(&mut w, Kind::Building, "NOCAP", 1, 0, 0);
        let (u, gone) = capture_at(&mut w, "ENGI", 0, b);
        assert_eq!(gone, on, "quirk {on}");
        assert_eq!(w.obj(u).is_none(), on);
        assert_eq!(owner(&w, b), Some(B));
        assert_eq!(health(&w, b), 400);
    }
}

// spec: S-0509 claim 3 row 6 (not arrival-friendly, being sold: refused, the engineer stays)
#[test]
fn claim_3_row_6_a_building_being_sold() {
    let mut w = game();
    let b = put(&mut w, Kind::Building, "TARGET", 1, 0, 0);
    let t = w.tick;
    w.obj_mut(b).unwrap().beh.force(Behaviour::Selling, t, true);
    let (u, gone) = capture_at(&mut w, "ENGI", 0, b);
    assert!(!gone && on_map(&w, u));
    assert_eq!(owner(&w, b), Some(B));
}

// spec: S-0509 Edge cases (quirks.engineer_repairs_during_sale: an engineer reaching a damaged friendly building during its sale repairs it and is used up; at full health it is refused; with the switch off the sale-time entry is refused whatever the health)
#[test]
fn edge_repair_during_a_sale() {
    for on in [true, false] {
        let mut w = game();
        let mut rules = (*w.rules_arc()).clone();
        rules.quirks.engineer_repairs_during_sale = on;
        w.rules = Some(std::sync::Arc::new(rules));
        let b = put(&mut w, Kind::Building, "TARGET", 0, 0, 0);
        w.obj_mut(b).unwrap().health = 100;
        let t = w.tick;
        w.obj_mut(b).unwrap().beh.force(Behaviour::Selling, t, true);
        let (u, gone) = capture_at(&mut w, "ENGI", 0, b);
        assert_eq!(gone, on, "quirk {on}");
        assert_eq!(health(&w, b) == 400, on);
        assert_eq!(w.obj(u).is_none(), on);
    }
}

// spec: S-0509 claim 3 and F2 (the damaging hit of the multi-engineer rule: A = trunc(Y) if Y < D, else D, with D = H − trunc(S × c × 0.5) and Y = ((1 − c × 0.5) × 0.5) × S; the engineer is used up, the building is not captured); in a skirmish the rule is always off (claim 3 row 4 never applies)
#[test]
fn claim_6_the_damaging_hit_and_the_rule_being_off() {
    let mut w = game();
    assert!(!engineer::multi_engineer(&w), "always off in a skirmish");
    // S = 400, c = 0.5: Y = 150; at H = 400, D = 300 and A = 150; at H = 200, D = 100 and A = 100.
    for (h, a) in [(400, 150), (200, 100)] {
        let b = put(&mut w, Kind::Building, "TARGET", 1, 0, 0);
        w.obj_mut(b).unwrap().health = h;
        let u = put(&mut w, Kind::Infantry, "ENGI", 0, 6, 6);
        engineer::damaging_hit(&mut w, u, b);
        assert_eq!(health(&w, b), h - a, "H {h}");
        assert_eq!(owner(&w, b), Some(B), "not captured");
        assert!(w.obj(u).is_none());
        glasnost_sim::objects::remove(&mut w, b);
    }
}

// spec: S-0509 claim 8 (a capture: two entry raises before its ownership and tag changes, the ownership change raises neither tag, the engineer's destroyed raise comes after the ownership change and acts on the tag the building now shares when the tag carried over)
#[test]
fn claim_8_capture_ordering() {
    let mut w = counted();
    let b = put(&mut w, Kind::Building, "TARGET", 1, 0, 0);
    tag(&mut w, b, "ENT");
    let f = w.triggers.fired;
    let eng = at_building(&mut w, "ENGI", 0, b, Behaviour::Capture, true);
    tag(&mut w, eng, "DIE_CARRY");
    assert!(entry::arrival(&mut w, eng));
    // Two entry raises on the old tag, then the engineer's destroyed raise: three firings.
    assert_eq!(w.triggers.fired - f, 3);
    assert!(
        w.triggers.variables[0],
        "the entry raises acted on the old tag"
    );
    assert!(
        w.triggers.variables[2],
        "the destroyed raise acted on the carried tag"
    );
    let carried = w.triggers.objects.get(&b).copied();
    assert_eq!(
        carried,
        w.triggers.map.object_tag(b"DIE_CARRY"),
        "the building received the engineer's tag"
    );
    assert_eq!(owner(&w, b), Some(A));
}

// spec: S-0509 claim 8 and S-0524 claim 10 (without the carry-over flag the old tag remains)
#[test]
fn claim_8_without_carry_over_the_old_tag_remains() {
    let mut w = counted();
    let b = put(&mut w, Kind::Building, "TARGET", 1, 0, 0);
    tag(&mut w, b, "ENT");
    let old = w.triggers.objects.get(&b).copied();
    let eng = at_building(&mut w, "ENGI", 0, b, Behaviour::Capture, true);
    tag(&mut w, eng, "DIE");
    assert!(entry::arrival(&mut w, eng));
    assert_eq!(w.triggers.objects.get(&b).copied(), old);
}

// spec: S-0509 claim 9 (after the ownership change every infantry unit standing on the captured building whose destination is none or the building receives a scatter request: forced, move-anywhere, no threat point; units of every house count)
#[test]
fn claim_9_infantry_standing_on_a_captured_building_scatter() {
    let mut w = game();
    let b = put(&mut w, Kind::Building, "TARGET", 1, 0, 0);
    let c = w.obj(b).unwrap().pos.cell();
    let idle = testkit::spawn(
        &mut w,
        Kind::Infantry,
        "WALKER",
        1,
        glasnost_map::CellXY::new(c.x + 1, c.y),
        0,
    );
    let busy = testkit::spawn(
        &mut w,
        Kind::Infantry,
        "WALKER",
        1,
        glasnost_map::CellXY::new(c.x, c.y + 1),
        0,
    );
    w.obj_mut(busy).unwrap().dest = Some(Target::Cell(glasnost_map::CellXY::new(c.x + 9, c.y + 9)));
    let busy_dest = w.obj(busy).unwrap().dest;
    let (_, gone) = capture_at(&mut w, "ENGI", 0, b);
    assert!(gone);
    assert!(
        matches!(w.obj(idle).unwrap().dest, Some(Target::Cell(_))),
        "an idle unit on the building scatters"
    );
    assert_eq!(
        w.obj(busy).unwrap().dest,
        busy_dest,
        "one with another destination does not"
    );
}

// spec: S-0509 claim 10 (a used-up engineer is removed, not killed: no loss or kill counted, no experience)
#[test]
fn claim_10_the_used_up_engineer_is_not_killed() {
    let mut w = game();
    let b = put(&mut w, Kind::Building, "TARGET", 1, 0, 0);
    let before: Vec<_> = w
        .houses
        .iter()
        .map(|h| (h.combat.kills, h.combat.losses, h.combat.score))
        .collect();
    let (u, gone) = capture_at(&mut w, "ENGI", 0, b);
    assert!(gone && w.obj(u).is_none());
    let after: Vec<_> = w
        .houses
        .iter()
        .map(|h| (h.combat.kills, h.combat.losses, h.combat.score))
        .collect();
    assert_eq!(before, after);
}

// spec: S-0509 claim 11 (the lost-to-capture mark is set on the house that owned the building when an engineer captures it; a takeover, a trigger transfer and every other ownership change leave it alone; nothing clears it)
#[test]
fn claim_11_the_lost_to_capture_mark() {
    let mut w = game();
    assert!(!engineer::lost_to_capture(&w, B));
    let b = put(&mut w, Kind::Building, "TARGET", 1, 0, 0);
    capture_at(&mut w, "ENGI", 0, b);
    assert!(engineer::lost_to_capture(&w, B));
    assert!(!engineer::lost_to_capture(&w, A));
    // Taking it back by a trigger transfer marks nobody; the mark persists.
    ownership::change(&mut w, b, Some(B), Cause::Trigger);
    assert!(!engineer::lost_to_capture(&w, A));
    assert!(engineer::lost_to_capture(&w, B));
    testkit::run(&mut w, 50);
    assert!(engineer::lost_to_capture(&w, B));
    // A vehicle-like takeover does not set it.
    let mut w = game();
    let v = put(&mut w, Kind::Building, "VLIKE", 1, 0, 0);
    let u = at_building(&mut w, "WALKER", 0, v, Behaviour::Capture, true);
    assert!(entry::arrival(&mut w, u));
    assert!(!engineer::lost_to_capture(&w, B));
}

// spec: S-0509 claim 12 (a building ownership change runs the effects the owning specs give: the counts and acquisition order, the captured mark; the capture announcement is requested for a capture and a takeover, not for a trigger or the occupation rules)
#[test]
fn claim_12_effects_and_announcement_eligibility() {
    let mut w = game();
    let b = put(&mut w, Kind::Building, "TARGET", 1, 0, 0);
    assert!(w.houses[1].econ.acquired.contains(&b));
    w.effects.clear();
    capture_at(&mut w, "ENGI", 0, b);
    assert!(w.houses[0].econ.acquired.contains(&b));
    assert!(!w.houses[1].econ.acquired.contains(&b));
    assert!(
        w.obj(b).unwrap().combat.captured,
        "the captured mark of S-0314"
    );
    let ev = |w: &World| {
        w.effects
            .iter()
            .filter_map(|e| match e {
                Effect::Captured {
                    old, new, announce, ..
                } => Some((*old, *new, *announce)),
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(ev(&w), vec![(Some(B), Some(A), true)]);
    w.effects.clear();
    ownership::change(&mut w, b, Some(B), Cause::Trigger);
    assert_eq!(ev(&w), vec![(Some(A), Some(B), false)]);
    w.effects.clear();
    ownership::change(&mut w, b, Some(B), Cause::Capture);
    assert!(
        ev(&w).is_empty(),
        "no effect when the new house is the old one"
    );
}

// spec: S-0509 claim 12 (table row: the primary-building mark cleared at a building ownership change) (#223)
#[test]
fn claim_12_primary_mark_cleared() {
    let mut w = game();
    let b = put(&mut w, Kind::Building, "TARGET", 1, 0, 0);
    w.obj_mut(b).unwrap().econ.building.primary = true;
    capture_at(&mut w, "ENGI", 0, b);
    assert_eq!(owner(&w, b), Some(A), "captured");
    assert!(
        !w.obj(b).unwrap().econ.building.primary,
        "a capture clears it"
    );
    w.obj_mut(b).unwrap().econ.building.primary = true;
    ownership::change(&mut w, b, Some(B), Cause::Trigger);
    assert!(
        !w.obj(b).unwrap().econ.building.primary,
        "a trigger transfer clears it"
    );
    w.obj_mut(b).unwrap().econ.building.primary = true;
    ownership::change(&mut w, b, Some(B), Cause::Capture);
    assert!(
        w.obj(b).unwrap().econ.building.primary,
        "no change of owner: the mark stays"
    );
}

// spec: S-0509 claim 14 (the objects in contact with the building: a staying one near the docking point changes owner with it and keeps its contact; a moving one, or one far away, is told to leave and loses its contact)
#[test]
fn claim_14_contacts_follow_or_leave() {
    let mut w = game();
    let b = put(&mut w, Kind::Building, "TARGET", 1, 0, 0);
    let rp = w.reference_point(b);
    // A vehicle standing at the docking point (the building's reference point), one 200 leptons away, and a moving one.
    let near = testkit::spawn(&mut w, Kind::Vehicle, "CART", 1, at(0, 0), 0);
    let far = testkit::spawn(&mut w, Kind::Vehicle, "CART", 1, at(5, 5), 0);
    let moving = testkit::spawn(&mut w, Kind::Vehicle, "CART", 1, at(0, 0), 0);
    w.obj_mut(near).unwrap().pos = rp;
    for v in [near, far, moving] {
        w.obj_mut(v).unwrap().contact = Some(b);
    }
    w.obj_mut(far).unwrap().dest = None;
    w.obj_mut(moving).unwrap().pos = rp;
    w.obj_mut(moving).unwrap().dest = Some(Target::Cell(at(9, 9)));
    w.obj_mut(moving).unwrap().path.steps = vec![0];
    ownership::change(&mut w, b, Some(A), Cause::Capture);
    assert_eq!(
        owner(&w, near),
        Some(A),
        "staying and near: changes owner with the building"
    );
    assert_eq!(w.obj(near).unwrap().contact, Some(b), "keeps its contact");
    assert_eq!(owner(&w, far), Some(B));
    assert_eq!(w.obj(far).unwrap().contact, None, "far: told to leave");
    assert_eq!(owner(&w, moving), Some(B));
    assert_eq!(
        w.obj(moving).unwrap().contact,
        None,
        "moving: told to leave"
    );
}

// spec: S-0509 claim 17 (any infantry unit in capture, area guard or patrol whose destination is a vehicle-like building takes it over: one entry raise seeing the old owner, the ownership change, the scatter, the unit used up; neither alliance, ownership nor the unit's type plays a part)
#[test]
fn claim_17_takeover_of_a_vehicle_like_building() {
    let mut w = counted();
    let v = put(&mut w, Kind::Building, "VLIKE", 1, 0, 0);
    tag(&mut w, v, "ENT");
    let f = w.triggers.fired;
    let spy = at_building(&mut w, "SPYX", 0, v, Behaviour::Capture, true);
    tag(&mut w, spy, "DIE");
    assert!(entry::arrival(&mut w, spy));
    assert_eq!(
        w.triggers.fired - f,
        2,
        "one entry raise and the unit's destroyed raise"
    );
    assert_eq!(owner(&w, v), Some(A));
    assert!(w.obj(spy).is_none());
    // Alliance and ownership are not tested: an own-house building is taken over by its own unit too (no change).
    let mut w = game();
    w.houses[0].allies = vec![A, B];
    let v = put(&mut w, Kind::Building, "VLIKE", 1, 0, 0);
    let u = at_building(&mut w, "WALKER", 0, v, Behaviour::AreaGuard, true);
    assert!(entry::arrival(&mut w, u));
    assert_eq!(owner(&w, v), Some(A), "even an allied building");
}

// spec: S-0509 claim 3 row 1 and claim 2 (a bridge repair hut, whoever owns it: the engineer's entry is S-0511's, whose entered-by raise is the general raise: the hut gets no second one)
#[test]
fn claim_3_row_1_the_hut_gets_one_raise() {
    let mut w = counted();
    let hut = put(&mut w, Kind::Building, "HUTB", 1, 0, 0);
    tag(&mut w, hut, "ENT");
    let f = w.triggers.fired;
    let (_, gone) = capture_at(&mut w, "ENGI", 0, hut);
    assert!(gone, "the engineer is consumed by its entry");
    assert_eq!(
        w.triggers.fired - f,
        1,
        "one entry raise (the hut's), no second one and none for a capture"
    );
    assert_eq!(owner(&w, hut), Some(B), "a hut is not captured");
}

// spec: S-0011 claim 24 (change owner: every present object carrying the tag changes its owner to the resolved house, without announcement; S-0509 claim 12 effects run)
#[test]
fn s0011_claim_24_change_owner_action() {
    use glasnost_map::triggers::{Action, Event, Triggers};
    let ev = Event {
        code: 1,
        kind: 0,
        value: -1,
        type_name: None,
    };
    // House value 0: the first house playing country 0 (Alpha).
    let t = trigger(
        "OWN",
        ev,
        vec![Action {
            code: 14,
            number: 0,
            ..Default::default()
        }],
    );
    let tr = Triggers {
        tags: vec![tag_def(0, &t, 2)],
        triggers: vec![t],
        ..Default::default()
    };
    let mut w = game_on(&ruleset(), tr);
    let b = put(&mut w, Kind::Building, "TARGET", 1, 0, 0);
    let u = put(&mut w, Kind::Infantry, "WALKER", 1, 6, 6);
    let other = put(&mut w, Kind::Building, "TARGET", 1, 8, 0);
    tag(&mut w, b, "OWN");
    tag(&mut w, u, "OWN");
    w.effects.clear();
    glasnost_sim::mp_triggers::entry_raise(&mut w, b, u);
    assert_eq!(owner(&w, b), Some(A));
    assert_eq!(owner(&w, u), Some(A));
    assert_eq!(owner(&w, other), Some(B), "untagged: unchanged");
    assert!(
        w.effects
            .iter()
            .all(|e| !matches!(e, Effect::Captured { announce: true, .. }))
    );
}

// spec: S-0509 claims 1, 7 and 8 and S-0214 table M (an engineer ordered to capture walks to the building, enters on arrival in the capture behaviour, captures it and is used up, all by orders through the arrival update)
#[test]
fn claim_1_a_walking_engineer_captures() {
    let mut w = counted();
    let b = put(&mut w, Kind::Building, "TARGET", 1, 0, 0);
    tag(&mut w, b, "ENT");
    let e = put(&mut w, Kind::Infantry, "ENGI", 0, -8, 0);
    send(&mut w, e, b);
    let t = until(&mut w, 800, |w| w.obj(e).is_none()).expect("arrives and is used up");
    assert!(t > 10);
    assert_eq!(owner(&w, b), Some(A));
    assert!(engineer::lost_to_capture(&w, B));
    assert!(w.triggers.variables[0], "the entry raises");
}

// spec: S-0108 claim 26 (Q-2186: a change made in the turn of an object that stands before the building in the update order comes before the building's slots' turns, one made by an object after it, after them; a trigger's change and the occupation rules' come before; with no actor, the announcing causes come after)
#[test]
fn s0108_claim_26_the_change_says_whether_it_follows_the_slots() {
    let after_slots = |w: &World| {
        w.effects
            .iter()
            .filter_map(|e| match e {
                Effect::Captured { after_slots, .. } => Some(*after_slots),
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    // An engineer that entered play before the building (a map's engineer).
    let mut w = game();
    let early = put(&mut w, Kind::Infantry, "ENGI", 0, 5, 5);
    let b = put(&mut w, Kind::Building, "TARGET", 1, 0, 0);
    // One that entered after it (a trained or a starting engineer).
    let late = put(&mut w, Kind::Infantry, "ENGI", 0, 6, 6);
    w.effects.clear();
    ownership::change_by(&mut w, b, Some(A), Cause::Capture, Some(early));
    assert_eq!(after_slots(&w), [false], "an engineer before the building");
    w.effects.clear();
    ownership::change_by(&mut w, b, Some(B), Cause::Capture, Some(late));
    assert_eq!(after_slots(&w), [true], "an engineer after the building");
    // A trigger's change, the occupation rules' and an actorless capture.
    for (cause, expect) in [
        (Cause::Trigger, false),
        (Cause::Occupation, false),
        (Cause::Capture, true),
    ] {
        w.effects.clear();
        let to = if w.obj(b).unwrap().owner == Some(A) {
            B
        } else {
            A
        };
        ownership::change(&mut w, b, Some(to), cause);
        assert_eq!(after_slots(&w), [expect], "{cause:?}");
    }
}
