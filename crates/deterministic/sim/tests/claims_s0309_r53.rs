//! Claim tests of S-0309 revision 53 (claims 7, 22 and 27; Q-2112 and
//! Q-2114) and S-0210 revision 31 (claim 4, a crushed vehicle's animations;
//! Q-2114), on the testkit's synthetic combat ruleset (CART, CRUSHER,
//! BUGGY, WALKER: every name and number invented, no game data). The ledger
//! is `docs/m8-claims/intake52b-S-0309-S-0405-M-0005-M-0006-M-0016.md`.

use glasnost_rules::schema::Ruleset;
use glasnost_sim::combat::acquire::{self, SearchContext, SearchKind};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::crush;
use glasnost_sim::objects;
use glasnost_sim::path::{self, Search};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{Layer, ObjectId, Target, World};

const SIZE: i32 = 24;

fn game_on(r: &Ruleset, seed: u32) -> World {
    testkit::game_with(r, SIZE, seed)
}

fn game() -> World {
    game_on(&testkit::combat_ruleset(), 7)
}

fn put(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, house, testkit::mid(SIZE, dx, dy), 0)
}

fn hold(w: &mut World, holder: ObjectId, v: ObjectId, left: i32) {
    let tick = w.tick;
    let o = w.obj_mut(holder).unwrap();
    o.target = Some(Target::Object(v));
    o.targeting_timer = tick + left;
}

fn timer_left(w: &World, holder: ObjectId) -> i32 {
    w.obj(holder).unwrap().targeting_timer - w.tick
}

fn anims(w: &World) -> Vec<String> {
    w.effects
        .iter()
        .filter_map(|e| match e {
            Effect::Anim { name, .. } => Some(name.clone()),
            _ => None,
        })
        .collect()
}

/// The combat ruleset with BUGGY given an explosion list of four test
/// animations and a destruction-animation list of two others (none of them
/// a registered animation type, so they take no creation draw).
fn listed_rules(explodes: bool) -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let c = &mut r.sim.vehicles.get_mut("BUGGY").unwrap().combat;
    c.explosion = Some(["XA", "XB", "XC", "XD"].map(String::from).to_vec());
    c.destroy_anim = Some(["DA", "DB"].map(String::from).to_vec());
    c.explodes = Some(explodes);
    r
}

const EXPLOSION: [&str; 4] = ["XA", "XB", "XC", "XD"];
const DESTROY: [&str; 2] = ["DA", "DB"];

// ---------------------------------------------------------------------------
// S-0210 claim 4 (r31): a crushed vehicle's animations

// spec: S-0210 claim 4 (a crushed vehicle shows a killed vehicle's explosion and destruction animations: one list pick each, explosion first, S-0312 claim 9; these picks come before the timer draws of its holders, S-0309 claim 27)
// spec: S-0309 claim 27 (several draws at a crush: a removed vehicle's animation picks, then its holders' timer draws)
#[test]
fn a_crushed_vehicle_shows_its_two_animations_before_its_holders_draw() {
    for explodes in [false, true] {
        let mut w = game_on(&listed_rules(explodes), 7);
        let c = put(&mut w, Kind::Vehicle, "CRUSHER", 0, 0, 0);
        let v = put(&mut w, Kind::Vehicle, "BUGGY", 1, 0, 0);
        let h = put(&mut w, Kind::Vehicle, "CART", 0, -4, 0);
        hold(&mut w, h, v, 50);
        let half = w.rules().quirks.ranged_draw_half_span;
        let mut e = w.rng.clone();
        let k = e.list_pick(4) as usize;
        let j = e.list_pick(2) as usize;
        let timer = e.ranged(4, 8, half);
        w.effects.clear();
        crush::at_cell_centre(&mut w, c);
        assert!(w.obj(v).is_none(), "crushed");
        // spec: S-0312 claim 9 (an exploding vehicle with ammunition shows the last entry; the pick is still taken)
        let first = if explodes { EXPLOSION[3] } else { EXPLOSION[k] };
        assert_eq!(anims(&w), vec![first.to_string(), DESTROY[j].to_string()]);
        assert_eq!(
            timer_left(&w, h),
            timer,
            "the holder's draw comes after the two picks"
        );
        assert_eq!(w.obj(h).unwrap().target, None);
        assert_eq!(w.rng.taken(), e.taken(), "no other draw");
    }
}

// spec: S-0210 claim 4 (each pick only when its list is not empty: none, one or two list-pick draws)
#[test]
fn a_crushed_vehicle_with_empty_lists_takes_no_pick() {
    let mut w = game();
    let c = put(&mut w, Kind::Vehicle, "CRUSHER", 0, 0, 0);
    let v = put(&mut w, Kind::Vehicle, "BUGGY", 1, 0, 0);
    let before = w.rng.taken();
    w.effects.clear();
    crush::at_cell_centre(&mut w, c);
    assert!(w.obj(v).is_none());
    assert!(anims(&w).is_empty());
    assert_eq!(w.rng.taken(), before);
}

