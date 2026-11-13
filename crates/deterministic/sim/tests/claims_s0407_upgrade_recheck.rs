//! S-0407 claim 12 events (d) "a building gains an upgrade" and (i) "a sale
//! ends without removing the building", and `quirks.sale_start_no_recheck`
//! (#202). On synthetic data: invented types and costs, no game data.

mod econ_common;

use econ_common as e;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{LandRow, LandType};
use glasnost_sim::econ::tech::{self, Answer};
use glasnost_sim::orders::Order;
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, Pos};

const H: HouseId = HouseId(0);

/// The economy ruleset with HUT taking one upgrade, an upgrade type UPG
/// for it, and an infantry type UPX whose prerequisite is UPG.
fn rules() -> Ruleset {
    let mut r = e::ruleset();
    r.sim.land_types.insert(
        LandType::Clear,
        LandRow {
            buildable: Some(true),
            ..Default::default()
        },
    );
    {
        let hut = r.sim.buildings.get_mut("HUT").unwrap();
        hut.econ.upgrades = Some(1);
        hut.econ.buildup = Some(true);
        hut.econ.buildup_frames = Some(10);
    }
    let mut up = e::plain(50);
    up.econ.powers_up_building = Some("HUT".into());
    r.registries.get_mut("buildings").unwrap().push("UPG".into());
    r.sim.buildings.insert("UPG".into(), up);
    let mut x = e::foot(100);
    x.econ.prerequisite = Some(vec!["UPG".into()]);
    r.registries.get_mut("infantry").unwrap().push("UPX".into());
    r.sim.infantry.insert("UPX".into(), x);
    r
}

// spec: S-0407 claim 12 (d) and (i) and claim 8 (c) (gaining an upgrade and a sale that ends without removing the building are recheck events: the offer follows in the next house update)
#[test]
fn claim_12_upgrade_placed_and_sold_recheck_the_offer() {
    let r = rules();
    let mut w = e::game(&r);
    e::building(&mut w, "CONYARD", 0, -8, 0);
    e::building(&mut w, "BARRACKS", 0, -8, 6);
    // The HUT is the most recently acquired building (claim 8 (c)).
    let hut = e::building(&mut w, "HUT", 0, 0, 0);
    e::ready(&mut w);
    e::set_credits(&mut w, 0, 1_000_000);
    let at = testkit::mid(e::SIZE, 0, 0);
    glasnost_sim::econ::shroud::area_reveal(&mut w, H, Pos::centre(at, 0), 12);
    for _ in 0..3 {
        e::step(&mut w);
    }
    let upg = e::t(&w, Kind::Building, "UPG");
    let upx = e::t(&w, Kind::Infantry, "UPX");
    assert!(tech::offered(&w, H, upg), "the upgrade is offered");
    assert!(!tech::offered(&w, H, upx));
    assert_eq!(tech::answer(&w, H, upx), Answer::Refused);

    assert!(e::issue(&mut w, 0, Order::Build { what: upg }));
    e::run_to_done(&mut w, 0, upg, 2000).expect("done");
    for _ in 0..5 {
        e::step(&mut w);
    }
    assert!(e::issue(
        &mut w,
        0,
        Order::Place {
            what: upg,
            cell: Some(at),
        }
    ));
    e::step(&mut w);
    assert_eq!(w.obj(hut).unwrap().econ.building.upgrade_count, 1, "placed");
    // spec: S-0407 claim 12 (a placement: the recheck in the next tick's house update, scenario T14 run 4)
    e::step(&mut w);
    assert_eq!(tech::answer(&w, H, upx), Answer::Allowed, "claim 8 (c) is met");
    assert!(tech::offered(&w, H, upx), "offered after the upgrade is gained");

    assert!(e::issue(&mut w, 0, Order::Sell { actor: hut }));
    for _ in 0..50 {
        e::step(&mut w);
    }
    let o = w.obj(hut).expect("the building stays");
    assert_eq!(o.econ.building.upgrade_count, 0, "the upgrade is sold");
    assert!(o.econ.sale.is_none(), "the sale has ended");
    assert_eq!(tech::answer(&w, H, upx), Answer::Refused);
    assert!(!tech::offered(&w, H, upx), "not offered after the sale ended");
}

/// Whether a recheck is due for house 0 right after the tick that applies
/// a sell order for a plain building.
fn due_after_sale_start(no_recheck: bool) -> bool {
    let mut r = rules();
    r.globals.quirks.sale_start_no_recheck = no_recheck;
    let mut w = e::game(&r);
    e::building(&mut w, "CONYARD", 0, -8, 0);
    let hut = e::building(&mut w, "HUT", 0, 0, 0);
    e::ready(&mut w);
    for _ in 0..3 {
        e::step(&mut w);
    }
    assert!(!w.houses[0].econ.tech.recheck_due, "settled");
    assert!(e::issue(&mut w, 0, Order::Sell { actor: hut }));
    e::step(&mut w);
    assert!(w.obj(hut).unwrap().econ.sale.is_some(), "the sale started");
    w.houses[0].econ.tech.recheck_due
}

// spec: S-0407 Edge cases (quirks.sale_start_no_recheck: on, the original, the start of a sale is no event of claim 12; off, it is one for the building's owner)
#[test]
fn quirk_sale_start_no_recheck() {
    assert!(!due_after_sale_start(true), "on: no recheck at the sale start");
    assert!(due_after_sale_start(false), "off: a recheck at the sale start");
}
