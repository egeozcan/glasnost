//! S-0312 claim 11: a vehicle that carries the bomb mark (Terms) and dies
//! within 10 leptons above a water cell skips claims 4 to 6 and 9, keeps
//! claim 7's attached bomb, and shows the game-wide wake animation and the
//! last entry of the game-wide splash list at its position in claim 9's
//! place, with no draws of its own; claims 12 to 15 follow, its passengers
//! killed, or kept inside until the crash for a type that can crash.
//! Claim 10 needs the vessel not to carry the mark. Scenario D8's
//! marked-water variant in our engine's form, on synthetic data: invented
//! types, weapons and animation names (none a registered animation type, so
//! creating one takes no draw of its own), the water cells set on an empty
//! map. Kills enter the damage path at `damage::apply`; one run lets the
//! fall of S-0511 claim 10 end in its landing hit in the tick pipeline.
//! Ledger: `docs/m8-claims/air-retaliate.md`.

use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    Decimal, LocomotionClass, Locomotor, MovementZone, UnitType, WarheadType, WeaponType,
};
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{Layer, ObjectId, Pos, World};

const SIZE: i32 = 24;
const WAKE: &str = "Ripple";
const SPLASHES: [&str; 3] = ["Spl1", "Spl2", "Spl3"];

fn dec(s: &str) -> Option<Decimal> {
    Some(Decimal(s.into()))
}

fn names(v: &[&str]) -> Option<Vec<String>> {
    Some(v.iter().map(|s| s.to_string()).collect())
}

/// An unarmed wheeled vehicle with debris (1 to 4 pieces of `Deb`), the
/// explosion list `Ex1`, `Ex2` and the destruction list `Ds1`, `Ds2`.
fn plain() -> UnitType {
    let mut t = UnitType {
        strength: Some(200),
        speed: Some(5),
        rot: Some(5),
        cost: Some(600),
        locomotion_class: Some(LocomotionClass::Wheel),
        locomotor: Some(Locomotor::Drive),
        ..Default::default()
    };
    t.combat.min_debris = Some(1);
    t.combat.max_debris = Some(5);
    t.combat.debris_anims = names(&["Deb"]);
    t.combat.explosion = names(&["Ex1", "Ex2"]);
    t.combat.destroy_anim = names(&["Ds1", "Ds2"]);
    t
}

