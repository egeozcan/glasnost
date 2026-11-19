//! Claim tests of S-0208 revision 71 (Q-3264 answered), claim 18: an
//! object's point in a high-bridge cell raised by 414 leptons
//! (`quirks.walker_bridge_destination_deck` on), or tested on the arrival
//! layer with the switch off. Claim 26 (a substitute cell given by an
//! assignment) is held under Q-3303 and not tested here. On a flat
//! synthetic map with invented types; no game data. Ledger:
//! `docs/m8-claims/sim-issues.md`, *Batch S42*.

mod common;

use common::{FOOT, Scene, cell, unit_type};
use glasnost_map::CellXY;
use glasnost_sim::world::{HouseId, Layer, ObjectId, Pos, Target, World};
use glasnost_sim::{econ, tick};

fn scene() -> Scene {
    let mut s = Scene::new();
    s.infantry("TESTMAN", unit_type(FOOT, 4, 0));
    s
}

fn of(w: &World, h: u8, kind: glasnost_sim::rules::Kind) -> ObjectId {
    w.objects
        .values()
        .find(|o| o.owner == Some(HouseId(h)) && o.kind == kind)
        .map(|o| o.id)
        .unwrap()
}

fn dest(w: &World, id: ObjectId) -> Option<Target> {
    w.obj(id).unwrap().dest
}

/// Flat ground with a high-bridge deck over cells (3 … 5, 0); a walker F
/// of Alpha's and an enemy soldier I of Beta's in cell (4, 0), each on the
/// deck when its flag is set (at *H* + 416). F is given I as its
/// destination and then stands on its arrival tick: its own standing point
/// as walking target, no path steps.
fn deck_pair(quirk: bool, f_deck: bool, i_deck: bool) -> (World, ObjectId, ObjectId) {
    let mut s = scene();
    s.ruleset.globals.quirks.walker_bridge_destination_deck = quirk;
    let c = cell(4, 0);
    s.place_infantry(0, "TESTMAN", c, 1);
    s.place_infantry(1, "TESTMAN", c, 2);
    let mut w = s.build();
    for x in 3..=5 {
        let i = w.cells.idx(cell(x, 0)).expect("cell");
        w.cells.deck[i] = true;
        w.cells.deck_entry[i] = true;
        w.cells.bridge_axis[i] = glasnost_sim::cells::Axis::X;
    }
    w.regions.dirty = true;
    let f = of(&w, 0, glasnost_sim::rules::Kind::Infantry);
    let i = of(&w, 1, glasnost_sim::rules::Kind::Infantry);
    for (id, on) in [(f, f_deck), (i, i_deck)] {
        if on {
            w.cells.leave(c, Layer::Ground, id);
            w.cells.enter(c, Layer::Deck, id, false);
            let o = w.obj_mut(id).unwrap();
            o.layer = Layer::Deck;
            o.pos.z += 416; // spec: S-0208 claim 18 (an object on the deck has height H + 416)
        }
    }
    econ::assign_destination(&mut w, f, Some(Target::Object(i)));
    let o = w.obj_mut(f).unwrap();
    o.path.steps.clear();
    let p = o.pos;
    let ws = o.walk.as_mut().unwrap();
    ws.target = Some(p);
    ws.fraction = true;
    (w, f, i)
}

fn point(w: &World, f: ObjectId) -> Option<(ObjectId, Pos)> {
    w.obj(f).unwrap().walk.as_ref().unwrap().object_point
}

fn h(w: &World) -> i32 {
    let c: CellXY = cell(4, 0);
    w.surface_at(256 * c.x + 128, 256 * c.y + 128)
}

// spec: S-0208 claim 18 (r71: an object's point in a high-bridge cell has the object's own height plus 414 leptons, whatever the layers; an object on the ground under the deck gives H + 414, and the walker's walk ends on the deck above it)
#[test]
fn r71_an_object_under_the_deck_gives_h_plus_414_and_the_walk_ends_on_the_deck() {
    let (mut w, f, i) = deck_pair(true, true, false);
    let p = w.obj(i).unwrap().pos;
    assert_eq!(point(&w, f), Some((i, Pos::new(p.x, p.y, p.z + 414))));
    assert_eq!(p.z, h(&w), "void: I is on the ground");
    tick::step(&mut w);
    assert_eq!(dest(&w, f), None, "the walk ends on its arrival tick");
}

// spec: S-0208 claim 18 (r71: an object on the deck at H + 416 gives H + 830; a walker arriving on that deck in that cell is 414 leptons below it, so claim 16 does not end its walk and it keeps its destination; its next fresh search finds no path, S-0212 claim 12, and claim 25 takes the destination away within the close-enough distance)
#[test]
fn r71_an_object_on_the_deck_gives_h_plus_830_and_claim_25_clears_it() {
    let (mut w, f, i) = deck_pair(true, true, true);
    let p = w.obj(i).unwrap().pos;
    assert_eq!(p.z, h(&w) + 416, "void: I is on the deck");
    assert_eq!(point(&w, f), Some((i, Pos::new(p.x, p.y, h(&w) + 830))));
    tick::step(&mut w);
    assert_eq!(
        dest(&w, f),
        Some(Target::Object(i)),
        "kept on the arrival tick"
    );
    let n = common::run_until(&mut w, 100, |w| dest(w, f).is_none());
    assert!(n < 100, "claim 25 clears the destination");
    assert_eq!(w.obj(f).unwrap().pos.cell(), cell(4, 0));
}

// spec: S-0208 claim 18 and Edge cases (r71: with quirks.walker_bridge_destination_deck off, an object's point in a high-bridge cell takes, for the arrival test, the terrain height at its map position on the walker's arrival layer, 416 leptons up on the deck, as a cell destination does)
#[test]
fn r71_switch_off_an_objects_point_is_tested_on_the_arrival_layer() {
    let (mut w, f, i) = deck_pair(false, true, false);
    let p = w.obj(i).unwrap().pos;
    assert_eq!(
        point(&w, f),
        Some((i, p)),
        "no 414 leptons with the switch off"
    );
    tick::step(&mut w);
    assert_eq!(dest(&w, f), None, "the walk ends on the deck");
}
