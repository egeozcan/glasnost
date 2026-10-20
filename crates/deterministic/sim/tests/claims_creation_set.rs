//! Claim tests of the creation set (S-0611 claim 14 and F5, S-0506 claim 1,
//! S-0402 claim 19) on invented types of the testkit's combat ruleset:
//! every name and number here is made up; no game data.
//!
//! Added types:
//! - aircraft DRN (plain), DRS (names slave type SLV, count 1), LOOP
//!   (spawns LOOP, count 2: a cycle);
//! - infantry SLV (plain), SELF (names slave type SELF, count 2: a cycle);
//! - vehicles SPW (spawns DRN ×2, slaves SLV ×3), NST (spawns DRS ×2,
//!   slaves SLV ×1).

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::UnitType;
use glasnost_sim::objects;
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, World};

const ALPHA: HouseId = HouseId(0);

fn aircraft() -> UnitType {
    UnitType {
        strength: Some(50),
        speed: Some(10),
        ..Default::default()
    }
}

fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let push = |r: &mut Ruleset, reg: &str, n: &str| {
        r.registries.entry(reg.into()).or_default().push(n.into());
    };
    for n in ["DRN", "DRS", "LOOP"] {
        push(&mut r, "aircraft", n);
    }
    for n in ["SLV", "SELF"] {
        push(&mut r, "infantry", n);
    }
    for n in ["SPW", "NST"] {
        push(&mut r, "vehicles", n);
    }
    let walker = r.sim.infantry["WALKER"].clone();
    let cart = r.sim.vehicles["CART"].clone();
    let s = &mut r.sim;
    s.aircraft.insert("DRN".into(), aircraft());
    let mut drs = aircraft();
    drs.econ.enslaves = Some("SLV".into());
    drs.econ.slaves_number = Some(1);
    s.aircraft.insert("DRS".into(), drs);
    let mut lp = aircraft();
    lp.combat.spawns = Some("LOOP".into());
    lp.combat.spawns_number = Some(2);
    s.aircraft.insert("LOOP".into(), lp);
    s.infantry.insert("SLV".into(), walker.clone());
    let mut slf = walker;
    slf.econ.enslaves = Some("SELF".into());
    slf.econ.slaves_number = Some(2);
    s.infantry.insert("SELF".into(), slf);
    let mut spw = cart.clone();
    spw.combat.spawns = Some("DRN".into());
    spw.combat.spawns_number = Some(2);
    spw.econ.enslaves = Some("SLV".into());
    spw.econ.slaves_number = Some(3);
    s.vehicles.insert("SPW".into(), spw);
    let mut nst = cart;
    nst.combat.spawns = Some("DRS".into());
    nst.combat.spawns_number = Some(2);
    nst.econ.enslaves = Some("SLV".into());
    nst.econ.slaves_number = Some(1);
    s.vehicles.insert("NST".into(), nst);
    r
}

fn game(r: &Ruleset) -> World {
    testkit::game_with(r, 30, 7)
}

fn create(w: &mut World, kind: Kind, name: &str) -> ObjectId {
    let ti = w.rules().index_of(kind, name).unwrap();
    objects::create(w, kind, ti, Some(ALPHA))
}

fn type_name(w: &World, id: ObjectId) -> (Kind, u32) {
    let o = w.obj(id).unwrap();
    (o.kind, o.type_index)
}

fn ty(w: &World, kind: Kind, name: &str) -> (Kind, u32) {
    (kind, w.rules().index_of(kind, name).unwrap())
}

/// Creates `name` and returns the new objects in creation-number order,
/// checking that each took one raw draw followed by its number: the report
/// value of the i-th new object is the i-th raw value drawn (S-0202
/// claim 10).
fn create_set(w: &mut World, kind: Kind, name: &str) -> Vec<ObjectId> {
    let mut rng = w.rng.clone();
    let first = w.next_id;
    let taken = w.rng.taken();
    let root = create(w, kind, name);
    let made: Vec<ObjectId> = w.objects.range(root..).map(|(&k, _)| k).collect();
    assert_eq!(made[0], root);
    assert_eq!(w.next_id - first, made.len() as u32, "one number each");
    assert_eq!(
        w.rng.taken() - taken,
        made.len() as u64,
        "one raw draw each"
    );
    for &id in &made {
        let x = rng.raw();
        assert_eq!(
            w.obj(id).unwrap().report_value,
            (x & 0xFFFF) as u16,
            "each member's draw comes before its number, in creation order"
        );
    }
    made
}

