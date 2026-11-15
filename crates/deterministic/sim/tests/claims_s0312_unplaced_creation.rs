//! S-0312 claim 13 with S-0611 claim 14: a crew that gets no position
//! never enters play, and nothing of its creation set does either. Its
//! draws and creation numbers stay taken (sim batch S13, #251).

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::Decimal;
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{Layer, World};

const SIZE: i32 = 24;

fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let push = |r: &mut Ruleset, reg: &str, n: &str| {
        r.registries.entry(reg.into()).or_default().push(n.into());
    };
    push(&mut r, "infantry", "SLV");
    push(&mut r, "infantry", "CREWM");
    push(&mut r, "vehicles", "VICTIM");
    let walker = r.sim.infantry["WALKER"].clone();
    let cart = r.sim.vehicles["CART"].clone();
    let s = &mut r.sim;
    s.infantry.insert("SLV".into(), walker.clone());
    // The crew type names a slave type: its creation set is itself and two slaves.
    let mut crew = walker;
    crew.econ.enslaves = Some("SLV".into());
    crew.econ.slaves_number = Some(2);
    s.infantry.insert("CREWM".into(), crew);
    let mut victim = cart;
    victim.crewed = Some(true);
    victim.passengers = None;
    s.vehicles.insert("VICTIM".into(), victim);
    // Synthetic: a certain crew escape.
    s.general.crew_escape = Some(Decimal("1".into()));
    s.general.technician = Some("CREWM".into());
    r
}

fn of_type(w: &World, name: &str) -> usize {
    let ti = w.rules().index_of(Kind::Infantry, name).unwrap();
    w.objects
        .values()
        .filter(|o| o.kind == Kind::Infantry && o.type_index == ti)
        .count()
}

// spec: S-0312 claim 13 (a crew that gets no position never enters play; its creation draw is taken); S-0611 claim 14 (the creation set: a refused set never enters play, its numbers stay reserved)
#[test]
fn a_refused_crew_takes_its_creation_set_with_it() {
    let r = rules();
    let mut w = testkit::game_with(&r, SIZE, 11);
    let id = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "VICTIM",
        0,
        testkit::mid(SIZE, 0, 0),
        0,
    );
    let c = w.obj(id).unwrap().pos.cell();
    // Another vehicle's vehicle-present claim on the cell: the position
    // table's first row refuses the crew.
    w.cells.claim_vehicle(c, Layer::Ground, true);
    let ids_before = w.next_id;
    damage::apply(
        &mut w,
        id,
        Hit {
            amount: 100_000,
            warhead: None,
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: true,
            no_escape: false,
        },
    );
    assert_eq!(
        w.next_id - ids_before,
        3,
        "crew and two slaves were created"
    );
    assert_eq!(of_type(&w, "CREWM"), 0, "the refused crew is gone");
    assert_eq!(of_type(&w, "SLV"), 0, "and so are its slaves");
}