// spec: S-0210 claim 4 (a crushed infantry unit shows no death animation and takes no pick)
#[test]
fn a_crushed_infantry_unit_shows_no_death_animation() {
    let mut r = listed_rules(false);
    let c = &mut r.sim.infantry.get_mut("WALKER").unwrap().combat;
    c.explosion = Some(vec!["XA".into()]);
    c.destroy_anim = Some(vec!["DA".into()]);
    let mut w = game_on(&r, 7);
    let c = put(&mut w, Kind::Vehicle, "CRUSHER", 0, 0, 0);
    let v = put(&mut w, Kind::Infantry, "WALKER", 1, 0, 0);
    let before = w.rng.taken();
    w.effects.clear();
    crush::at_cell_centre(&mut w, c);
    assert!(w.obj(v).is_none(), "crushed");
    assert!(anims(&w).is_empty());
    assert_eq!(w.rng.taken(), before);
}

// ---------------------------------------------------------------------------
// S-0309 claim 27 (r53): the order of the timer draws

/// The world `build` sets up, on the first seed from 1 whose next two timer
/// draws after the setup differ, so that which draw went to which holder
/// can be told; with those two draws.
fn distinct_draws<T>(build: impl Fn(&mut World) -> T) -> (World, T, i32, i32) {
    for seed in 1..64 {
        let mut w = game_on(&testkit::combat_ruleset(), seed);
        let ids = build(&mut w);
        let half = w.rules().quirks.ranged_draw_half_span;
        let mut e = w.rng.clone();
        let a = e.ranged(4, 8, half);
        let b = e.ranged(4, 8, half);
        if a != b {
            return (w, ids, a, b);
        }
    }
    panic!("no seed with two different draws");
}

/// Takes `x` inside and puts it back where it stood, so that it enters
/// play again and moves to the end of the update order (S-0203 claim 3).
fn reenter(w: &mut World, x: ObjectId) {
    let o = w.obj(x).unwrap().clone();
    objects::take_inside(w, x);
    assert!(!w.obj(x).unwrap().on_map);
    let f = w.body_facing(x);
    objects::place(w, x, o.pos, f);
}

fn turn_of(w: &World, x: ObjectId) -> usize {
    w.update.order.iter().position(|&y| y == x).unwrap()
}

// spec: S-0309 Terms (creation order) and claim 27 (several holders of one object draw in creation order, the holder created first first, whatever their update order or distances; scenario A18 run f1)
#[test]
fn claim_27_holders_draw_in_creation_order() {
    let (mut w, (c, v, h1, h2), first, second) = distinct_draws(|w| {
        let c = put(w, Kind::Vehicle, "CRUSHER", 0, 0, 0);
        let v = put(w, Kind::Infantry, "WALKER", 1, 0, 0);
        // H1 created first and farther from the victim; it re-enters play
        // after H2, so that its turn comes after H2's.
        let h1 = put(w, Kind::Vehicle, "CART", 0, -5, 0);
        let h2 = put(w, Kind::Vehicle, "CART", 0, -2, 2);
        reenter(w, h1);
        hold(w, h1, v, 50);
        hold(w, h2, v, 50);
        (c, v, h1, h2)
    });
    assert!(turn_of(&w, h2) < turn_of(&w, h1), "H2's turn comes first");
    crush::at_cell_centre(&mut w, c);
    assert!(w.obj(v).is_none());
    assert_eq!(timer_left(&w, h1), first, "H1, created first, draws first");
    assert_eq!(timer_left(&w, h2), second);
}

// spec: S-0309 claim 27 (one crush removing several objects removes them one at a time in the cell layer's listing order, S-0209 claim 13; every draw of one removed object comes before those of the next; scenario A18 run f2)
#[test]
fn claim_27_a_crush_draws_per_removed_object() {
    let (mut w, (c, v1, v2, h1, h2), first, second) = distinct_draws(|w| {
        let c = put(w, Kind::Vehicle, "CRUSHER", 0, 0, 0);
        let v2 = put(w, Kind::Infantry, "WALKER", 1, 0, 0);
        let v1 = put(w, Kind::Infantry, "WALKER", 1, 0, 0);
        let h1 = put(w, Kind::Vehicle, "CART", 0, -5, 0);
        let h2 = put(w, Kind::Vehicle, "CART", 0, -2, 2);
        hold(w, h1, v2, 50);
        hold(w, h2, v1, 50);
        (c, v1, v2, h1, h2)
    });
    let l = w
        .cells
        .list(testkit::mid(SIZE, 0, 0), Layer::Ground)
        .to_vec();
    let at = |x| l.iter().position(|&y| y == x).unwrap();
    assert!(at(v1) < at(v2), "V1 entered last: listed first");
    crush::at_cell_centre(&mut w, c);
    assert!(w.obj(v1).is_none() && w.obj(v2).is_none());
    assert_eq!(
        timer_left(&w, h2),
        first,
        "V1's holder first, by removed object"
    );
    assert_eq!(timer_left(&w, h1), second);
}

// ---------------------------------------------------------------------------
// S-0309 claims 7 and 22 (r53): route failures

