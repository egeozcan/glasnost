//! Claim tests of S-0502 (movement of jumpjets): occupancy, hovering,
//! landing, orders, stopped landings and the neighbour step.

mod jump_common;

use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_num::F64;
use glasnost_sim::cells::Marker;
use glasnost_sim::jump::{self, JumpState, Phase};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{Layer, ObjectId, Pos, Target, World};
use jump_common::*;

fn j(w: &World, id: ObjectId) -> &JumpState {
    w.obj(id).unwrap().jump.as_ref().unwrap()
}

fn js(w: &mut World, id: ObjectId) -> &mut JumpState {
    w.obj_mut(id).unwrap().jump.as_mut().unwrap()
}

fn quirk(w: &mut World, f: impl FnOnce(&mut glasnost_rules::quirks::Quirks)) {
    let mut rules = (*w.rules_arc()).clone();
    f(&mut rules.quirks);
    w.rules = Some(std::sync::Arc::new(rules));
}

fn enemy(w: &mut World) -> ObjectId {
    put(w, "CART", Kind::Vehicle, 1, 18, 18)
}

/// A hovering jumpjet holding a move order to its own cell and a target.
fn hoverer(w: &mut World, name: &str, kind: Kind, dx: i32, dy: i32) -> ObjectId {
    let e = enemy(w);
    let u = put(w, name, kind, 0, dx, dy);
    go(w, u, at(dx, dy));
    let d = Pos::centre(at(dx, dy), 0);
    w.obj_mut(u).unwrap().target = Some(Target::Object(e));
    // The behaviour processing must not move the destination.
    let t = w.tick;
    w.obj_mut(u).unwrap().beh.force(Behaviour::Move, t, true);
    jump::start_in_flight(w, u, Phase::Hovering, 500);
    js(w, u).dest = Some(d);
    js(w, u).moving = true; // the commitment of a completed flight (the own-cell order itself is not accepted, claim 23)
    js(w, u).order = Some(Target::Cell(at(dx, dy)));
    // spec: S-0502 claim 23 (r24, Q-3043: an infantry unit's own-cell assignment above changes nothing, so the order-level destination is set as an order given elsewhere leaves it once the flight arrives)
    w.obj_mut(u).unwrap().dest = Some(Target::Cell(at(dx, dy)));
    u
}

/// A jumpjet descending over its destination cell, `height` above the
/// surface, with no target.
fn descender(w: &mut World, name: &str, kind: Kind, dx: i32, dy: i32, height: i32) -> ObjectId {
    let u = put(w, name, kind, 0, dx, dy);
    go(w, u, at(dx, dy));
    let t = w.tick;
    w.obj_mut(u).unwrap().beh.force(Behaviour::Move, t, true);
    jump::start_in_flight(w, u, Phase::Descending, height);
    js(w, u).dest = Some(Pos::centre(at(dx, dy), 0));
    js(w, u).order = Some(Target::Cell(at(dx, dy))); // as the order itself would have left it
    // spec: S-0502 claim 23 (r24, Q-3043: an infantry unit's own-cell assignment above changes nothing, so the order-level destination is set as an order given elsewhere leaves it)
    w.obj_mut(u).unwrap().dest = Some(Target::Cell(at(dx, dy)));
    js(w, u).ht = 0;
    u
}

fn record(w: &World, c: CellXY, l: Layer) -> glasnost_sim::cells::Occupancy {
    *w.cells.occ(c, l).unwrap()
}

fn marked_by(w: &World, c: CellXY, id: ObjectId) -> bool {
    record(w, c, Layer::Ground)
        .marks
        .iter()
        .flatten()
        .any(|m| matches!(m.by, Marker::Infantry(who, _) if who == id))
}