// spec: S-0506 claim 1 (each initial slot object takes one raw creation draw and one creation number after the spawner's, in increasing slot index; all of them precede the spawner's initial slaves)
// spec: S-0611 claim 14 and F5 (C(o) = ⟨raw(o), num(o)⟩ · C(s₁) · … · C(s_N) · C(v₁) · … · C(v_M); every member owned by the same house)
#[test]
fn s0506_claim_1_slot_objects_before_slaves() {
    let r = rules();
    let mut w = game(&r);
    let made = create_set(&mut w, Kind::Vehicle, "SPW");
    assert_eq!(made.len(), 1 + 2 + 3);
    let c = w.obj(made[0]).unwrap().spawn.clone().unwrap();
    let slots: Vec<ObjectId> = c.slots.iter().map(|s| s.object.unwrap()).collect();
    assert_eq!(slots, made[1..3], "slot objects in increasing slot index");
    let g = glasnost_sim::econ::slaves::group_of(&w, made[0]).unwrap();
    let slaves: Vec<ObjectId> = g.members.iter().map(|m| m.id.unwrap()).collect();
    assert_eq!(
        slaves,
        made[3..6],
        "slaves after every slot object, in place order"
    );
    for &id in &made {
        assert_eq!(w.obj(id).unwrap().owner, Some(ALPHA));
    }
}

// spec: S-0402 claim 19 (the initial slaves' draws and numbers follow the master's and those of its initial spawn-slot objects; nested creation sets complete before the next slot or place)
// spec: S-0611 F5 (nested: each slot object's slave directly follows that slot object)
#[test]
fn s0402_claim_19_slaves_after_nested_slot_sets() {
    let r = rules();
    let mut w = game(&r);
    let made = create_set(&mut w, Kind::Vehicle, "NST");
    let drs = ty(&w, Kind::Aircraft, "DRS");
    let slv = ty(&w, Kind::Infantry, "SLV");
    let kinds: Vec<(Kind, u32)> = made.iter().map(|&id| type_name(&w, id)).collect();
    assert_eq!(
        kinds[1..],
        [drs, slv, drs, slv, slv],
        "slot 0, its slave, slot 1, its slave, then the master's slave"
    );
    let slaves = glasnost_sim::econ::slaves::slaves_of;
    assert_eq!(slaves(&w, made[1]), vec![made[2]]);
    assert_eq!(slaves(&w, made[3]), vec![made[4]]);
    assert_eq!(slaves(&w, made[0]), vec![made[5]]);
    for &s in &[made[2], made[4], made[5]] {
        let o = w.obj(s).unwrap();
        // spec: S-0402 claim 19 (inside, off the map, owned by the master's house)
        assert!(!o.on_map);
        assert_eq!(o.owner, Some(ALPHA));
    }
}

// spec: S-0402 claim 19 and S-0611 claim 14 (a created infantry object whose type names a slave type gets its group: the made infantry type's slaves)
#[test]
fn s0402_claim_19_infantry_master_gets_its_group() {
    let mut r = rules();
    r.sim.infantry.get_mut("SELF").unwrap().econ.enslaves = Some("SLV".into());
    let mut w = game(&r);
    let made = create_set(&mut w, Kind::Infantry, "SELF");
    assert_eq!(made.len(), 3);
    assert_eq!(
        glasnost_sim::econ::slaves::slaves_of(&w, made[0]),
        made[1..]
    );
}

// spec: S-0611 F5 (a chain of spawned and slave types leading back to a type already in it: the importer warns, and the type closing the cycle gets no initial fill inside that chain; Q-1530 interim: nor a controller or group)
#[test]
fn s0611_f5_cycles_end() {
    let r = rules();
    let mut w = game(&r);
    let warnings = w.rules().missiles.warnings.clone();
    assert_eq!(warnings.len(), 2, "{warnings:?}");
    let made = create_set(&mut w, Kind::Aircraft, "LOOP");
    assert_eq!(made.len(), 3, "the spawner and its two slot objects");
    assert!(w.obj(made[0]).unwrap().spawn.is_some());
    for &x in &made[1..] {
        assert!(w.obj(x).unwrap().spawn.is_none(), "no initial fill");
    }
    let made = create_set(&mut w, Kind::Infantry, "SELF");
    assert_eq!(made.len(), 3, "the master and its two slaves");
    for &x in &made[1..] {
        assert!(glasnost_sim::econ::slaves::group_of(&w, x).is_none());
    }
    // A type outside any cycle draws no warning.
    let mut r = rules();
    r.sim.aircraft.get_mut("LOOP").unwrap().combat.spawns = Some("DRN".into());
    r.sim.infantry.get_mut("SELF").unwrap().econ.enslaves = Some("SLV".into());
    let w = game(&r);
    assert!(w.rules().missiles.warnings.is_empty());
}
