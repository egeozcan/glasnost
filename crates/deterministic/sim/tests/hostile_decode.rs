//! An order's bytes cannot nest without bound (#155). Before the fix the
//! route marker held a whole order, so one byte per level made the derived
//! decoder recurse until the stack overflowed, which aborts the process: a
//! failure here ends this test binary, so the tests live in a file of
//! their own. Synthetic bytes only.

use glasnost_sim::orders::Order;
use glasnost_sim::save::{FORMAT_VERSION, Replay};
use glasnost_sim::testkit::setup;
use glasnost_sim::world::{HouseId, ObjectId};

/// Levels of nesting: what a 64 KiB relay message holds.
const DEPTH: usize = 60_000;
/// A small stack for the decoding thread.
const STACK: usize = 256 * 1024;

/// The tag byte of the route marker, the order kind after the point
/// delete.
fn marker_tag() -> u8 {
    let delete = postcard::to_allocvec(&Order::RouteDelete { point: 0 }).unwrap();
    assert_eq!(delete, [delete[0], 0]);
    delete[0] + 1
}

/// The marker's tag `DEPTH` times, then a cheer order.
fn nested() -> Vec<u8> {
    let cheer = postcard::to_allocvec(&Order::Cheer).unwrap();
    let mut b = vec![marker_tag(); DEPTH];
    b.extend(cheer);
    b
}

/// Runs `f` on a thread with a small stack; a stack overflow aborts the
/// process.
fn on_small_stack(f: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .stack_size(STACK)
        .spawn(f)
        .unwrap()
        .join()
        .unwrap();
}

// nocite: our own input validation (#155): a hostile frame's order decodes, or fails to, without recursion
#[test]
fn deeply_nested_order_bytes_decode_without_overflowing_the_stack() {
    let bytes = nested();
    on_small_stack(move || {
        let r = postcard::from_bytes::<Order>(&bytes);
        // Whatever it decodes to, it is dropped here too.
        drop(r);
    });
}

/// The marker holds a behaviour order's fields now, so the bytes of every
/// order that input sends are what they were: the wire format keeps its
/// version. Each order kind's tag is its place in the list of kinds.
// nocite: our own input validation (#155): the encoding of input orders is unchanged
#[test]
fn input_orders_keep_their_bytes() {
    let bytes = |o: Order| postcard::to_allocvec(&o).unwrap();
    assert_eq!(bytes(Order::Stop { actor: ObjectId(7) }), [1, 7]);
    assert_eq!(bytes(Order::Cheer), [5]);
    assert_eq!(bytes(Order::Surrender), [17]);
    assert_eq!(
        bytes(Order::Detonate {
            actor: ObjectId(300)
        }),
        [19, 172, 2]
    );
    assert_eq!(bytes(Order::GameSpeed { speed: 4 }), [21, 4]);
    assert_eq!(bytes(Order::RouteCommit), [24]);
    assert_eq!(bytes(Order::RouteDelete { point: 3 }), [25, 3]);
    assert_eq!(marker_tag(), 26);
}

// nocite: our own input validation (#155): the same bytes inside a replay file
#[test]
fn a_replay_with_deeply_nested_order_bytes_reads_without_overflowing_the_stack() {
    let mut rep = Replay {
        version: FORMAT_VERSION,
        rules_hash: String::new(),
        map: "synthetic".into(),
        setup: setup(1),
        orders: Vec::new(),
        ticks: 0,
    };
    rep.record(0, HouseId(0), Order::Cheer);
    let mut bytes = rep.to_bytes();
    // The bytes end with the order's tag, then `ticks`.
    let cheer = postcard::to_allocvec(&Order::Cheer).unwrap();
    let tail = bytes.split_off(bytes.len() - 2);
    assert_eq!(tail, [cheer[0], 0]);
    bytes.extend(std::iter::repeat_n(marker_tag(), DEPTH));
    bytes.extend(tail);
    on_small_stack(move || {
        let r = Replay::from_bytes(&bytes);
        drop(r);
    });
}