// spec: S-0502 claim 4 and scenario J3 (on flat land a non-hover, non-cruise placement lists and marks at heights 1 and 50 and not at 300; the first rise clears the old mark but does not forbid a later low replacement; a hovering or cruising jumpjet makes no new mark; a descending one holds its landing reservation; touchdown marks its place)
#[test]
fn claim_4_occupancy_marks() {
    let mut w = game();
    let u = put(&mut w, "JJ", Kind::Infantry, 0, -4, 0);
    let v = put(&mut w, "JV", Kind::Vehicle, 0, 4, 0);
    let (cu, cv) = (pos(&w, u).cell(), pos(&w, v).cell());
    assert!(marked_by(&w, cu, u), "grounded: its standing position");
    assert_eq!(
        record(&w, cv, Layer::Ground).vehicles,
        1,
        "a vehicle's flag"
    );
    go(&mut w, u, at(8, 0));
    go(&mut w, v, at(8, 2));
    step(&mut w);
    assert_eq!(phase(&w, u), Phase::Ascending);
    assert!(
        marked_by(&w, cu, u),
        "still marked at height 0 in the take-off update"
    );
    // Low ascent places again at every height below 208 and not above it.
    let mut low = 0;
    let mut high = 0;
    for _ in 0..60 {
        step(&mut w);
        let h = height(&w, u);
        let c = pos(&w, u).cell();
        let listed = w.cells.list(c, Layer::Ground).contains(&u);
        let marked = marked_by(&w, c, u);
        let (cv2, hv) = (pos(&w, v).cell(), height(&w, v));
        if h > 0 && h < 208 {
            assert!(listed && marked, "low ascent lists and marks it at {h}");
            low += 1;
        } else if h >= 208 {
            assert!(!listed && !marked, "no placement at {h}");
            assert!(j(&w, u).listed.is_none());
            high += 1;
        }
        if hv > 0 && hv < 208 {
            assert_eq!(
                record(&w, cv2, Layer::Ground).vehicles,
                1,
                "the flag at {hv}"
            );
        } else if hv >= 208 {
            assert_eq!(
                record(&w, cv2, Layer::Ground).vehicles,
                0,
                "no flag at {hv}"
            );
            assert!(!w.cells.list(cv2, Layer::Ground).contains(&v));
        }
    }
    assert!(low > 10 && high > 0);
    assert_eq!(
        record(&w, cv, Layer::Ground).vehicles,
        0,
        "the first cell is clear"
    );
    // Cruising: no place anywhere.
    let c = cruiser_over(&mut w);
    let cells: Vec<CellXY> = w.cells.grid.iter().map(|(c, _)| c).collect();
    for k in cells {
        assert!(
            !marked_by(&w, k, c),
            "a cruising jumpjet marks no cell ({k:?})"
        );
    }
    // Descending with a reservation, then touchdown.
    let d = descender(&mut w, "JJ", Kind::Infantry, 8, 8, 30);
    step(&mut w);
    let cd = Pos::centre(at(8, 8), 0).cell();
    assert!(j(&w, d).reservation.is_some());
    assert!(
        marked_by(&w, cd, d),
        "the landing reservation marks the landing cell"
    );
    for _ in 0..10 {
        step(&mut w);
    }
    assert_eq!(phase(&w, d), Phase::Grounded);
    assert!(j(&w, d).reservation.is_none() && j(&w, d).mark.is_some());
    assert!(marked_by(&w, cd, d));
    assert!(
        w.cells.list(cd, Layer::Ground).contains(&d),
        "listed again on the ground"
    );
}

fn cruiser_over(w: &mut World) -> ObjectId {
    let c = put(w, "JJ", Kind::Infantry, 0, -2, 6);
    go(w, c, at(14, 6));
    jump::start_in_flight(w, c, Phase::Cruising, 500);
    c
}

// spec: S-0502 claim 13 and scenario J8 (hovering rows: a destination elsewhere: cruise and release the slot; a target: keep hovering; with no target an ordinary jumpjet releases its slot and descends; a balloon-hover type keeps hovering)
#[test]
fn claim_13_hovering_rows() {
    let mut w = game();
    // Row 1: a destination elsewhere.
    let a = hoverer(&mut w, "JJ", Kind::Infantry, -4, 0);
    assert!(w.jump_slots.contains_key(&pos(&w, a).cell()));
    go(&mut w, a, at(2, 0));
    step(&mut w);
    assert_eq!(phase(&w, a), Phase::Cruising);
    assert!(
        j(&w, a).slot.is_none() && w.jump_slots.is_empty(),
        "the slot is released"
    );
    // Row 2: a target keeps it hovering.
    let b = hoverer(&mut w, "JJ", Kind::Infantry, -8, 4);
    for _ in 0..30 {
        step(&mut w);
    }
    assert_eq!(phase(&w, b), Phase::Hovering);
    // Row 6: no target: it releases its slot and descends.
    w.obj_mut(b).unwrap().target = None;
    step(&mut w);
    assert_eq!(phase(&w, b), Phase::Descending);
    assert!(j(&w, b).slot.is_none());
    assert_eq!(j(&w, b).ht, 0);
    // Row 5: a balloon-hover vehicle keeps hovering without a target.
    let c = hoverer(&mut w, "BALLOON", Kind::Vehicle, 8, 4);
    w.obj_mut(c).unwrap().target = None;
    for _ in 0..30 {
        step(&mut w);
    }
    assert_eq!(phase(&w, c), Phase::Hovering);
    // Precedence: balloon-hover and simple deployer, without the landing flag: the earlier row descends.
    let d = hoverer(&mut w, "BDEPN", Kind::Vehicle, 8, -4);
    w.obj_mut(d).unwrap().target = None;
    step(&mut w);
    assert_eq!(phase(&w, d), Phase::Descending);
}

