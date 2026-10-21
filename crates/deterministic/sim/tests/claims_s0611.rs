//! Claim tests of S-0611 (mutation) on an empty synthetic map with invented
//! types: every name and number here is made up; no game data. Houses Alpha
//! (0) and Beta (1) of the testkit's sandbox game, not allied, on the
//! superweapon ruleset of `sw_common`. The ledger is
//! `docs/m7-claims/S-0611.md`.
//!
//! Added on top of `sw_common`:
//! - superweapon GENE (the genetic-conversion kind word), its building
//!   GENEB;
//! - warheads Mutate (infantry-death kind 9, the cell warhead), MutateX
//!   (kind 9, cell spread 1, the explosion warhead), Plain (kind 0);
//! - infantry GROWN (strength 300, the made type), BIG (strength 30 000),
//!   HUSK (not human), JJC (crashable, jumpjet movement without the
//!   jumpjet type flag);
//! - animations MUTA (make-infantry index 0, 4 image frames, no rate field:
//!   one update per frame), MUTB (index 1, out of the one-entry list),
//!   IONB (the launch animation), PRE (makes nothing, follow-up MUTA);
//! - the general keys naming them, and the activation sound `Gene`.

mod sw_common;

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    AnimationType, BuildingType, Decimal, LandRow, LandType, Locomotor, WarheadType,
};
use glasnost_rules::sim_econ::SuperweaponType;
use glasnost_sim::cells::{Mark, Marker, position};
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::mutation::{self, MakerEvent};
use glasnost_sim::rules::Kind;
use glasnost_sim::superweapons::SwEvent;
use glasnost_sim::world::{HouseId, Layer, ObjectId, Pos, World};
use glasnost_sim::{objects, save, testkit};
use sw_common::{ALPHA, BETA, at, fire_now, ready_tick};

/// GENE's list position (after `sw_common`'s twelve).
const GENE: u32 = 12;
/// The made type's strength.
const GROWN_HP: i32 = 300;

fn d(s: &str) -> Decimal {
    Decimal(s.into())
}

