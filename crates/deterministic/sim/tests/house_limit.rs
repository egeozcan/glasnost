//! A setup with more houses than a house id can name is refused (#334): no
//! house past the 256th gets a wrapped id. Synthetic data only (the
//! testkit's map, rules and vehicle type CART).

use glasnost_map::Unit;
use glasnost_map::objects::Placed;
use glasnost_sim::load::{new_game, sandbox_setup};
use glasnost_sim::testkit;
use glasnost_sim::world::HouseId;

fn unit(owner: &str, dx: i32, dy: i32, key: usize) -> Unit {
    Unit {
        placed: Placed {
            key: key.to_string().into_bytes(),
            owner: owner.as_bytes().to_vec(),
            type_name: b"CART".to_vec(),
            health: 256,
            cell: testkit::cell(dx, dy),
            facing: 0,
            raw: Vec::new(),
        },
    }
}

/// A map whose object lists name `n` owners: those of the testkit's map,
/// then H2, H3 and so on.
fn owners(n: usize) -> glasnost_map::Map {
    let mut m = testkit::map();
    for i in 2..n {
        m.objects.units.push(unit(&format!("H{i}"), 12, 12, 1000 + i));
    }
    m
}

// design: #334 (a setup with more houses than a house id can name is refused)
#[test]
fn issue_334_a_setup_of_257_houses_is_refused() {
    let rules = testkit::rules();
    let map = owners(257);
    let (setup, _) = sandbox_setup(&map, &rules, 1);
    assert_eq!(setup.houses.len(), 257);
    match new_game(rules, &map, testkit::catalogue(), &setup) {
        Err(e) => assert!(
            e.to_string().contains("257 houses"),
            "refused for another reason: {e}"
        ),
        Ok(w) => panic!(
            "accepted: house 256 ({}) has allies {:?}",
            w.houses[256].name, w.houses[256].allies
        ),
    }
}

// design: #334 (256 houses, the most a house id can name, are accepted, each allied with itself, S-0209 Terms)
#[test]
fn issue_334_a_setup_of_256_houses_is_accepted() {
    let rules = testkit::rules();
    let map = owners(256);
    let (setup, _) = sandbox_setup(&map, &rules, 1);
    assert_eq!(setup.houses.len(), 256);
    let w = new_game(rules, &map, testkit::catalogue(), &setup).expect("accepted");
    assert_eq!(w.houses.len(), 256);
    assert!(w.houses[255].allies.contains(&HouseId(255)));
}