// spec: S-0502 claim 13 and S-0517 claim 5 (a deploy-to-land vehicle with no landing request keeps hovering with its target height set to H; with a request, and any other simple-deployer vehicle, it descends)
#[test]
fn claim_13_deployer_rows() {
    let mut w = game();
    let a = hoverer(&mut w, "DEPL", Kind::Vehicle, -4, 0);
    w.obj_mut(a).unwrap().target = None;
    js(&mut w, a).ht = 300;
    step(&mut w);
    assert_eq!(phase(&w, a), Phase::Hovering);
    assert_eq!(j(&w, a).ht, 500, "the target height is set to H");
    js(&mut w, a).landing_request = true;
    step(&mut w);
    assert_eq!(phase(&w, a), Phase::Descending);
    let b = hoverer(&mut w, "DEPN", Kind::Vehicle, 4, 0);
    w.obj_mut(b).unwrap().target = None;
    step(&mut w);
    assert_eq!(phase(&w, b), Phase::Descending);
}

// spec: S-0502 claim 14 and scenario J9 (a descending jumpjet whose destination is water or beach does not land: it claims the hover slot, sets its target height to H and hovers; a current or queued enter behaviour lands)
#[test]
fn claim_14_no_landing_on_water_or_beach() {
    for l in [land::WATER, land::BEACH] {
        let mut w = game();
        let u = descender(&mut w, "JJ", Kind::Infantry, 4, 0, 300);
        w.cells.grid.get_mut(at(4, 0)).unwrap().land = l;
        // The unit is a cell away: a descending jumpjet's physics recomputes the land of the cell under it (claim 26).
        let p = pos(&w, u);
        w.obj_mut(u).unwrap().pos = Pos::new(p.x - 256, p.y, p.z);
        step(&mut w);
        assert_eq!(phase(&w, u), Phase::Hovering, "land {l}");
        assert_eq!(j(&w, u).ht, 500);
        assert!(w.jump_slots.contains_key(&at(4, 0)));
        for _ in 0..40 {
            step(&mut w);
        }
        assert_ne!(phase(&w, u), Phase::Grounded);
        assert!(
            height(&w, u) > 100,
            "it never lands there: it bobs between hovering and descending"
        );
    }
    // Entering: it lands on beach.
    let mut w = game();
    let u = descender(&mut w, "JJ", Kind::Infantry, 4, 0, 60);
    w.cells.grid.get_mut(at(4, 0)).unwrap().land = land::BEACH;
    w.obj_mut(u).unwrap().beh.queue(Some(Behaviour::Enter));
    for _ in 0..30 {
        step(&mut w);
    }
    assert_eq!(phase(&w, u), Phase::Grounded);
    // A held slot: a neighbour step and cruising, one draw.
    let mut w = game();
    let other = hoverer(&mut w, "JJ", Kind::Infantry, 4, 0);
    assert!(j(&w, other).slot.is_some());
    let u = descender(&mut w, "JJ", Kind::Infantry, 4, 0, 60);
    w.cells.grid.get_mut(at(4, 0)).unwrap().land = land::WATER;
    // The unit is a cell away: a descending jumpjet's physics recomputes the land of the cell under it (claim 26).
    let p = pos(&w, u);
    w.obj_mut(u).unwrap().pos = Pos::new(p.x - 256, p.y, p.z);
    let mut r = w.rng.clone();
    let half = w.rules().quirks.ranged_draw_half_span;
    let draw = r.ranged(0, 7, half);
    let _ = r.ranged(0, 3, half); // spec: S-0502 claims 21 and 23 (the infantry point adjustment's own draw follows the neighbour draw)
    jump::tick(&mut w, u);
    assert_eq!(phase(&w, u), Phase::Cruising);
    assert_eq!(
        w.rng.taken(),
        r.taken(),
        "one ranged draw over 0 to 7, then the adjustment's draw over 0 to 3"
    );
    let Some(Target::Cell(n)) = w.obj(u).unwrap().dest else {
        panic!("a destination cell")
    };
    assert_eq!(
        direction(at(3, 0), n),
        draw as usize,
        "from the unit's own cell"
    );
}

// spec: S-0502 claim 15 and scenario J9 (touchdown at bridge-adjusted height 0: speed fraction 0, position the destination point, its mark placed, destination and move order cleared, grounded, slot released; the descent is by c per tick)
#[test]
fn claim_15_touchdown() {
    let mut w = game();
    let u = descender(&mut w, "JJ", Kind::Infantry, 4, 0, 23);
    let mut hs = Vec::new();
    for _ in 0..8 {
        step(&mut w);
        hs.push(height(&w, u));
        if phase(&w, u) == Phase::Grounded {
            break;
        }
    }
    // 23, 18, 13, 8, 3 then 0 (the descent snaps within one step).
    assert_eq!(hs, vec![18, 13, 8, 3, 0], "descent by c = 5 per tick");
    assert_eq!(phase(&w, u), Phase::Grounded);
    let o = w.obj(u).unwrap();
    let s = j(&w, u);
    assert!(s.fraction.eq_num(F64::ZERO) && s.dest.is_none() && !s.moving && s.slot.is_none());
    assert!(o.dest.is_none(), "the order-level destination is cleared");
    assert_eq!(
        (o.pos.x, o.pos.y),
        (Pos::centre(at(4, 0), 0).x, Pos::centre(at(4, 0), 0).y)
    );
    assert!(s.mark.is_some() && marked_by(&w, o.pos.cell(), u));
}