/// A CART of house 0 at the centre holding `t`, its destination `dest`
/// cells along the x axis.
fn approach(w: &mut World, t: ObjectId, dest: i32) -> ObjectId {
    let a = put(w, Kind::Vehicle, "CART", 0, 0, 0);
    let o = w.obj_mut(a).unwrap();
    o.target = Some(Target::Object(t));
    o.dest = Some(Target::Cell(testkit::mid(SIZE, dest, 0)));
    o.beh.queued = None;
    a
}

// spec: S-0309 claim 7 (a route search that fails more than 1 cell from the destination, S-0212 claim 23 row 2, also drops the actor's target; it queues the resting behaviour) and claim 22 (no limited-scan mark at that give-up)
#[test]
fn claim_7_a_give_up_drops_the_target_and_sets_no_mark() {
    let mut w = game();
    let t = put(&mut w, Kind::Vehicle, "BUGGY", 1, 9, 0);
    let a = approach(&mut w, t, 6);
    w.obj_mut(a).unwrap().beh.current = Some(Behaviour::Hunt);
    path::failed(&mut w, a, Search::SearchFailed, false);
    let o = w.obj(a).unwrap();
    assert_eq!(o.target, None, "dropped at the give-up");
    assert!(
        matches!(o.beh.queued, Some(Behaviour::Guard | Behaviour::AreaGuard)),
        "the resting behaviour is queued"
    );
    assert!(!o.combat.limited_scan, "no mark at a give-up");
}

// spec: S-0309 claim 22 (a driving vehicle's route failure within 1 cell of its destination, S-0212 claim 23 row 3, with a target not within reach, claim 23: the target is dropped and the limited-scan mark set; a gate failure is another failure that does not give the move up)
#[test]
fn claim_22_a_failure_short_of_a_give_up_sets_the_mark() {
    for res in [Search::SearchFailed, Search::GateFailed] {
        let mut w = game();
        let t = put(&mut w, Kind::Vehicle, "BUGGY", 1, 9, 0);
        let a = approach(&mut w, t, 1);
        path::failed(&mut w, a, res, false);
        let o = w.obj(a).unwrap();
        assert_eq!(o.target, None, "{res:?}: dropped");
        assert!(o.combat.limited_scan, "{res:?}: marked");
        assert_eq!(o.beh.queued, None, "{res:?}: not given up");
    }
}

// spec: S-0309 claim 22 (a target within reach, claim 23, is kept and no mark is set)
#[test]
fn claim_22_a_target_within_reach_is_kept() {
    let mut w = game();
    let t = put(&mut w, Kind::Vehicle, "BUGGY", 1, 3, 0);
    let a = approach(&mut w, t, 1);
    path::failed(&mut w, a, Search::SearchFailed, false);
    let o = w.obj(a).unwrap();
    assert_eq!(o.target, Some(Target::Object(t)));
    assert!(!o.combat.limited_scan);
}

// spec: S-0309 claim 22 (vehicles only: a walking infantry unit's failure sets no mark)
#[test]
fn claim_22_no_mark_for_infantry() {
    let mut w = game();
    let t = put(&mut w, Kind::Vehicle, "BUGGY", 1, 9, 0);
    let a = put(&mut w, Kind::Infantry, "WALKER", 0, 0, 0);
    {
        let o = w.obj_mut(a).unwrap();
        o.target = Some(Target::Object(t));
        o.dest = Some(Target::Cell(testkit::mid(SIZE, 1, 0)));
    }
    path::failed(&mut w, a, Search::SearchFailed, false);
    let o = w.obj(a).unwrap();
    assert_eq!(o.target, Some(Target::Object(t)));
    assert!(!o.combat.limited_scan);
}

// spec: S-0309 claim 22 (with the mark a map-wide search is a range search; a search that finds a target does not clear the mark, nor does that target's removal; only a search that finds nothing clears it; scenario A11 runs c1 and c2)
#[test]
fn claim_22_the_mark_is_kept_until_an_empty_search() {
    let mut w = game();
    let far = put(&mut w, Kind::Vehicle, "BUGGY", 1, 9, 0);
    let a = approach(&mut w, far, 1);
    path::failed(&mut w, a, Search::SearchFailed, false);
    assert!(w.obj(a).unwrap().combat.limited_scan);
    let near = put(&mut w, Kind::Infantry, "WALKER", 1, 2, 1);
    let hunt = |w: &mut World| acquire::search(w, a, SearchKind::MapWide, SearchContext::Hunt);
    assert_eq!(
        hunt(&mut w),
        Some(near),
        "a range search finds the near one"
    );
    assert!(w.obj(a).unwrap().combat.limited_scan, "found: kept");
    objects::remove(&mut w, near);
    assert!(w.obj(a).unwrap().combat.limited_scan, "removal: kept");
    w.obj_mut(a).unwrap().target = None;
    assert_eq!(hunt(&mut w), None, "the far one is beyond a range search");
    assert!(!w.obj(a).unwrap().combat.limited_scan, "empty: cleared");
    assert_eq!(hunt(&mut w), Some(far), "map-wide again");
}