fn reg(r: &mut Ruleset, registry: &str, name: &str) {
    let l = r.registries.entry(registry.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

fn walker_like(r: &Ruleset, strength: i32) -> glasnost_rules::sim::UnitType {
    let mut t = r.sim.infantry["WALKER"].clone();
    t.strength = Some(strength);
    t
}

/// The mutation ruleset, in explosion mode when `explosion`.
fn rules(explosion: bool) -> Ruleset {
    let mut r = sw_common::ruleset();
    // Unarmed infantry, so that no unit fires on its own; a maximum damage
    // above every amount here (S-0302 claim 6).
    r.sim.infantry.get_mut("WALKER").unwrap().primary = None;
    r.sim.general.max_damage = Some(1_000_000);
    r.sim.superweapons.insert(
        "GENE".into(),
        SuperweaponType {
            kind: Some("GeneticConverter".into()),
            action: Some("GeneticConverter".into()),
            recharge_time: Some(d("0.1")),
            ..Default::default()
        },
    );
    reg(&mut r, "superweapons", "GENE");
    let mut b = BuildingType {
        strength: Some(500),
        foundation: Some([1, 1]),
        ..Default::default()
    };
    b.econ.super_weapon = Some("GENE".into());
    r.sim.buildings.insert("GENEB".into(), b);
    reg(&mut r, "buildings", "GENEB");
    let verses = || Some(vec![d("1"); 11]);
    for (n, kind, spread) in [
        ("Mutate", 9, None),
        ("MutateX", 9, Some(d("1"))),
        ("Plain", 0, None),
    ] {
        r.sim.warheads.insert(
            n.into(),
            WarheadType {
                inf_death: Some(kind),
                cell_spread: spread,
                verses: verses(),
                ..Default::default()
            },
        );
    }
    // JJC: a crashable human type with jumpjet movement and no jumpjet type flag (S-0611 claim 7, MU13's fixture).
    let mut jjc = walker_like(&r, 100);
    jjc.locomotor = Some(Locomotor::Jumpjet);
    jjc.jumpjet = Some(false);
    jjc.jumpjet_speed = Some(14);
    jjc.jumpjet_climb = Some(d("5.0"));
    jjc.jumpjet_crash = Some(d("6.0"));
    jjc.jumpjet_height = Some(500);
    jjc.jumpjet_accel = Some(d("2.0"));
    jjc.crashable = Some(true);
    r.sim.infantry.insert("JJC".into(), jjc);
    reg(&mut r, "infantry", "JJC");
    for (n, hp) in [("GROWN", GROWN_HP), ("BIG", 30_000), ("HUSK", 100)] {
        let mut t = walker_like(&r, hp);
        if n == "HUSK" {
            t.not_human = Some(true);
        }
        r.sim.infantry.insert(n.into(), t);
        reg(&mut r, "infantry", n);
    }
    let maker = |i: i32| AnimationType {
        make_infantry: Some(i),
        frame_count: Some(4),
        ..Default::default()
    };
    r.sim.animations.insert("MUTA".into(), maker(0));
    r.sim.animations.insert("MUTB".into(), maker(1));
    r.sim.animations.insert(
        "PRE".into(),
        AnimationType {
            frame_count: Some(2),
            next: Some("MUTA".into()),
            ..Default::default()
        },
    );
    r.sim
        .animations
        .insert("IONB".into(), AnimationType::default());
    for n in ["MUTA", "MUTB", "PRE", "IONB"] {
        reg(&mut r, "animations", n);
    }
    let g = &mut r.sim.general.superweapons;
    g.mutate_explosion = Some(explosion);
    g.mutate_explosion_warhead = Some("MutateX".into());
    g.mutate_warhead = Some("Mutate".into());
    g.infantry_mutate = Some("MUTA".into());
    g.anim_to_infantry = Some(vec!["GROWN".into()]);
    g.ion_blast = Some("IONB".into());
    g.genetic_mutator_activate_sound = Some("Gene".into());
    r
}

/// One tick, the effects of the earlier ticks cleared first.
fn step(w: &mut World) {
    w.effects.clear();
    sw_common::step(w);
}

/// Whether `id` is still in play as a killed unit (the die-2 request of a
/// refused conversion: S-0107 claim 17; this ruleset's infantry has no
/// sequence art, so the sequence itself is `claims_s0312`'s).
fn dying(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| o.combat.killed)
}

fn warhead(w: &World, name: &str) -> u32 {
    w.rules()
        .combat
        .warheads
        .iter()
        .position(|h| h.name == name)
        .unwrap() as u32
}

fn game(r: &Ruleset) -> World {
    sw_common::game(r)
}

/// Infantry `name` of house `h` standing at position `p` of cell `c`.
fn put(w: &mut World, name: &str, h: HouseId, c: CellXY, p: u8) -> ObjectId {
    let ti = w.rules().index_of(Kind::Infantry, name).unwrap();
    let id = objects::create(w, Kind::Infantry, ti, Some(h));
    let (u, v) = glasnost_sim::walk::standing_point(p);
    let z = w.surface_at(256 * c.x + u, 256 * c.y + v);
    objects::place(w, id, Pos::new(256 * c.x + u, 256 * c.y + v, z), 0);
    id
}

/// A lethal mutating hit (the cell warhead Mutate) on `id`.
fn mutate(w: &mut World, id: ObjectId, attacker: Option<ObjectId>, house: Option<HouseId>) {
    let h = warhead(w, "Mutate");
    damage::apply(
        w,
        id,
        Hit {
            amount: 100_000,
            warhead: Some(h),
            distance: 0,
            attacker,
            house,
            ignore_defences: true,
            no_escape: false,
        },
    );
}

fn makers(w: &World) -> Vec<ObjectId> {
    w.mutation.makers.keys().copied().collect()
}

fn maker_events(w: &World) -> Vec<(ObjectId, MakerEvent, Option<HouseId>, Pos)> {
    w.effects
        .iter()
        .filter_map(|e| match e {
            Effect::Maker {
                id,
                event,
                owner,
                pos,
                ..
            } => Some((*id, *event, *owner, *pos)),
            _ => None,
        })
        .collect()
}

/// Steps until the maker `m` ends; returns the unit it made (if any) and
/// the tick of its end.
fn finish(w: &mut World, m: ObjectId) -> (Option<ObjectId>, i32) {
    for _ in 0..200 {
        step(w);
        for (id, e, _, _) in maker_events(w) {
            if id == m
                && let MakerEvent::Ended { unit, .. } = e
            {
                return (unit, w.tick);
            }
        }
    }
    panic!("maker {m:?} did not end");
}

fn infantry_of(w: &World, name: &str) -> Vec<ObjectId> {
    let ti = w.rules().index_of(Kind::Infantry, name).unwrap();
    w.objects
        .values()
        .filter(|o| o.kind == Kind::Infantry && o.type_index == ti && o.on_map)
        .map(|o| o.id)
        .collect()
}

fn mark(w: &World, c: CellXY, l: Layer, p: u8) -> Option<Mark> {
    w.cells.occ(c, l).and_then(|o| o.marks[usize::from(p)])
}

/// A ready GENE for Alpha (its building at (−6, −6)).
fn armed(r: &Ruleset) -> World {
    let mut w = game(r);
    sw_common::spawn(&mut w, "GENEB", ALPHA, -6, -6);
    ready_tick(&mut w, ALPHA, GENE, 400);
    w
}

// ---------------------------------------------------------------------------
// Claim 1: only a death converts.
// ---------------------------------------------------------------------------

// spec: S-0611 claim 1 (a surviving target keeps its type and owner and takes the ordinary health change; only the killing hit converts)
#[test]
fn claim_1_only_a_death_converts() {
    let r = rules(false);
    let mut w = game(&r);
    let v = put(&mut w, "WALKER", BETA, at(0, 0), position::RIGHT);
    let full = w.obj(v).unwrap().health;
    let h = warhead(&w, "Mutate");
    damage::apply(
        &mut w,
        v,
        Hit {
            amount: 10,
            warhead: Some(h),
            distance: 0,
            attacker: None,
            house: Some(ALPHA),
            ignore_defences: true,
            no_escape: false,
        },
    );
    let o = w.obj(v).expect("survives");
    assert_eq!(o.health, full - 10);
    assert_eq!(o.owner, Some(BETA));
    assert!(makers(&w).is_empty());
    mutate(&mut w, v, None, Some(ALPHA));
    assert!(w.obj(v).is_none(), "the killing hit converts");
    assert_eq!(makers(&w).len(), 1);
}

// spec: S-0611 claim 1 (a type marked not human, and a vehicle, never convert)
#[test]
fn claim_1_excluded_victims() {
    let r = rules(false);
    let mut w = game(&r);
    let husk = put(&mut w, "HUSK", BETA, at(0, 0), position::RIGHT);
    mutate(&mut w, husk, None, Some(ALPHA));
    assert!(makers(&w).is_empty(), "not human");
    assert!(dying(&w, husk), "S-0107 claim 17 row 4: an ordinary death");
    let car = sw_common::spawn(&mut w, "CART", BETA, 2, 0);
    mutate(&mut w, car, None, Some(ALPHA));
    assert!(makers(&w).is_empty(), "a vehicle never converts");
}

// ---------------------------------------------------------------------------
// Claim 2 and F1: the launch.
// ---------------------------------------------------------------------------

// spec: S-0611 claim 2 and F1 (in the launch tick: the ion-blast animation at P + 5 leptons, the announcement, the positional sound at P, the radar event at the target cell, whether or not infantry is reached)
#[test]
fn claim_2_launch_feedback() {
    for explosion in [false, true] {
        let r = rules(explosion);
        let mut w = armed(&r);
        let c = at(0, 0);
        fire_now(&mut w, ALPHA, GENE, c);
        let p = glasnost_sim::superweapons::cell_point(&w, c, true);
        let anims: Vec<(String, Pos)> = w
            .effects
            .iter()
            .filter_map(|e| match e {
                Effect::Anim { name, pos, .. } => Some((name.clone(), *pos)),
                _ => None,
            })
            .collect();
        assert!(
            anims.contains(&("IONB".into(), Pos::new(p.x, p.y, p.z + 5))),
            "{anims:?}"
        );
        assert_eq!((p.x, p.y), (256 * c.x + 128, 256 * c.y + 128));
        let ev: Vec<(SwEvent, Option<Pos>)> = w
            .effects
            .iter()
            .filter_map(|e| match e {
                Effect::Superweapon { event, pos, sw, .. } if *sw == Some(GENE) => {
                    Some((*event, *pos))
                }
                _ => None,
            })
            .collect();
        assert!(ev.contains(&(SwEvent::Launch, None)), "{ev:?}");
        assert!(ev.contains(&(SwEvent::GeneticSound, Some(p))), "{ev:?}");
        assert!(ev.iter().any(|e| e.0 == SwEvent::Radar), "{ev:?}");
        assert!(
            !sw_common::rec(&w, ALPHA, GENE).ready,
            "the record is spent"
        );
    }
}

// ---------------------------------------------------------------------------
// Claim 3: explosion mode.
// ---------------------------------------------------------------------------

// spec: S-0611 claim 3 (one area hit of 10 000 at P, the explosion warhead, not ignoring defences: a 30 000-strength unit at the centre keeps 20 000)
#[test]
fn claim_3_explosion_amount() {
    let r = rules(true);
    let mut w = armed(&r);
    let big = put(&mut w, "BIG", BETA, at(0, 0), position::CENTRE);
    fire_now(&mut w, ALPHA, GENE, at(0, 0));
    assert_eq!(w.obj(big).unwrap().health, 20_000);
    assert!(makers(&w).is_empty());
}

// spec: S-0611 claim 3 (an infantry death caused by the explosion converts; the converted victim leaves play in the launch tick)
#[test]
fn claim_3_explosion_converts() {
    let r = rules(true);
    let mut w = armed(&r);
    let v = put(&mut w, "WALKER", BETA, at(0, 0), position::RIGHT);
    fire_now(&mut w, ALPHA, GENE, at(0, 0));
    assert!(w.obj(v).is_none());
    assert_eq!(makers(&w).len(), 1);
}

// ---------------------------------------------------------------------------
// Claim 4 and F2: cell mode.
// ---------------------------------------------------------------------------

// spec: S-0611 claim 4 (every infantry listed in the block is hit with its whole strength, any house, no list cut; nothing outside the block and no vehicle receives a hit)
#[test]
fn claim_4_cell_mode_block() {
    let r = rules(false);
    let mut w = armed(&r);
    let c = at(0, 0);
    let three: Vec<ObjectId> = [position::RIGHT, position::LEFT, position::BOTTOM]
        .into_iter()
        .map(|p| put(&mut w, "WALKER", BETA, c, p))
        .collect();
    let n1 = put(&mut w, "WALKER", BETA, at(1, 0), position::RIGHT);
    let own = put(&mut w, "WALKER", ALPHA, at(0, 1), position::RIGHT);
    let outer = put(&mut w, "WALKER", BETA, at(2, 0), position::RIGHT);
    let car = sw_common::spawn(&mut w, "CART", BETA, -1, 0);
    let car_hp = w.obj(car).unwrap().health;
    let outer_hp = w.obj(outer).unwrap().health;
    fire_now(&mut w, ALPHA, GENE, c);
    for id in three.iter().chain([&n1, &own]) {
        assert!(w.obj(*id).is_none(), "{id:?} converted");
    }
    assert_eq!(makers(&w).len(), 5);
    assert_eq!(w.obj(outer).unwrap().health, outer_hp);
    assert_eq!(w.obj(car).unwrap().health, car_hp);
}

// spec: S-0611 claim 4 and F2 (the hit's amount is the type strength; with a non-mutating cell warhead the deaths are ordinary)
#[test]
fn claim_4_plain_cell_warhead() {
    let mut r = rules(false);
    r.sim.general.superweapons.mutate_warhead = Some("Plain".into());
    let mut w = armed(&r);
    let big = put(&mut w, "BIG", BETA, at(0, 0), position::RIGHT);
    fire_now(&mut w, ALPHA, GENE, at(0, 0));
    assert!(makers(&w).is_empty(), "no conversion without kind 9");
    let o = w.obj(big);
    assert!(
        o.is_none_or(|o| o.health == 0),
        "the whole strength 30 000 is lethal"
    );
}

// spec: S-0611 Import mapping (a warhead reference left empty: no hits for that mode, one importer warning)
#[test]
fn missing_mode_warhead() {
    let mut r = rules(false);
    r.sim.general.superweapons.mutate_warhead = None;
    let mut w = armed(&r);
    assert!(
        w.rules()
            .superweapons
            .warnings
            .iter()
            .any(|x| x.contains("MutateWarhead"))
    );
    let v = put(&mut w, "WALKER", BETA, at(0, 0), position::RIGHT);
    fire_now(&mut w, ALPHA, GENE, at(0, 0));
    assert!(w.obj(v).is_some());
}

// ---------------------------------------------------------------------------
// Claims 6 and 7: the conversion test and the victim.
// ---------------------------------------------------------------------------

// spec: S-0611 claim 6 (row 1: a foot land multiplier of exactly 0 refuses, with no draw; a negative one does not); claim 7 (the refused victim takes die-2)
#[test]
fn claim_6_land_multiplier() {
    for (foot, converts) in [("0", false), ("-1", true), ("1", true)] {
        let mut r = rules(false);
        r.sim.land_types.insert(
            LandType::Clear,
            LandRow {
                foot: Some(d(foot)),
                ..Default::default()
            },
        );
        let mut w = game(&r);
        let v = put(&mut w, "WALKER", BETA, at(0, 0), position::RIGHT);
        let before = w.rng.taken();
        mutate(&mut w, v, None, Some(ALPHA));
        assert_eq!(!makers(&w).is_empty(), converts, "foot {foot}");
        if !converts {
            assert_eq!(w.rng.taken(), before, "row 1 takes no draw");
            assert!(dying(&w, v), "die-2 instead");
        }
    }
}

// spec: S-0611 claim 6 (row 2: no free position on the ground record refuses; a centre-class victim's choice takes one draw, a side-position victim's none)
#[test]
fn claim_6_position_choice_and_draw() {
    let r = rules(false);
    // The draws of a conversion beyond the test: those of a side victim.
    let mut w = game(&r);
    let v = put(&mut w, "WALKER", BETA, at(0, 0), position::RIGHT);
    let t0 = w.rng.taken();
    mutate(&mut w, v, None, Some(ALPHA));
    let side = w.rng.taken() - t0;
    let mut w = game(&r);
    let v = put(&mut w, "WALKER", BETA, at(0, 0), position::CENTRE);
    let t0 = w.rng.taken();
    mutate(&mut w, v, None, Some(ALPHA));
    assert_eq!(makers(&w).len(), 1);
    assert_eq!(w.rng.taken() - t0, side + 1, "one ranged draw for centre");
    // Every selectable side position taken while a centre victim crosses.
    let mut w = game(&r);
    let c = at(0, 0);
    for p in position::SIDES {
        put(&mut w, "WALKER", BETA, c, p);
    }
    let v = put(&mut w, "WALKER", BETA, c, position::CENTRE);
    mutate(&mut w, v, None, Some(ALPHA));
    assert!(makers(&w).is_empty(), "no position: refused");
    assert!(dying(&w, v), "die-2 instead");
}

// spec: S-0611 claim 6 (row 3: a building listed in the cell's ground layer refuses after the choice)
#[test]
fn claim_6_building_refuses() {
    let r = rules(false);
    let mut w = game(&r);
    let v = put(&mut w, "WALKER", BETA, at(0, 0), position::RIGHT);
    let b = sw_common::spawn(&mut w, "AUXB", BETA, 3, 3);
    // List the building in the victim's cell as an open gate would.
    w.cells.enter(at(0, 0), Layer::Ground, b, true);
    mutate(&mut w, v, None, Some(ALPHA));
    assert!(makers(&w).is_empty());
    assert!(dying(&w, v), "die-2 instead");
}

// spec: S-0611 claim 7 and Edge cases (quirks.refused_mutation_frees_position on: the refused victim's position is free from its death tick, and its removal frees it again even when a walker took it; off: taken until removal, which frees only its own mark)
#[test]
fn claim_7_refused_position() {
    for quirk in [true, false] {
        let mut r = rules(false);
        r.globals.quirks.refused_mutation_frees_position = quirk;
        r.sim.land_types.insert(
            LandType::Clear,
            LandRow {
                foot: Some(d("0")),
                ..Default::default()
            },
        );
        let mut w = game(&r);
        let c = at(0, 0);
        let v = put(&mut w, "WALKER", BETA, c, position::RIGHT);
        mutate(&mut w, v, None, Some(ALPHA));
        assert!(w.obj(v).is_some());
        assert_eq!(
            mark(&w, c, Layer::Ground, position::RIGHT).is_none(),
            quirk,
            "free during die-2 (quirk {quirk})"
        );
        // A walker takes the position meanwhile.
        let other = ObjectId(9_000);
        if quirk {
            w.cells
                .mark_infantry(c, Layer::Ground, position::RIGHT, other, BETA);
        }
        objects::remove(&mut w, v);
        let left = mark(&w, c, Layer::Ground, position::RIGHT);
        assert!(
            left.is_none(),
            "the removal frees the position (quirk {quirk})"
        );
    }
}

// spec: S-0611 claim 7 (a converted victim leaves play in its death tick, no corpse, credit as for any death)
#[test]
fn claim_7_victim_leaves_play() {
    let r = rules(false);
    let mut w = game(&r);
    let v = put(&mut w, "WALKER", BETA, at(0, 0), position::RIGHT);
    let killer = sw_common::spawn(&mut w, "CRUSHER", ALPHA, 3, 0);
    mutate(&mut w, v, Some(killer), Some(ALPHA));
    assert!(w.obj(v).is_none());
    assert!(
        !w.effects.iter().any(|e| matches!(e, Effect::Anim { .. })),
        "no death animation or corpse"
    );
}

// spec: S-0611 claim 7 (Q-1461: a crash begins only for a crash-flag type above its surface; then the victim is kept for its crash, yet the conversion animation starts in the death tick with claim 8's owner, the standing position is marked taken, and the animation later makes its unit; on open ground the flag alone does not prevent removal)
#[test]
fn claim_7_crash_victim_still_converts() {
    let r = rules(false);
    let c = at(0, 0);
    // On the ground: removed at once.
    let mut w = game(&r);
    let v = put(&mut w, "JJC", BETA, c, position::RIGHT);
    assert!(glasnost_sim::jump::is_jumpjet(&w, v));
    mutate(&mut w, v, None, Some(ALPHA));
    assert!(w.obj(v).is_none(), "the flag alone does not keep it");
    assert_eq!(makers(&w).len(), 1);
    // Above the surface: a crash, and the conversion anyway.
    let mut w = game(&r);
    let v = put(&mut w, "JJC", BETA, c, position::RIGHT);
    // MU13: 1 remaining health, so that the airborne mutating hit's raised damage of 1 kills (S-0302 claims 5 and 17).
    w.obj_mut(v).unwrap().pos.z += 300;
    w.obj_mut(v).unwrap().health = 1;
    assert!(glasnost_sim::jump::height_above(&w, v) > 0);
    let at_v = w.obj(v).unwrap().pos;
    w.effects.clear();
    let h = warhead(&w, "Mutate");
    damage::apply(
        &mut w,
        v,
        Hit {
            amount: 100,
            warhead: Some(h),
            distance: 0,
            attacker: None,
            house: Some(ALPHA),
            ignore_defences: false,
            no_escape: false,
        },
    );
    assert!(glasnost_sim::jump::crashing(&w, v), "it crashes");
    let ev = maker_events(&w);
    assert_eq!(ev.len(), 1, "one conversion animation");
    assert_eq!(ev[0].1, MakerEvent::Made);
    assert_eq!(ev[0].2, Some(ALPHA));
    assert_eq!(ev[0].3, at_v, "at its position, height included");
    assert!(
        mark(&w, c, Layer::Ground, position::RIGHT).is_some(),
        "the standing position is taken in the death tick"
    );
    let m = makers(&w)[0];
    let (unit, _) = finish(&mut w, m);
    assert!(unit.is_some(), "it makes its unit");
}

// spec: S-0107 claim 17 row 7 (revision 18: for a successful replacement a crash changes only the victim's removal; the conversion animation still starts in the death tick, the standing position is marked taken again at crash entry, and the animation's later freeing need not wait for the crash to end)
// spec: S-0611 claims 7 and 8 (the crash keeps the victim; the animation runs its ordinary course)
#[test]
fn s0107_claim_17_row_7_crash_changes_only_the_removal() {
    let r = rules(false);
    let c = at(0, 0);
    let kill = |w: &mut World, v: ObjectId| {
        let h = warhead(w, "Mutate");
        damage::apply(
            w,
            v,
            Hit {
                amount: 100,
                warhead: Some(h),
                distance: 0,
                attacker: None,
                house: Some(ALPHA),
                ignore_defences: false,
                no_escape: false,
            },
        );
    };
    // On the ground: the victim is removed; the animation's span from the
    // death tick to its end.
    let mut w = game(&r);
    let v = put(&mut w, "JJC", BETA, c, position::RIGHT);
    w.obj_mut(v).unwrap().health = 1;
    let t0 = w.tick;
    kill(&mut w, v);
    assert!(w.obj(v).is_none());
    let m = makers(&w)[0];
    let (_, end) = finish(&mut w, m);
    let span = end - t0;
    // Above the surface: a crash, the mark taken at crash entry, and the
    // animation ends after the same span while the victim still crashes.
    let mut w = game(&r);
    let v = put(&mut w, "JJC", BETA, c, position::RIGHT);
    w.obj_mut(v).unwrap().pos.z += 2000;
    w.obj_mut(v).unwrap().health = 1;
    let t0 = w.tick;
    w.effects.clear();
    kill(&mut w, v);
    assert!(glasnost_sim::jump::crashing(&w, v));
    assert_eq!(maker_events(&w).len(), 1, "the animation starts at once");
    assert!(
        mark(&w, c, Layer::Ground, position::RIGHT).is_some(),
        "marked taken at crash entry"
    );
    let m = makers(&w)[0];
    let (unit, end) = finish(&mut w, m);
    assert_eq!(end - t0, span, "the end does not wait for the crash");
    assert!(
        w.obj(v)
            .is_some_and(|_| glasnost_sim::jump::crashing(&w, v)),
        "the victim is still crashing"
    );
    assert!(unit.is_some());
}

// ---------------------------------------------------------------------------
// Claim 8: the conversion animation, owner and occupancy.
// ---------------------------------------------------------------------------

// spec: S-0611 claim 8 (the owner: the attacker's current owner, else the attacking house, else none; at the victim's exact position)
#[test]
fn claim_8_owner() {
    let r = rules(false);
    let mut w = game(&r);
    let killer = sw_common::spawn(&mut w, "CRUSHER", ALPHA, 3, 0);
    // The firer's current owner, not the attacking house of the hit.
    w.obj_mut(killer).unwrap().owner = Some(HouseId(1));
    let v = put(&mut w, "WALKER", BETA, at(0, 0), position::RIGHT);
    let at_v = w.obj(v).unwrap().pos;
    w.effects.clear();
    mutate(&mut w, v, Some(killer), Some(ALPHA));
    let ev = maker_events(&w);
    assert_eq!(ev.len(), 1);
    assert_eq!(ev[0].1, MakerEvent::Made);
    assert_eq!(ev[0].2, Some(BETA), "the attacker's current owner");
    assert_eq!(ev[0].3, at_v, "the victim's exact position");
    let v = put(&mut w, "WALKER", BETA, at(2, 0), position::RIGHT);
    w.effects.clear();
    mutate(&mut w, v, None, Some(ALPHA));
    assert_eq!(maker_events(&w)[0].2, Some(ALPHA), "the attacking house");
    let v = put(&mut w, "WALKER", BETA, at(4, 0), position::RIGHT);
    w.effects.clear();
    mutate(&mut w, v, None, None);
    assert_eq!(maker_events(&w)[0].2, None, "no house");
}

// spec: S-0611 claim 8 and Edge cases (quirks.infantry_animation_occupancy on: free between the death and the first update; each update marks the position with the animation's owner as infantry house; at the end the position is freed whatever holds it)
#[test]
fn claim_8_occupancy_on() {
    let r = rules(false);
    let mut w = game(&r);
    let c = at(0, 0);
    let v = put(&mut w, "WALKER", BETA, c, position::RIGHT);
    mutate(&mut w, v, None, Some(ALPHA));
    let m = makers(&w)[0];
    assert!(
        mark(&w, c, Layer::Ground, position::RIGHT).is_none(),
        "free"
    );
    // A walker (of the animation's owner, so that the cell still takes
    // its unit) takes the position in the gap.
    let other = put(&mut w, "WALKER", ALPHA, c, position::RIGHT);
    step(&mut w);
    assert_eq!(
        mark(&w, c, Layer::Ground, position::RIGHT).map(|x| x.by),
        Some(Marker::Anim(m, Some(ALPHA)))
    );
    assert_eq!(w.cells.occ(c, Layer::Ground).unwrap().house, Some(ALPHA));
    let _ = other;
    // The end frees the position (the made unit then marks its own).
    let (unit, _) = finish(&mut w, m);
    let unit = unit.expect("made");
    let marks: Vec<Marker> = w
        .cells
        .occ(c, Layer::Ground)
        .unwrap()
        .marks
        .iter()
        .flatten()
        .map(|x| x.by)
        .collect();
    assert!(
        !marks
            .iter()
            .any(|x| matches!(x, Marker::Infantry(o, _) if *o == other)),
        "the walker lost its mark: {marks:?}"
    );
    assert!(
        marks
            .iter()
            .any(|x| matches!(x, Marker::Infantry(o, _) if *o == unit))
    );
}

// spec: S-0611 Edge cases (quirks.infantry_animation_occupancy off: the animation holds its position only while no other unit holds it, leaves the infantry house alone while other infantry stands there, and frees only its own mark)
#[test]
fn claim_8_occupancy_off() {
    let mut r = rules(false);
    r.globals.quirks.infantry_animation_occupancy = false;
    let mut w = game(&r);
    let c = at(0, 0);
    let v = put(&mut w, "WALKER", BETA, c, position::RIGHT);
    mutate(&mut w, v, None, Some(ALPHA));
    let m = makers(&w)[0];
    let other = put(&mut w, "WALKER", BETA, c, position::RIGHT);
    step(&mut w);
    assert_eq!(
        mark(&w, c, Layer::Ground, position::RIGHT).map(|x| x.by),
        Some(Marker::Infantry(other, BETA)),
        "the walker keeps the position"
    );
    assert_eq!(w.cells.occ(c, Layer::Ground).unwrap().house, Some(BETA));
    // The end frees only the animation's own mark; Beta's walker in the
    // cell then refuses Alpha's unit (claim 11), so the animation stays.
    let mut refused = false;
    for _ in 0..40 {
        step(&mut w);
        if maker_events(&w)
            .iter()
            .any(|e| e.0 == m && e.1 == MakerEvent::Refused)
        {
            refused = true;
            break;
        }
    }
    assert!(refused);
    assert_eq!(
        mark(&w, c, Layer::Ground, position::RIGHT).map(|x| x.by),
        Some(Marker::Infantry(other, BETA)),
        "only the animation's own mark is freed"
    );
    sw_common::remove(&mut w, other);
    assert!(finish(&mut w, m).0.is_some());
}

// ---------------------------------------------------------------------------
// Claims 9 to 12: creation at the end.
// ---------------------------------------------------------------------------

// spec: S-0611 claims 10 and 12, F3 (a fresh unit of the made type for the owner, full health, facing 24 576, at the position; it appears in the update in which the animation ends)
#[test]
fn claim_10_new_unit() {
    let r = rules(false);
    let mut w = game(&r);
    let c = at(0, 0);
    let v = put(&mut w, "WALKER", BETA, c, position::RIGHT);
    w.obj_mut(v).unwrap().health = 5;
    let death = w.tick;
    mutate(&mut w, v, None, Some(ALPHA));
    let m = makers(&w)[0];
    let mut seen = None;
    let (unit, end) = loop {
        step(&mut w);
        if seen.is_none() && !infantry_of(&w, "GROWN").is_empty() {
            seen = Some(w.tick);
        }
        if let Some((_, MakerEvent::Ended { unit, conversion }, _, _)) = maker_events(&w)
            .into_iter()
            .find(|e| e.0 == m && matches!(e.1, MakerEvent::Ended { .. }))
        {
            assert!(conversion);
            break (unit.unwrap(), w.tick);
        }
    };
    assert_eq!(seen, Some(end), "the unit appears in the end's update");
    // spec: S-0109 claim 16 (lifetime max(d, u + 1 − c) + (E − 1)d = E × d + 1 ticks for d = 1 and a first update u = c + 1: 5 for four frames)
    assert_eq!(end - death, 5);
    let o = w.obj(unit).unwrap();
    assert_eq!(o.owner, Some(ALPHA));
    assert_eq!(o.health, GROWN_HP);
    assert_eq!(o.body.target, mutation::MADE_FACING);
    assert_eq!(mutation::MADE_FACING, 24_576);
    assert_eq!(o.pos.cell(), c);
    assert!(!w.mutation.makers.contains_key(&m), "removed");
}

// spec: S-0611 claim 9 and F4 (an index outside the list creates nothing and the importer warns; a follow-up is followed first and only the last type's index counts)
#[test]
fn claim_9_index_and_followup() {
    let mut r = rules(false);
    r.sim.general.superweapons.infantry_mutate = Some("MUTB".into());
    let mut w = game(&r);
    assert!(
        w.rules()
            .superweapons
            .warnings
            .iter()
            .any(|x| x.contains("MUTB"))
    );
    let v = put(&mut w, "WALKER", BETA, at(0, 0), position::RIGHT);
    mutate(&mut w, v, None, Some(ALPHA));
    let m = makers(&w)[0];
    assert_eq!(finish(&mut w, m).0, None);
    assert!(infantry_of(&w, "GROWN").is_empty());
    // PRE makes nothing itself, its follow-up MUTA makes the unit.
    let mut r = rules(false);
    r.sim.general.superweapons.infantry_mutate = Some("PRE".into());
    let mut w = game(&r);
    let v = put(&mut w, "WALKER", BETA, at(0, 0), position::RIGHT);
    mutate(&mut w, v, None, Some(ALPHA));
    let m = makers(&w)[0];
    assert!(finish(&mut w, m).0.is_some());
    assert_eq!(infantry_of(&w, "GROWN").len(), 1);
}

// spec: S-0611 claim 9 (no civilian house: an ownerless animation creates nothing; a defeated owner keeps the unit)
#[test]
fn claim_9_without_civilian_house() {
    let r = rules(false);
    let mut w = game(&r);
    let v = put(&mut w, "WALKER", BETA, at(0, 0), position::RIGHT);
    mutate(&mut w, v, None, None);
    let m = makers(&w)[0];
    assert_eq!(finish(&mut w, m).0, None, "ownerless: nothing");
    let v = put(&mut w, "WALKER", BETA, at(3, 0), position::RIGHT);
    mutate(&mut w, v, None, Some(ALPHA));
    let m = makers(&w)[0];
    w.houses[0].econ.result.defeated = true;
    let unit = finish(&mut w, m).0.expect("made");
    assert_eq!(w.obj(unit).unwrap().owner, Some(ALPHA));
}

/// A game on `r` with a neutral civilian-side house Civ (house 2).
fn civ_game(r: &mut Ruleset) -> World {
    use glasnost_rules::sim::Country;
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
    reg(r, "countries", "Civland");
    let mut s = testkit::setup(1);
    s.houses.push(glasnost_sim::world::HouseSetup {
        name: "Civ".into(),
        country: "Civland".into(),
        human: false,
        start: None,
        role: glasnost_sim::world::HouseRole::Neutral,
        ai: false,
    });
    let rules = std::sync::Arc::new(glasnost_sim::rules::SimRules::new(r, "synthetic-test"));
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

// spec: S-0611 claim 9 (an ownerless or defeated owner's animation changes owner to the civilian house and the unit goes to it); claim 10 (a neutral house's unit is given hunt)
#[test]
fn claim_9_civilian_fallback() {
    const CIV: HouseId = HouseId(2);
    let mut r = rules(false);
    let mut w = civ_game(&mut r);
    let v = put(&mut w, "WALKER", BETA, at(0, 0), position::RIGHT);
    mutate(&mut w, v, None, None);
    let m = makers(&w)[0];
    let unit = loop {
        step(&mut w);
        let ev = maker_events(&w);
        if let Some(e) = ev.iter().find(|e| e.0 == m && e.1 == MakerEvent::Owner) {
            assert_eq!(e.2, Some(CIV));
        }
        if let Some((_, MakerEvent::Ended { unit, .. }, _, _)) = ev
            .iter()
            .find(|e| e.0 == m && matches!(e.1, MakerEvent::Ended { .. }))
        {
            break unit.unwrap();
        }
    };
    assert_eq!(w.obj(unit).unwrap().owner, Some(CIV));
    assert_eq!(
        w.obj(unit)
            .unwrap()
            .beh
            .queued
            .or(w.obj(unit).unwrap().beh.current),
        Some(glasnost_sim::rules::Behaviour::Hunt)
    );
    // A defeated owner: the civilian house.
    let v = put(&mut w, "WALKER", BETA, at(3, 0), position::RIGHT);
    mutate(&mut w, v, None, Some(ALPHA));
    let m = makers(&w)[0];
    w.houses[0].econ.result.defeated = true;
    let unit = finish(&mut w, m).0.expect("made");
    assert_eq!(w.obj(unit).unwrap().owner, Some(CIV));
}

// spec: S-0611 claim 10 (a player house's unit gets no behaviour from its creation)
#[test]
fn claim_10_player_unit_no_hunt() {
    let r = rules(false);
    let mut w = game(&r);
    let v = put(&mut w, "WALKER", BETA, at(0, 0), position::RIGHT);
    mutate(&mut w, v, None, Some(ALPHA));
    let m = makers(&w)[0];
    let unit = finish(&mut w, m).0.unwrap();
    assert_ne!(
        w.obj(unit)
            .unwrap()
            .beh
            .queued
            .or(w.obj(unit).unwrap().beh.current),
        Some(glasnost_sim::rules::Behaviour::Hunt)
    );
}

// spec: S-0611 claims 11 and 14 (a refused placement: no unit, the animation steps back one frame and retries when that frame is next due; our engine builds no never-placed unit, but the attempt's creation number is consumed)
#[test]
fn claim_11_placement_refused() {
    let r = rules(false);
    let mut w = game(&r);
    let c = at(0, 0);
    let v = put(&mut w, "WALKER", BETA, c, position::RIGHT);
    mutate(&mut w, v, None, Some(ALPHA));
    let m = makers(&w)[0];
    let car = sw_common::spawn(&mut w, "CART", BETA, 0, 0);
    let objects_before = w.objects.len();
    let next_before = w.next_id;
    let mut refused = 0;
    for _ in 0..12 {
        let draws = w.rng.taken();
        step(&mut w);
        let n = maker_events(&w)
            .iter()
            .filter(|e| e.0 == m && e.1 == MakerEvent::Refused)
            .count();
        if n > 0 {
            // spec: S-0611 claim 14 (a refused attempt under a vehicle takes the one raw creation draw and no position-choice draw)
            assert_eq!(w.rng.taken() - draws, n as u64, "one raw draw per attempt");
        }
        refused += n;
    }
    assert!(refused >= 2, "it keeps retrying: {refused}");
    assert!(infantry_of(&w, "GROWN").is_empty());
    assert!(w.mutation.makers.contains_key(&m), "the animation stays");
    assert_eq!(w.objects.len(), objects_before, "no never-placed unit");
    // spec: S-0611 claim 14 (Q-1460: each refused attempt consumes one creation number, never given back)
    assert_eq!(
        w.next_id,
        next_before + refused as u32,
        "one number per refused attempt, none given back"
    );
    sw_common::remove(&mut w, car);
    let (unit, _) = finish(&mut w, m);
    assert!(unit.is_some());
}

/// Runs a refused-then-made conversion of a victim at position `p` with a
/// vehicle blocking the cell (MU12's setup), checking every refused
/// attempt's draws and numbers against `set` members, and returns the made
/// unit with the draws of its creation tick.
fn refused_then_made(r: &Ruleset, p: u8, set: u32) -> (World, ObjectId, Vec<ObjectId>, u64) {
    let mut w = game(r);
    let c = at(0, 0);
    let v = put(&mut w, "WALKER", BETA, c, p);
    mutate(&mut w, v, None, Some(ALPHA));
    let m = makers(&w)[0];
    let car = sw_common::spawn(&mut w, "CART", BETA, 0, 0);
    let objects_before = w.objects.len();
    let launchers_before = w.spawned.launcher.len();
    let controllers_before = w.mind.controllers.len();
    let next_before = w.next_id;
    let mut refused = 0;
    for _ in 0..12 {
        let draws = w.rng.taken();
        step(&mut w);
        let n = maker_events(&w)
            .iter()
            .filter(|e| e.0 == m && e.1 == MakerEvent::Refused)
            .count();
        if n > 0 {
            // spec: S-0611 claim 14 and F5 (a refused attempt under a vehicle takes one raw creation draw per member of its creation set and no position-choice draw)
            assert_eq!(w.rng.taken() - draws, n as u64 * u64::from(set));
        }
        refused += n;
    }
    assert!(refused >= 2, "it keeps retrying: {refused}");
    // spec: S-0611 claim 14 (a refused set never enters play; our engine keeps none of it, nor the records its creation made)
    assert_eq!(w.objects.len(), objects_before);
    assert_eq!(w.spawned.launcher.len(), launchers_before);
    assert_eq!(w.mind.controllers.len(), controllers_before);
    // spec: S-0611 claim 14 (one creation number per member of the set at each attempt, never given back)
    assert_eq!(w.next_id, next_before + refused as u32 * set);
    sw_common::remove(&mut w, car);
    let (unit, mut rng, draws, first) = loop {
        let (rng, draws, first) = (w.rng.clone(), w.rng.taken(), w.next_id);
        step(&mut w);
        if let Some((_, MakerEvent::Ended { unit, .. }, _, _)) = maker_events(&w)
            .into_iter()
            .find(|e| e.0 == m && matches!(e.1, MakerEvent::Ended { .. }))
        {
            break (unit.unwrap(), rng, draws, first);
        }
    };
    let made: Vec<ObjectId> = w.objects.range(unit..).map(|(&k, _)| k).collect();
    assert_eq!(unit.0, first);
    assert_eq!(made.len() as u32, set, "the whole set, kept on success");
    for &x in &made {
        // spec: S-0611 F5 (each member's raw value, then its number, in sequence order: the set's draws come first in the attempt)
        assert_eq!(w.obj(x).unwrap().report_value, (rng.raw() & 0xFFFF) as u16);
        // spec: S-0611 claim 14 (every member owned by the attempt's house)
        assert_eq!(w.obj(x).unwrap().owner, Some(ALPHA));
    }
    let taken = w.rng.taken() - draws;
    (w, unit, made, taken)
}

/// GROWN with a spawned type (SPYP ×2) and a slave type (WALKER ×3): MU12's
/// combined variant.
fn set_rules() -> Ruleset {
    let mut r = rules(false);
    let b = r.sim.infantry.get_mut("GROWN").unwrap();
    b.combat.spawns = Some("SPYP".into());
    b.combat.spawns_number = Some(2);
    b.econ.enslaves = Some("WALKER".into());
    b.econ.slaves_number = Some(3);
    r
}

// spec: S-0611 claim 14 and F5 (Q-1460: a made type with k = 2 slot objects and m = 3 slaves: each attempt, refused or made, takes 1 + k + m raw draws and as many numbers; the made unit's number is the lowest, then the slot objects', then the slaves'; none is undone)
#[test]
fn claim_14_creation_set_draws_and_numbers() {
    let r = set_rules();
    let (w, unit, made, draws) = refused_then_made(&r, position::RIGHT, 6);
    // spec: S-0611 claim 10 (a side-class point: no position-choice draw)
    assert_eq!(draws, 6);
    let c = w.obj(unit).unwrap().spawn.clone().unwrap();
    let slots: Vec<ObjectId> = c.slots.iter().map(|s| s.object.unwrap()).collect();
    assert_eq!(slots, made[1..3]);
    let slaves = glasnost_sim::econ::slaves::slaves_of(&w, unit);
    assert_eq!(slaves, made[3..6]);
    for &x in &made[1..] {
        // spec: S-0611 claim 14 (after a successful attempt the slot objects and slaves stay off the map)
        assert!(!w.obj(x).unwrap().on_map);
    }
}

// spec: S-0611 claim 14 and F5 (claim 10's position-choice draw comes after the whole creation set)
#[test]
fn claim_14_position_choice_after_the_set() {
    let r = set_rules();
    let (_, _, _, draws) = refused_then_made(&r, position::CENTRE, 6);
    assert_eq!(draws, 6 + 1, "the set's six, then the ranged draw");
}

// spec: S-0611 claim 14 (a mind-control primary weapon's controller takes no draw and no number: the set is the made unit alone)
#[test]
fn claim_14_weapon_controllers_take_nothing() {
    let mut r = rules(false);
    r.sim.warheads.insert(
        "MIND".into(),
        WarheadType {
            verses: Some(vec![d("1"); 11]),
            mind_control: Some(true),
            ..Default::default()
        },
    );
    let mut ray = r.sim.weapons["Rifle"].clone();
    ray.warhead = Some("MIND".into());
    r.sim.weapons.insert("Ray".into(), ray);
    r.sim.infantry.get_mut("GROWN").unwrap().primary = Some("Ray".into());
    let (w, unit, _, draws) = refused_then_made(&r, position::RIGHT, 1);
    assert_eq!(draws, 1);
    assert!(
        w.mind.controllers.contains_key(&unit),
        "it has its controller"
    );
}

// spec: S-0611 claim 14 (revision 4, scenario MU12's controller-only variants: a parasite primary weapon's controller and an airstrike designator take no draw and no number either; each variant alone)
#[test]
fn claim_14_parasite_and_designator_take_nothing() {
    for variant in ["parasite", "designator"] {
        let mut r = rules(false);
        r.sim.warheads.insert(
            "CTRL".into(),
            WarheadType {
                verses: Some(vec![d("1"); 11]),
                parasite: Some(variant == "parasite"),
                airstrike: Some(variant == "designator"),
                ..Default::default()
            },
        );
        let mut ray = r.sim.weapons["Rifle"].clone();
        ray.warhead = Some("CTRL".into());
        r.sim.weapons.insert("Ray".into(), ray);
        let brute = r.sim.infantry.get_mut("GROWN").unwrap();
        brute.primary = Some("Ray".into());
        if variant == "designator" {
            brute.combat.airstrike_team = Some(2);
            brute.combat.airstrike_team_type = Some("WALKER".into());
        }
        let (_, _, _, draws) = refused_then_made(&r, position::RIGHT, 1);
        assert_eq!(draws, 1, "{variant}: the made unit alone");
    }
}

// spec: S-0611 claim 5 (revision 4: height alone is no conversion ban; S-0302 claim 17 resets an airborne mutating hit's amount, but one that does not ignore defences can still kill through the minimum damage, so an ordinary human type in the air with neither the jumpjet type nor a parachute converts; an ignore-defences hit stays at 0 and kills nothing)
#[test]
fn claim_5_height_is_no_conversion_ban() {
    let r = rules(false);
    let c = at(0, 0);
    for ignore in [false, true] {
        let mut w = game(&r);
        let v = put(&mut w, "WALKER", BETA, c, position::RIGHT);
        w.obj_mut(v).unwrap().pos.z += 300;
        w.obj_mut(v).unwrap().health = 1;
        assert!(w.height_above_surface(v) > 0);
        let h = warhead(&w, "Mutate");
        damage::apply(
            &mut w,
            v,
            Hit {
                amount: 100,
                warhead: Some(h),
                distance: 0,
                attacker: None,
                house: Some(ALPHA),
                ignore_defences: ignore,
                no_escape: false,
            },
        );
        if ignore {
            assert_eq!(w.obj(v).map(|o| o.health), Some(1), "amount 0: alive");
            assert!(makers(&w).is_empty());
        } else {
            assert!(w.obj(v).is_none(), "killed by the minimum damage");
            assert_eq!(makers(&w).len(), 1, "and converted in the air");
        }
    }
}

// ---------------------------------------------------------------------------
// Claim 13: each victim converts once.
// ---------------------------------------------------------------------------

// spec: S-0611 claim 13 (two launches over the same block convert each victim once; a made unit converts again, to the later house)
#[test]
fn claim_13_once_then_again() {
    let r = rules(false);
    let mut w = armed(&r);
    let v = put(&mut w, "WALKER", BETA, at(0, 0), position::RIGHT);
    fire_now(&mut w, ALPHA, GENE, at(0, 0));
    assert_eq!(makers(&w).len(), 1);
    let m = makers(&w)[0];
    assert!(w.obj(v).is_none());
    // A second mutating hit cannot reach the victim, and the animation is
    // not a target.
    mutate(&mut w, m, None, Some(BETA));
    assert_eq!(makers(&w), vec![m]);
    let unit = finish(&mut w, m).0.unwrap();
    mutate(&mut w, unit, None, Some(BETA));
    assert!(w.obj(unit).is_none());
    let m2 = makers(&w)[0];
    let again = finish(&mut w, m2).0.unwrap();
    assert_eq!(w.obj(again).unwrap().owner, Some(BETA));
}

// ---------------------------------------------------------------------------
// The ground-record quirk (claim 6 on a deck).
// ---------------------------------------------------------------------------

// spec: S-0611 claim 6 and Edge cases (quirks.mutation_test_ground_record on: a deck victim's test reads the ground record, so occupants beneath refuse it; off: the deck record)
#[test]
fn ground_record_quirk() {
    for quirk in [true, false] {
        let mut r = rules(false);
        r.globals.quirks.mutation_test_ground_record = quirk;
        let mut w = game(&r);
        let c = at(0, 0);
        w.cells.mark_high_bridge(c, 24);
        let v = put(&mut w, "WALKER", BETA, c, position::CENTRE);
        {
            let o = w.obj_mut(v).unwrap();
            o.layer = Layer::Deck;
            o.pos.z += 416;
        }
        w.cells
            .clear_infantry(c, Layer::Ground, v, glasnost_sim::objects::occ_quirks(&w));
        w.cells
            .mark_infantry(c, Layer::Deck, position::CENTRE, v, BETA);
        // The ground beneath: every position taken.
        for p in [
            position::CENTRE,
            position::TOP,
            position::RIGHT,
            position::LEFT,
            position::BOTTOM,
        ] {
            w.cells
                .mark_infantry(c, Layer::Ground, p, ObjectId(9_000 + u32::from(p)), BETA);
        }
        mutate(&mut w, v, None, Some(ALPHA));
        assert_eq!(makers(&w).is_empty(), quirk, "quirk {quirk}");
    }
}

// ---------------------------------------------------------------------------
// Weapon delivery and save/load.
// ---------------------------------------------------------------------------

// spec: S-0611 claims 1 and 8 (a weapon shot whose warhead has the mutating death kind converts like a launch; the firer's owner owns the animation)
#[test]
fn weapon_delivery() {
    let mut r = rules(false);
    r.sim.weapons.get_mut("Gun").unwrap().warhead = Some("Mutate".into());
    r.sim.weapons.get_mut("Gun").unwrap().damage = Some(1_000);
    let mut w = game(&r);
    let tank = sw_common::spawn(&mut w, "CRUSHER", ALPHA, -3, 0);
    let v = put(&mut w, "WALKER", BETA, at(0, 0), position::RIGHT);
    glasnost_sim::orders::issue(
        &mut w,
        ALPHA,
        glasnost_sim::orders::Order::Behaviour {
            actor: tank,
            behaviour: glasnost_sim::rules::Behaviour::Attack,
            attack_move: false,
            target: Some(glasnost_sim::world::Target::Object(v)),
            dest: None,
        },
    );
    let mut owner = None;
    for _ in 0..400 {
        step(&mut w);
        if let Some(e) = maker_events(&w)
            .into_iter()
            .find(|e| e.1 == MakerEvent::Made)
        {
            owner = Some(e.2);
            break;
        }
    }
    assert_eq!(owner, Some(Some(ALPHA)));
    assert!(w.obj(v).is_none());
}

// spec: S-0611 claims 8 to 12 (the animation's state survives a save and load mid-play: the same unit at the same tick, the same state hash)
#[test]
fn save_load_mid_animation() {
    let r = rules(false);
    let mut w = game(&r);
    let v = put(&mut w, "WALKER", BETA, at(0, 0), position::CENTRE);
    mutate(&mut w, v, None, Some(ALPHA));
    step(&mut w);
    step(&mut w);
    let bytes = save::save(&w);
    let mut l = save::load(&bytes, w.rules_arc(), w.catalogue_arc()).expect("load");
    assert_eq!(save::state_hash(&l), save::state_hash(&w));
    assert_eq!(l.mutation, w.mutation);
    let m = makers(&w)[0];
    let a = finish(&mut w, m);
    let b = finish(&mut l, m);
    assert_eq!(a, b);
    assert_eq!(save::state_hash(&l), save::state_hash(&w));
}

// spec: S-0611 Edge cases (absent conversion animation: the victim is removed with no replacement and the importer warns)
#[test]
fn absent_conversion_animation() {
    let mut r = rules(false);
    r.sim.general.superweapons.infantry_mutate = None;
    let mut w = game(&r);
    assert!(
        w.rules()
            .superweapons
            .warnings
            .iter()
            .any(|x| x.contains("InfantryMutate"))
    );
    let v = put(&mut w, "WALKER", BETA, at(0, 0), position::RIGHT);
    mutate(&mut w, v, None, Some(ALPHA));
    assert!(w.obj(v).is_none());
    assert!(makers(&w).is_empty());
}