// spec: S-0502 claim 15 and scenario J9 (a jumpjet that ends on a high-bridge cell lands on the deck at bridge-adjusted height 0: its position is the deck point and it is on the deck layer)
#[test]
fn claim_15_touchdown_on_a_deck() {
    let mut w = game();
    let c = at(4, 0);
    let i = w.cells.idx(c).unwrap();
    w.cells.deck[i] = true;
    let u = put(&mut w, "JJ", Kind::Infantry, 0, 4, 2);
    // A point on the deck: terrain height 0 plus 416.
    let t = w.tick;
    w.obj_mut(u).unwrap().beh.force(Behaviour::Move, t, true);
    jump::start_in_flight(&mut w, u, Phase::Descending, 600);
    w.obj_mut(u).unwrap().pos = Pos::new(Pos::centre(c, 0).x, Pos::centre(c, 0).y, 600);
    js(&mut w, u).dest = Some(Pos::centre(c, 416));
    js(&mut w, u).ht = 0;
    w.obj_mut(u).unwrap().dest = Some(Target::Cell(c));
    js(&mut w, u).order = Some(Target::Cell(c));
    for _ in 0..60 {
        step(&mut w);
        if phase(&w, u) == Phase::Grounded {
            break;
        }
    }
    assert_eq!(phase(&w, u), Phase::Grounded);
    let o = w.obj(u).unwrap();
    assert_eq!(o.pos.z, 416, "on the deck, not on the terrain beneath it");
    assert_eq!(o.layer, Layer::Deck);
    assert!(w.cells.list(c, Layer::Deck).contains(&u));
    assert!(j(&w, u).on_deck);
    // Taking off from the deck: the old mark is cleared and it is no longer on a deck (claim 4). Over the high bridge, with the on-deck state unset, a height of 416 leptons would list but a few leptons more does not (revision 6: r < 208 or r' = 0, not r' < 208).
    go(&mut w, u, at(12, 4));
    for _ in 0..4 {
        step(&mut w);
    }
    assert!(height(&w, u) > 0);
    assert!(!j(&w, u).on_deck);
    assert_eq!(w.obj(u).unwrap().layer, Layer::Ground);
    assert!(!w.cells.list(c, Layer::Deck).contains(&u));
    assert!(
        !w.cells.list(c, Layer::Ground).contains(&u) && j(&w, u).listed.is_none(),
        "above the deck by a few leptons it is listed nowhere (S-0502 claim 4, revision 6)"
    );
    for _ in 0..60 {
        step(&mut w);
    }
    assert!(height(&w, u) >= 208 && j(&w, u).mark.is_none() && j(&w, u).listed.is_none());
}

// spec: S-0502 claim 15 and scenario J9 (the landing check fails on an occupied cell: the vehicle standing in the cell makes the entry outcome refuse, the reservation is released and the jumpjet re-routes to a nearby free cell on that tick, and lands there)
#[test]
fn claim_15_failed_landing_reroutes() {
    // The blocker may arrive at any of the last descending ticks: the reroute is on that very tick.
    for delay in [0, 1] {
        let mut w = game();
        let u = descender(&mut w, "JJ", Kind::Infantry, 4, 0, 14);
        for _ in 0..=delay {
            step(&mut w);
        }
        assert!(j(&w, u).reservation.is_some());
        let blocker = put(&mut w, "CART", Kind::Vehicle, 0, 4, 0);
        assert!(w.obj(blocker).is_some());
        step(&mut w);
        let s = j(&w, u);
        assert!(
            s.reservation.is_none(),
            "released on the failing tick (delay {delay})"
        );
        assert_eq!(
            s.phase,
            Phase::Ascending,
            "re-ordered to a nearby cell (delay {delay})"
        );
        let d = s.dest.unwrap();
        assert_ne!(d.cell(), at(4, 0), "not the occupied cell");
        for _ in 0..300 {
            step(&mut w);
            if phase(&w, u) == Phase::Grounded {
                break;
            }
        }
        assert_eq!(phase(&w, u), Phase::Grounded);
        assert_eq!(pos(&w, u).cell(), d.cell());
    }
}

// spec: S-0502 claim 15 (infantry: the standing position at the destination point must be free, or the jumpjet holds its reservation; a taken position fails the check)
#[test]
fn claim_15_standing_position() {
    let mut w = game();
    let u = descender(&mut w, "JJ", Kind::Infantry, 4, 0, 30);
    let other = put(&mut w, "WALKER", Kind::Infantry, 0, 4, 0);
    assert!(marked_by(&w, at(4, 0), other));
    step(&mut w);
    assert_eq!(
        phase(&w, u),
        Phase::Ascending,
        "the position is taken: it re-routes"
    );
    assert_ne!(j(&w, u).dest.unwrap().cell(), at(4, 0));
}