/// The combat ruleset with the game-wide wake animation and splash list,
/// the sinking weight 3, the infantry RIDER, and the vehicles
/// - PLAIN ([`plain`]);
/// - BOOMER, PLAIN exploding with the death weapon Boom;
/// - FERRY, PLAIN carrying two passengers; CFERRY, FERRY of a type that can
///   crash; BFERRY, FERRY exploding with the death weapon Boom;
/// - SKIFF, PLAIN on the ship locomotor, naval and heavier than the sinking
///   weight;
///
/// and the bomb warhead Ivan of damage 77.
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    // spec: S-0312 claim 6 (the death weapon's projectile tests at its own place, so the unmarked control fires it)
    r.globals.quirks.death_weapon_unplaced_projectile = false;
    let push = |r: &mut Ruleset, reg: &str, n: &str| {
        r.registries.entry(reg.into()).or_default().push(n.into());
    };
    let mut boomer = plain();
    boomer.combat.explodes = Some(true);
    boomer.combat.death_weapon = Some("Boom".into());
    let mut ferry = plain();
    ferry.passengers = Some(2);
    ferry.size_limit = dec("10");
    let mut cferry = ferry.clone();
    cferry.crashable = Some(true);
    let mut bferry = ferry.clone();
    bferry.combat.explodes = Some(true);
    bferry.combat.death_weapon = Some("Boom".into());
    let mut skiff = plain();
    skiff.locomotion_class = Some(LocomotionClass::Float);
    skiff.locomotor = Some(Locomotor::Ship);
    skiff.movement_zone = Some(MovementZone::Water);
    skiff.weight = dec("4");
    skiff.combat.naval = Some(true);
    for (n, u) in [
        ("PLAIN", plain()),
        ("BOOMER", boomer),
        ("FERRY", ferry),
        ("CFERRY", cferry),
        ("BFERRY", bferry),
        ("SKIFF", skiff),
    ] {
        push(&mut r, "vehicles", n);
        r.sim.vehicles.insert(n.into(), u);
    }
    push(&mut r, "infantry", "RIDER");
    r.sim.infantry.insert(
        "RIDER".into(),
        UnitType {
            strength: Some(100),
            speed: Some(4),
            locomotion_class: Some(LocomotionClass::Foot),
            locomotor: Some(Locomotor::Walk),
            ..Default::default()
        },
    );
    r.sim.weapons.insert(
        "Boom".into(),
        WeaponType {
            damage: Some(300),
            range: dec("1"),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("BLAST".into()),
            ..Default::default()
        },
    );
    for n in ["BLAST", "Ivan"] {
        r.sim.warheads.insert(
            n.into(),
            WarheadType {
                cell_spread: dec("1"),
                percent_at_max: dec("1"),
                verses: Some(vec![Decimal("1".into()); 11]),
                ..Default::default()
            },
        );
    }
    let g = &mut r.sim.general;
    g.wake = Some(WAKE.into());
    g.splash_list = names(&SPLASHES);
    g.ship_sinking_weight = dec("3");
    g.ivan_warhead = Some("Ivan".into());
    g.ivan_damage = Some(77);
    g.ivan_timed_delay = Some(100_000);
    r
}

fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

/// A game on `r` whose cells within 3 of the middle are water.
fn game(r: &Ruleset) -> World {
    let mut w = testkit::game_with(r, SIZE, 11);
    water(&mut w);
    w
}

/// Sets the cells within 3 of the middle to water (the land the cell tests
/// read; the synthetic map's tiles are clear ground).
fn water(w: &mut World) {
    for dx in -3..=3 {
        for dy in -3..=3 {
            w.cells.grid.get_mut(at(dx, dy)).unwrap().land = land::WATER;
        }
    }
}

/// A vehicle `name` of house 1 at cell offset (dx, 0), `dz` leptons above
/// the surface, carrying the bomb mark when `marked`.
fn put(w: &mut World, name: &str, dx: i32, dz: i32, marked: bool) -> ObjectId {
    let id = testkit::spawn(w, Kind::Vehicle, name, 1, at(dx, 0), 0);
    water(w);
    let o = w.obj_mut(id).unwrap();
    o.pos.z += dz;
    o.fall.bomb_mark = marked;
    id
}

/// Two RIDERs of house 1 boarded on `t`.
fn board(w: &mut World, t: ObjectId) -> Vec<ObjectId> {
    (0..2)
        .map(|k| {
            let p = testkit::spawn(w, Kind::Infantry, "RIDER", 1, at(5, k), 0);
            glasnost_sim::transport::board(w, p, t);
            p
        })
        .collect()
}

fn lethal() -> Hit {
    Hit {
        amount: 100_000,
        warhead: None,
        distance: 0,
        attacker: None,
        house: None,
        ignore_defences: true,
        no_escape: true,
    }
}

fn anims(w: &World) -> Vec<(String, Pos)> {
    w.effects
        .iter()
        .filter_map(|e| match e {
            Effect::Anim { name, pos, .. } => Some((name.clone(), *pos)),
            _ => None,
        })
        .collect()
}

fn names_of(w: &World) -> Vec<String> {
    anims(w).into_iter().map(|a| a.0).collect()
}

fn detonations(w: &World) -> Vec<i32> {
    w.effects
        .iter()
        .filter_map(|e| match e {
            Effect::Detonation { damage, .. } => Some(*damage),
            _ => None,
        })
        .collect()
}