// spec: S-0502 claim 15 (the deploy direction: a simple-deployer vehicle not seized turns its heading towards the game-wide deploy direction, (8192 × b) mod 65536)
#[test]
fn claim_15_deploy_direction() {
    for (b, want) in [(0, 0u16), (1, 8192), (9, 8192), (-1, 57344)] {
        let mut r = ruleset();
        r.sim.general.economy.deploy_dir = Some(b);
        let mut w = testkit::game_with(&r, SIZE, 7);
        testkit::run(&mut w, 4);
        let u = descender(&mut w, "DEPN", Kind::Vehicle, 4, 0, 400);
        js(&mut w, u).heading = glasnost_sim::turning::Turning::at(16384);
        step(&mut w);
        assert_eq!(j(&w, u).heading.target, want, "DeployDir {b}");
    }
}

// spec: S-0502 claim 16 and scenario J10 (a move order: it releases its landing reservation; no point: no longer moving; a descending jumpjet goes back to ascending with target height H; an order into a building snaps to a nearby cell centre, a balloon-hover vehicle or an entering vehicle keeps the exact point)
#[test]
fn claim_16_orders() {
    let mut w = game();
    // Re-ascent and reservation release.
    let u = descender(&mut w, "JJ", Kind::Infantry, 4, 0, 200);
    step(&mut w);
    assert!(j(&w, u).reservation.is_some());
    go(&mut w, u, at(12, 0));
    let s = j(&w, u);
    assert!(s.reservation.is_none());
    assert_eq!((s.phase, s.ht), (Phase::Ascending, 500));
    assert!(s.moving);
    // The order-level destination cleared (S-0213 claim 8 and S-0502 claim 24): not the direct no-point instruction; the moving unit is rerouted by claim 17 to a replacement point.
    glasnost_sim::econ::assign_destination(&mut w, u, None);
    assert!(jump::moving(&w, u) && j(&w, u).dest.is_some());
    assert_eq!(j(&w, u).phase, Phase::Ascending);
    // A direct no-point instruction (claim 16, claim 24's safety departure): refused while moving now.
    let before = j(&w, u).clone();
    jump::no_point(&mut w, u);
    assert_eq!(&before, j(&w, u), "refused while ascending");
    // To an idle hovering unit it clears the point and the moving flag and changes nothing else.
    let hv = hoverer(&mut w, "JJ", Kind::Infantry, 6, -6);
    assert!(jump::moving(&w, hv));
    let (ph, ht) = (j(&w, hv).phase, j(&w, hv).ht);
    jump::no_point(&mut w, hv);
    assert!(!jump::moving(&w, hv) && j(&w, hv).dest.is_none());
    assert_eq!((j(&w, hv).phase, j(&w, hv).ht), (ph, ht));
    // An order into a building's cell snaps to a nearby cell's centre.
    let b = put(&mut w, "TALL", Kind::Building, 1, -6, 6);
    let _ = b;
    let v = put(&mut w, "JV", Kind::Vehicle, 0, -10, 6);
    go(&mut w, v, at(-6, 6));
    let d = j(&w, v).dest.unwrap();
    assert_ne!(d.cell(), at(-6, 6));
    assert_eq!((d.x % 256, d.y % 256), (128, 128), "the centre of a cell");
    // A balloon-hover vehicle keeps the exact point.
    let k = put(&mut w, "BALLOON", Kind::Vehicle, 0, -10, 8);
    let exact = Pos::new(
        Pos::centre(at(-6, 8), 0).x + 50,
        Pos::centre(at(-6, 8), 0).y - 30,
        0,
    );
    // The order-level destination names a cell; the exact point is the object's: give it an object destination.
    let t = put(&mut w, "CART", Kind::Vehicle, 1, -6, 8);
    w.obj_mut(t).unwrap().pos = exact;
    glasnost_sim::econ::assign_destination(&mut w, k, Some(Target::Object(t)));
    assert_eq!(j(&w, k).dest, Some(exact));
    // An unload in progress is replaced by guard at a cancelled landing (S-0517 claim 8).
    let e = descender(&mut w, "DEPL", Kind::Vehicle, 8, 8, 100);
    let tick = w.tick;
    w.obj_mut(e)
        .unwrap()
        .beh
        .force(Behaviour::Unload, tick, true);
    js(&mut w, e).landing_request = true;
    go(&mut w, e, at(14, 8));
    assert_eq!(w.obj(e).unwrap().beh.queued, Some(Behaviour::Guard));
    assert!(w.obj(e).unwrap().beh.allowance);
    assert!(!j(&w, e).landing_request, "the landing request ends");
}

// spec: S-0502 claim 17 and scenario J11 (a failed landing with no free cell nearby: the jumpjet is destroyed by its remaining health with no attacker and no kill credit; with quirks.jumpjet_no_cell_self_destruct off, in the air, it hovers again, revision 16's Edge cases)
#[test]
fn claim_17_no_cell_self_destruct() {
    for on in [true, false] {
        let mut w = game();
        quirk(&mut w, |q| q.jumpjet_no_cell_self_destruct = on);
        let u = descender(&mut w, "JJ", Kind::Infantry, 4, 0, 30);
        step(&mut w);
        // Every cell is occupied: no nearby cell qualifies.
        let cells: Vec<CellXY> = w.cells.grid.iter().map(|(c, _)| c).collect();
        for c in cells {
            w.cells.claim_vehicle(c, Layer::Ground, true);
        }
        let credit = |w: &World| {
            (
                w.houses[0].combat.kills,
                w.houses[1].combat.kills,
                w.houses[0].combat.score,
                w.houses[1].combat.score,
            )
        };
        let scores = credit(&w);
        jump::stop(&mut w, u);
        if on {
            let dead = w.obj(u).is_none_or(|o| o.health <= 0);
            assert!(dead, "destroyed by its remaining health");
            assert_eq!(scores, credit(&w), "no kill and no score is credited");
        } else {
            assert_eq!(phase(&w, u), Phase::Hovering);
            assert!(w.obj(u).unwrap().health > 0);
            // spec: S-0502 Edge cases (over its current cell: no movement point elsewhere, no movement commitment)
            assert!(!jump::moving(&w, u), "no movement commitment");
            let c = w.obj(u).unwrap().pos.cell();
            for _ in 0..10 {
                step(&mut w);
            }
            assert_eq!(phase(&w, u), Phase::Hovering);
            assert_eq!(w.obj(u).unwrap().pos.cell(), c, "over its current cell");
        }
    }
}

// spec: S-0502 Edge cases (revision 16, quirks.jumpjet_no_cell_self_destruct off) and claim 17: a grounded jumpjet with a movement commitment, stopped before its take-off with no cell nearby, clears its movement point and commitment and stays grounded: no hover slot, no take-off, its height 0; with the switch on it is destroyed
#[test]
fn claim_17_no_cell_grounded_stays_grounded() {
    for on in [true, false] {
        let mut w = game();
        quirk(&mut w, |q| q.jumpjet_no_cell_self_destruct = on);
        let u = put(&mut w, "JV", Kind::Vehicle, 0, 4, 0);
        go(&mut w, u, at(8, 0));
        assert!(jump::moving(&w, u), "committed before its take-off");
        assert_eq!(phase(&w, u), Phase::Grounded);
        let cells: Vec<CellXY> = w.cells.grid.iter().map(|(c, _)| c).collect();
        for c in cells {
            w.cells.claim_vehicle(c, Layer::Ground, true);
        }
        jump::stop(&mut w, u);
        if on {
            assert!(
                w.obj(u).is_none_or(|o| o.health <= 0),
                "switch on: destroyed"
            );
            continue;
        }
        assert!(w.obj(u).unwrap().health > 0);
        assert_eq!(phase(&w, u), Phase::Grounded, "switch off: stays grounded");
        assert!(!jump::moving(&w, u), "no movement commitment");
        assert_eq!(j(&w, u).dest, None, "no movement point");
        assert_eq!(j(&w, u).slot, None, "no hover slot");
        let p = pos(&w, u);
        for _ in 0..10 {
            step(&mut w);
        }
        assert_eq!(phase(&w, u), Phase::Grounded);
        assert_eq!(height(&w, u), 0);
        assert_eq!(pos(&w, u), p, "it stays where it is");
    }
}

/// Claims every cell for a vehicle, so that no nearby cell qualifies for
/// claim 17's search.
fn block_every_cell(w: &mut World) {
    let cells: Vec<CellXY> = w.cells.grid.iter().map(|(c, _)| c).collect();
    for c in cells {
        w.cells.claim_vehicle(c, Layer::Ground, true);
    }
}