/// Kills `id` and gives the draws its death took.
fn kill(w: &mut World, id: ObjectId) -> u64 {
    w.effects.clear();
    let before = w.rng.taken();
    damage::apply(w, id, lethal());
    w.rng.taken() - before
}

fn wake_and_splash(pos: Pos) -> Vec<(String, Pos)> {
    vec![(WAKE.into(), pos), (SPLASHES[2].into(), pos)]
}

// spec: S-0312 claim 11 and scenario D8's marked-water variant (a bomb-marked vehicle that dies at the water surface shows the game-wide wake animation and then the last entry of the splash list at its position, and no debris (claim 4) and none of claim 9's explosion or destruction animations; State and ordering: no draws for claim 11, F1's debris draws and claim 9's two list picks skipped); the unmarked control on the same water takes the ordinary effects and their draws; claim 15 (it then leaves play)
#[test]
fn d8_marked_water_variant_wake_and_last_splash_in_place_of_the_ordinary_effects() {
    let r = rules();
    let mut w = game(&r);
    let id = put(&mut w, "PLAIN", 0, 0, true);
    let pos = w.obj(id).unwrap().pos;
    assert_eq!(kill(&mut w, id), 0, "no draws");
    assert_eq!(anims(&w), wake_and_splash(pos));
    assert!(w.obj(id).is_none(), "it leaves play");
    // The control: the same vehicle without the mark.
    let mut w = game(&r);
    let id = put(&mut w, "PLAIN", 0, 0, false);
    assert!(kill(&mut w, id) >= 3, "debris count, pieces and two picks");
    let shown = names_of(&w);
    assert!(shown.contains(&"Deb".to_string()), "{shown:?}");
    assert!(shown.iter().any(|n| n.starts_with("Ex")), "{shown:?}");
    assert!(shown.iter().any(|n| n.starts_with("Ds")), "{shown:?}");
    assert!(!shown.contains(&WAKE.to_string()), "{shown:?}");
}

// spec: S-0312 claim 11 (r66, Q-2793, the water test: its height above the surface, from the terrain off any deck, 10 leptons or less, inclusive: at 10 leptons the wake and splash, at 11 the ordinary effects of claims 4 and 9; no lower bound, so a negative height passes)
#[test]
fn claim_11_the_10_lepton_boundary() {
    let r = rules();
    for (dz, low) in [(10, true), (11, false), (-30, true)] {
        let mut w = game(&r);
        let id = put(&mut w, "PLAIN", 0, dz, true);
        let pos = w.obj(id).unwrap().pos;
        let draws = kill(&mut w, id);
        if low {
            assert_eq!(draws, 0, "{dz}");
            assert_eq!(anims(&w), wake_and_splash(pos), "{dz}");
        } else {
            assert!(draws >= 3, "{dz}");
            assert!(!names_of(&w).contains(&WAKE.to_string()), "{dz}");
        }
    }
}

// spec: S-0312 claim 11 (above a water cell only: a marked vehicle dying on land takes the ordinary effects); Terms (the mark alone does not suppress them)
#[test]
fn claim_11_needs_a_water_cell() {
    let r = rules();
    let mut w = game(&r);
    let id = put(&mut w, "PLAIN", 6, 0, true);
    assert!(kill(&mut w, id) >= 3);
    let shown = names_of(&w);
    assert!(shown.contains(&"Deb".to_string()), "{shown:?}");
    assert!(!shown.contains(&WAKE.to_string()), "{shown:?}");
}

/// Makes the cell of `id` a high-bridge deck cell and stands `id` on its
/// deck, 416 leptons above the terrain (S-0307 Constants), at height 0
/// above the surface.
fn on_deck(w: &mut World, id: ObjectId) {
    let c = w.obj(id).unwrap().pos.cell();
    let i = w.cells.idx(c).unwrap();
    w.cells.deck[i] = true;
    w.cells.deck_entry[i] = true;
    w.cells.bridge_axis[i] = glasnost_sim::cells::Axis::X;
    w.cells.leave(c, Layer::Ground, id);
    w.cells.enter(c, Layer::Deck, id, false);
    let o = w.obj_mut(id).unwrap();
    o.layer = Layer::Deck;
    o.pos.z += 416;
    assert_eq!(w.height_above_surface(id), 0, "on the deck");
}

/// One ordinary killing hit of the warhead BLAST: defences count and the
/// passengers may escape (S-0312 claim 12; D8's marked-deck variant and D9
/// variant h).
fn ordinary(w: &World) -> Hit {
    let blast = w
        .rules()
        .combat
        .warheads
        .iter()
        .position(|h| h.name == "BLAST")
        .unwrap() as u32;
    Hit {
        warhead: Some(blast),
        ignore_defences: false,
        no_escape: false,
        ..lethal()
    }
}

// spec: S-0312 claim 11 (r66, Q-2793, scenario D8's marked-deck variant part a: a marked vehicle that dies on a bridge deck over a water cell, at height 0 above the surface measured from the deck, passes the water test, the cell's land type read whatever cell layer it is on: the wake and the last splash at its position on the deck, no debris, no claim 9 animations, no draws) and Edge cases (quirks.bomb_mark_water_death_on_deck on; off: a vehicle on a bridge deck never passes the water test and takes the ordinary effects of claims 4 and 9)
#[test]
fn d8_marked_deck_variant_a_both_ways() {
    for quirk in [true, false] {
        let mut r = rules();
        r.globals.quirks.bomb_mark_water_death_on_deck = quirk;
        let mut w = game(&r);
        let id = put(&mut w, "PLAIN", 0, 0, true);
        on_deck(&mut w, id);
        let pos = w.obj(id).unwrap().pos;
        w.effects.clear();
        let before = w.rng.taken();
        let h = ordinary(&w);
        damage::apply(&mut w, id, h);
        let draws = w.rng.taken() - before;
        if quirk {
            assert_eq!(draws, 0, "no draws");
            assert_eq!(anims(&w), wake_and_splash(pos));
        } else {
            assert!(draws >= 3, "debris count, pieces and two picks: {draws}");
            let shown = names_of(&w);
            assert!(shown.contains(&"Deb".to_string()), "{shown:?}");
            assert!(shown.iter().any(|n| n.starts_with("Ex")), "{shown:?}");
            assert!(!shown.contains(&WAKE.to_string()), "{shown:?}");
        }
        assert!(w.obj(id).is_none(), "quirk {quirk}: it leaves play");
    }
}

// spec: S-0312 claim 11 (r66, D8's marked-deck variant part a: an exploding marked vehicle on a deck over water fires no death weapon, claims 5 and 6 skipped) and Edge cases (quirks.bomb_mark_water_death_on_deck off: it fires its death weapon as an unmarked one)
#[test]
fn d8_marked_deck_death_weapon_both_ways() {
    for quirk in [true, false] {
        let mut r = rules();
        r.globals.quirks.bomb_mark_water_death_on_deck = quirk;
        let mut w = game(&r);
        let id = put(&mut w, "BOOMER", 0, 0, true);
        on_deck(&mut w, id);
        w.effects.clear();
        let h = ordinary(&w);
        damage::apply(&mut w, id, h);
        assert_eq!(detonations(&w).is_empty(), quirk, "quirk {quirk}");
    }
}