// spec: S-0502 Edge cases (revision 17, quirks.jumpjet_no_cell_self_destruct off; scenario J27 c, the boundary controls) and claim 17, S-0607 claim 9 (revision 9; answers Q-1344): with no cell for the stop, the integer height above the surface decides: ascending at height 0 (just after its take-off update), hovering at height 0 and ascending at −1 lepton stay put and become grounded, with no movement point, no commitment and no hover slot; ascending at 1 lepton hovers over its current cell instead
#[test]
fn claim_17_no_cell_switch_off_height_decides() {
    for case in ["ascending 0", "hovering 0", "ascending -1", "ascending 1"] {
        let mut w = game();
        quirk(&mut w, |q| q.jumpjet_no_cell_self_destruct = false);
        let u = put(&mut w, "JV", Kind::Vehicle, 0, 4, 0);
        match case {
            "ascending 0" => {
                go(&mut w, u, at(8, 0));
                step(&mut w);
                assert_eq!(phase(&w, u), Phase::Ascending, "{case}: took off");
            }
            "hovering 0" => {
                jump::start_in_flight(&mut w, u, Phase::Hovering, 0);
                js(&mut w, u).dest = Some(Pos::centre(at(8, 0), 0));
                js(&mut w, u).moving = true;
            }
            "ascending -1" => jump::start_in_flight(&mut w, u, Phase::Ascending, -1),
            _ => jump::start_in_flight(&mut w, u, Phase::Ascending, 1),
        }
        let z = height(&w, u);
        let expected = match case {
            "ascending -1" => -1,
            "ascending 1" => 1,
            _ => 0,
        };
        assert_eq!(z, expected, "{case}: the height before the stop");
        assert!(jump::moving(&w, u), "{case}: committed");
        block_every_cell(&mut w);
        jump::stop(&mut w, u);
        assert!(w.obj(u).unwrap().health > 0, "{case}: not destroyed");
        assert!(!jump::moving(&w, u), "{case}: no movement commitment");
        assert_eq!(j(&w, u).dest, None, "{case}: no movement point");
        let c = pos(&w, u).cell();
        if z >= 1 {
            assert_eq!(phase(&w, u), Phase::Hovering, "{case}: hovers");
            assert_eq!(j(&w, u).slot, Some(c), "{case}: its cell's hover slot");
        } else {
            assert_eq!(phase(&w, u), Phase::Grounded, "{case}: becomes grounded");
            assert_eq!(j(&w, u).slot, None, "{case}: no hover slot");
        }
        let p = pos(&w, u);
        for _ in 0..10 {
            step(&mut w);
        }
        assert_eq!(pos(&w, u), p, "{case}: it stays put");
        assert_eq!(height(&w, u), z, "{case}: neither climbs nor descends");
    }
}

// spec: S-0502 Edge cases (revision 17, switch off: height 0 or less, it becomes grounded, also from ascending) and claim 5 with scenario J2 (a grounded jumpjet's take-off timing): a jumpjet stopped with no cell while ascending at height 0 keeps its ground listing and mark, and a later order takes it off exactly as a freshly placed grounded unit: the same height trace from the order on, no early rise
#[test]
fn claim_17_grounded_from_ascending_takes_off_as_a_fresh_unit() {
    let trace = |stopped: bool| {
        let mut w = game();
        quirk(&mut w, |q| q.jumpjet_no_cell_self_destruct = false);
        let u = put(&mut w, "JV", Kind::Vehicle, 0, 4, 0);
        if stopped {
            go(&mut w, u, at(8, 0));
            step(&mut w);
            assert_eq!(phase(&w, u), Phase::Ascending);
            block_every_cell(&mut w);
            jump::stop(&mut w, u);
            assert_eq!(phase(&w, u), Phase::Grounded);
            let c = pos(&w, u).cell();
            assert_eq!(j(&w, u).listed, Some((c, Layer::Ground)), "still listed");
            assert!(w.cells.list(c, Layer::Ground).contains(&u));
            assert!(j(&w, u).mark.is_some(), "still marked");
            let cells: Vec<CellXY> = w.cells.grid.iter().map(|(c, _)| c).collect();
            for c in cells {
                w.cells.claim_vehicle(c, Layer::Ground, false);
            }
        }
        go(&mut w, u, at(4, 8));
        let mut v = Vec::new();
        for _ in 0..12 {
            step(&mut w);
            v.push((height(&w, u), phase(&w, u)));
        }
        v
    };
    assert_eq!(trace(true), trace(false));
}

// spec: S-0502 claim 17 (stopping a jumpjet that is not moving does nothing)
#[test]
fn claim_17_stopping_an_idle_jumpjet_does_nothing() {
    let mut w = game();
    let u = put(&mut w, "JJ", Kind::Infantry, 0, 0, 0);
    let before = j(&w, u).clone();
    jump::stop(&mut w, u);
    assert_eq!(&before, j(&w, u));
    assert!(w.obj(u).unwrap().health > 0);
}

/// The step direction (0 to 7) from cell `a` to the neighbouring cell `b`.
fn direction(a: CellXY, b: CellXY) -> usize {
    jump::STEP_DIRECTIONS
        .iter()
        .position(|&(dx, dy)| (a.x + dx, a.y + dy) == (b.x, b.y))
        .expect("a neighbour")
}

// spec: S-0502 claim 21 and scenario J8 (b) (a held hover slot: one ranged draw over 0 to 7 from the scenario random stream, taken in the tick of the transition; the neighbouring cell in that step direction becomes the destination and the jumpjet cruises; nothing tests the neighbour first)
#[test]
fn claim_21_neighbour_step_on_arrival() {
    let mut w = game();
    let _holder = hoverer(&mut w, "JJ", Kind::Infantry, 4, 0);
    let u = put(&mut w, "JJ", Kind::Infantry, 0, 0, 0);
    let t = w.tick;
    w.obj_mut(u).unwrap().beh.force(Behaviour::Move, t, true);
    go(&mut w, u, at(4, 0));
    jump::start_in_flight(&mut w, u, Phase::Cruising, 500);
    let d = Pos::centre(at(4, 0), 0);
    js(&mut w, u).dest = Some(d);
    w.obj_mut(u).unwrap().pos = Pos::new(d.x - 15, d.y, 500);
    // A target keeps it hovering at arrival, so the slot conflict decides.
    let e = enemy(&mut w);
    w.obj_mut(u).unwrap().target = Some(Target::Object(e));
    let half = w.rules().quirks.ranged_draw_half_span;
    let mut r = w.rng.clone();
    let draw = r.ranged(0, 7, half);
    let _ = r.ranged(0, 3, half); // spec: S-0502 claims 21 and 23 (the infantry point adjustment follows the neighbour draw with its own draw over 0 to 3)
    jump::tick(&mut w, u);
    assert_eq!(phase(&w, u), Phase::Cruising);
    assert_eq!(
        w.rng.taken(),
        r.taken(),
        "the neighbour draw and the infantry adjustment's draw"
    );
    let Some(Target::Cell(n)) = w.obj(u).unwrap().dest else {
        panic!("the neighbour cell is the new destination")
    };
    assert_eq!(
        direction(at(4, 0), n),
        draw as usize,
        "the step direction of the draw"
    );
}

// spec: S-0502 claim 21 and claim 7 (the same one-draw rule at the end of an ascent whose cell's slot is held)
#[test]
fn claim_21_neighbour_step_at_the_end_of_an_ascent() {
    let mut w = game();
    let _holder = hoverer(&mut w, "JJ", Kind::Infantry, 0, 0);
    let u = put(&mut w, "JJ", Kind::Infantry, 0, 2, 0);
    let e = enemy(&mut w);
    let t = w.tick;
    w.obj_mut(u).unwrap().beh.force(Behaviour::Move, t, true);
    go(&mut w, u, at(2, 0));
    w.obj_mut(u).unwrap().target = Some(Target::Object(e));
    jump::start_in_flight(&mut w, u, Phase::Ascending, 499);
    js(&mut w, u).dest = Some(Pos::centre(at(2, 0), 0));
    js(&mut w, u).order = Some(Target::Cell(at(2, 0)));
    // spec: S-0502 claim 23 (r24, Q-3043: the own-cell assignment above changes nothing, so the order-level destination is set as an order given elsewhere leaves it)
    w.obj_mut(u).unwrap().dest = Some(Target::Cell(at(2, 0)));
    // Move the climber over the held slot's cell.
    let p = pos(&w, u);
    w.obj_mut(u).unwrap().pos =
        Pos::new(Pos::centre(at(0, 0), 0).x, Pos::centre(at(0, 0), 0).y, p.z);
    js(&mut w, u).ht = 500;
    let mut r = w.rng.clone();
    let half = w.rules().quirks.ranged_draw_half_span;
    let draw = r.ranged(0, 7, half);
    let _ = r.ranged(0, 3, half); // spec: S-0502 claims 21 and 23 (the infantry point adjustment follows the neighbour draw with its own draw over 0 to 3)
    jump::tick(&mut w, u);
    assert_eq!(phase(&w, u), Phase::Cruising);
    assert_eq!(
        w.rng.taken(),
        r.taken(),
        "the neighbour draw and the infantry adjustment's draw at the conflict"
    );
    let Some(Target::Cell(n)) = w.obj(u).unwrap().dest else {
        panic!("a destination cell")
    };
    assert_eq!(direction(at(0, 0), n), draw as usize);
}

// spec: S-0502 claim 32 (firing a disc-laser weapon does not lower a hovering jumpjet, change its target height or put it in the ground layer: it keeps hovering under claim 13)
#[test]
fn claim_32_disc_laser_attacks_do_not_force_a_descent() {
    let mut r = ruleset();
    let rifle = r.sim.weapons.get_mut("Rifle").unwrap();
    rifle.disk_laser = Some(true);
    rifle.laser_inner_color = Some([200, 100, 50]);
    rifle.laser_outer_color = Some([100, 50, 25]);
    rifle.laser_outer_spread = Some([10, 10, 10]);
    let mut w = testkit::game_with(&r, SIZE, 7);
    testkit::run(&mut w, 4);
    let u = hoverer(&mut w, "JJ", Kind::Infantry, 16, 18);
    let ht = j(&w, u).ht;
    let mut fired = false;
    for _ in 0..120 {
        step(&mut w);
        fired |= !w.discs.is_empty();
        assert_eq!(phase(&w, u), Phase::Hovering);
        assert_eq!(j(&w, u).ht, ht, "the target height is unchanged");
        assert!(
            !glasnost_sim::temporal::ground_layer(&w, u),
            "not in the ground render layer"
        );
    }
    assert!(fired, "the jumpjet fired its disc laser");
}