// spec: S-0312 claim 12 (r66, Q-2793, D8's marked-deck variant part b: the bomb-mark column tests the mark alone, not claim 11's water test, so a marked transport whose type cannot crash killed on land at height 0 by an ordinary hit loses all its passengers and shows claim 9's animations, no wake or splash; the unmarked control's passengers get out) and Edge cases (quirks.bomb_mark_water_death_on_deck does not touch the column)
#[test]
fn d8_marked_deck_variant_b_the_mark_alone_kills_the_passengers() {
    for quirk in [true, false] {
        for marked in [true, false] {
            let mut r = rules();
            r.globals.quirks.bomb_mark_water_death_on_deck = quirk;
            let mut w = game(&r);
            let t = put(&mut w, "FERRY", 6, 0, marked);
            let riders = board(&mut w, t);
            w.effects.clear();
            let h = ordinary(&w);
            damage::apply(&mut w, t, h);
            let out = riders
                .iter()
                .filter(|&&p| w.obj(p).is_some_and(|o| !damage::dead(o)))
                .count();
            let tag = format!("quirk {quirk}, marked {marked}");
            assert_eq!(out, if marked { 0 } else { 2 }, "{tag}");
            let shown = names_of(&w);
            assert!(
                shown.iter().any(|n| n.starts_with("Ex")),
                "{tag}: {shown:?}"
            );
            assert!(
                shown.iter().any(|n| n.starts_with("Ds")),
                "{tag}: {shown:?}"
            );
            assert!(!shown.contains(&WAKE.to_string()), "{tag}: {shown:?}");
        }
    }
}

// spec: S-0312 claim 12 (r66, Q-2793, scenario D9 variant h: the height is measured from the deck on a bridge deck, so an unmarked transport whose type cannot crash, standing on a deck 416 leptons above the terrain and at height 0 above the surface, follows the rows for 208 leptons or less: its passengers get out; the control 416 leptons above the terrain off any deck loses them all)
#[test]
fn d9_h_passengers_get_out_on_a_deck() {
    let r = rules();
    for deck in [true, false] {
        let mut w = game(&r);
        let t = put(&mut w, "FERRY", 6, 0, false);
        if deck {
            on_deck(&mut w, t);
        } else {
            w.obj_mut(t).unwrap().pos.z += 416;
        }
        let riders = board(&mut w, t);
        let h = ordinary(&w);
        damage::apply(&mut w, t, h);
        let out = riders
            .iter()
            .filter(|&&p| w.obj(p).is_some_and(|o| !damage::dead(o)))
            .count();
        assert_eq!(out, if deck { 2 } else { 0 }, "on a deck {deck}");
    }
}

// spec: S-0312 claim 11 (claims 5 and 6 skipped: an exploding marked vehicle fires no death weapon) and claim 7 with S-0519 claim 5 (its attached bomb still goes off once, at the end of the shared effects); the unmarked exploding control fires its death weapon and its bomb
#[test]
fn claim_11_skips_the_death_weapon_and_keeps_the_attached_bomb() {
    let r = rules();
    for marked in [true, false] {
        let mut w = game(&r);
        let id = put(&mut w, "BOOMER", 0, 0, marked);
        let t = w.tick;
        w.obj_mut(id).unwrap().combat.bomb = Some(glasnost_sim::bomb::Bomb {
            planter: None,
            house: None,
            planted: t,
        });
        kill(&mut w, id);
        let d = detonations(&w);
        if marked {
            assert_eq!(d, vec![77], "the bomb only");
        } else {
            assert_eq!(d.len(), 2, "the death weapon, then the bomb: {d:?}");
            assert_eq!(d[1], 77);
        }
    }
}

// spec: S-0312 claims 11 and 12 (a marked vehicle low over water whose type cannot crash: its passengers do not get out, all die) and claim 5 skipped for an exploding one (they die by claim 12 instead, the death weapon not fired)
#[test]
fn claim_11_passengers_die_when_it_cannot_crash() {
    let r = rules();
    for name in ["FERRY", "BFERRY"] {
        let mut w = game(&r);
        let t = put(&mut w, name, 0, 0, true);
        let riders = board(&mut w, t);
        assert_eq!(glasnost_sim::transport::count(&w, t), 2);
        kill(&mut w, t);
        for p in riders {
            assert!(
                w.obj(p).is_none_or(damage::dead),
                "{name}: no passenger gets out"
            );
        }
        assert!(detonations(&w).is_empty(), "{name}: no death weapon");
        assert!(w.obj(t).is_none());
    }
}

// spec: S-0312 claims 11 and 12 (a marked vehicle within 10 leptons above water whose type can crash keeps its passengers inside: none gets out) and claim 15 (our engine starts a vehicle's crash only for jumpjet movement, S-0502 claim 18, so this wheeled one leaves play at the end of its death effects and its passengers are lost with it, S-0507 claim 16)
#[test]
fn claim_11_a_crashable_type_keeps_its_passengers_inside() {
    let r = rules();
    let mut w = game(&r);
    let t = put(&mut w, "CFERRY", 0, 5, true);
    let pos = w.obj(t).unwrap().pos;
    let riders = board(&mut w, t);
    kill(&mut w, t);
    assert_eq!(anims(&w), wake_and_splash(pos));
    assert!(w.obj(t).is_none(), "it leaves play");
    for p in riders {
        assert!(w.obj(p).is_none_or(damage::dead), "no passenger gets out");
    }
}

// spec: S-0312 claim 10 (a vessel that carries the bomb mark does not sink) and claim 11 (it shows the wake and the last splash and leaves play, its type not crashable)
#[test]
fn claim_10_a_marked_vessel_does_not_sink() {
    let r = rules();
    let mut w = game(&r);
    let s = put(&mut w, "SKIFF", 0, 0, true);
    let pos = w.obj(s).unwrap().pos;
    assert_eq!(kill(&mut w, s), 0);
    assert_eq!(anims(&w), wake_and_splash(pos));
    assert!(w.obj(s).is_none(), "no sinking reprieve");
    // The control sinks.
    let mut w = game(&r);
    let s = put(&mut w, "SKIFF", 0, 0, false);
    kill(&mut w, s);
    assert!(glasnost_sim::naval::sinking(&w, s));
}

// spec: S-0312 Edge cases (departure: an empty splash list shows no splash, the wake alone) and claim 11 (no wake animation in the rules: the splash alone)
#[test]
fn claim_11_without_a_splash_list_or_a_wake() {
    let mut r = rules();
    r.sim.general.splash_list = Some(vec![]);
    let mut w = game(&r);
    let id = put(&mut w, "PLAIN", 0, 0, true);
    let pos = w.obj(id).unwrap().pos;
    assert_eq!(kill(&mut w, id), 0);
    assert_eq!(anims(&w), vec![(WAKE.to_string(), pos)]);
    let mut r = rules();
    r.sim.general.wake = None;
    let mut w = game(&r);
    let id = put(&mut w, "PLAIN", 0, 0, true);
    let pos = w.obj(id).unwrap().pos;
    assert_eq!(kill(&mut w, id), 0);
    assert_eq!(anims(&w), vec![(SPLASHES[2].to_string(), pos)]);
}

// spec: S-0312 claim 11 and scenario D8's marked-water variant through the real path: a vehicle that falls with the bomb mark (S-0511 claim 10, S-0503 claim 11) over water is killed by its landing hit in the tick pipeline and shows the wake and the last splash, none of the ordinary effects
#[test]
fn d8_marked_water_variant_through_the_landing_hit() {
    let r = rules();
    let mut w = game(&r);
    let id = put(&mut w, "PLAIN", 0, 300, false);
    glasnost_sim::fall::start(&mut w, id, true);
    w.effects.clear();
    let mut n = 0;
    while w.obj(id).is_some() {
        testkit::run(&mut w, 1);
        n += 1;
        assert!(n < 200, "it never landed");
    }
    let shown = names_of(&w);
    assert!(shown.contains(&WAKE.to_string()), "{shown:?}");
    assert!(shown.contains(&SPLASHES[2].to_string()), "{shown:?}");
    for n in ["Deb", "Ex1", "Ex2", "Ds1", "Ds2"] {
        assert!(!shown.contains(&n.to_string()), "{shown:?}");
    }
}
