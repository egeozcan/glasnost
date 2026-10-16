//! The own AI builds the building of every superweapon kind its side can
//! build, found through the rules' superweapon types that the buildings
//! name (S-0512 Terms, M6c step 2b; PLAN §4.7), and does not build one of a
//! kind it already has a building for. The tests use invented types: one
//! superweapon type per kind word, each given by a cheap building that a
//! house can build; no name is known to the AI.

use glasnost_ai::{fixture, game::Game};
use glasnost_rules::sim::Decimal;
use glasnost_rules::sim_econ::SuperweaponType;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{HouseId, World};

/// The kinds under test: the invented type and building names, and the
/// kind word the type carries (S-0512 claim 24).
const KINDS: [(&str, &str, &str); 8] = [
    ("MSL", "MSLB", "MultiMissile"),
    ("STM", "STMB", "LightningStorm"),
    ("IRON", "IRNB", "IronCurtain"),
    ("SHLD", "SHDB", "ForceShield"),
    ("SPYW", "SPYB", "SpyPlane"),
    ("CHRN", "CHRB", "ChronoSphere"),
    ("DROP", "DRPB", "ParaDrop"),
    ("AMER", "AMDB", "AmerParaDrop"),
];

/// The skirmish ruleset with the superweapon types of `kinds` (indices into
/// [`KINDS`]) and a cheap building for each; the teleport's building also
/// gives the linked second stage. Costs differ, so the order is fixed.
fn rules(kinds: &[usize]) -> std::sync::Arc<glasnost_sim::rules::SimRules> {
    let mut r = fixture::ruleset();
    let reg = |r: &mut glasnost_rules::schema::Ruleset, k: &str, n: &str| {
        r.registries.entry(k.into()).or_default().push(n.into());
    };
    for &i in kinds {
        let (sw, giver, word) = KINDS[i];
        let mut t = SuperweaponType {
            kind: Some(word.into()),
            action: Some(
                match word {
                    "MultiMissile" => "Nuke",
                    "LightningStorm" => "LightningStorm",
                    "ParaDrop" | "AmerParaDrop" => "ParaDrop",
                    w => w,
                }
                .into(),
            ),
            recharge_time: Some(Decimal("0.1".into())),
            is_powered: Some(false),
            ..Default::default()
        };
        let mut second = None;
        if word == "ChronoSphere" {
            t.pre_click = Some(true);
            let mut warp = SuperweaponType {
                kind: Some("ChronoWarp".into()),
                action: Some("ChronoWarp".into()),
                recharge_time: Some(Decimal("0.1".into())),
                is_powered: Some(false),
                post_click: Some(true),
                pre_dependent: Some("ChronoSphere".into()),
                ..Default::default()
            };
            warp.pre_click = None;
            r.sim.superweapons.insert("CHWP".into(), warp);
            reg(&mut r, "superweapons", "CHWP");
            second = Some("CHWP");
        }
        r.sim.superweapons.insert(sw.into(), t);
        reg(&mut r, "superweapons", sw);
        let mut b = fixture::building(2, 2, 200 + 100 * i as i32, -10);
        b.econ.super_weapon = Some(sw.into());
        b.econ.super_weapon2 = second.map(Into::into);
        r.sim.buildings.insert(giver.into(), b);
        reg(&mut r, "buildings", giver);
    }
    std::sync::Arc::new(glasnost_sim::rules::SimRules::new(&r, "synthetic-sw-build"))
}

fn start(kinds: &[usize]) -> Game {
    Game::new(
        rules(kinds),
        &fixture::map(),
        "synthetic-sw-build",
        fixture::catalogue(),
        fixture::setup(5),
    )
    .expect("the synthetic skirmish loads")
}

/// How many buildings of type `name` house `h` has on the map.
fn count(w: &World, h: u8, name: &str) -> usize {
    w.objects
        .values()
        .filter(|o| o.owner == Some(HouseId(h)) && o.on_map && o.health > 0)
        .filter(|o| {
            o.kind == Kind::Building && w.rules().buildings[o.type_index as usize].name == name
        })
        .count()
}

// Every kind on its own: a house that can build the giving building builds
// it (S-0512 Terms: the type is named by the building's first field).
#[test]
fn builds_the_building_of_each_kind() {
    for (i, kind) in KINDS.iter().enumerate() {
        let giver = kind.1;
        let mut g = start(&[i]);
        while g.world.tick < 12_000
            && g.outcome().is_none()
            && (0..2).all(|h| count(&g.world, h, giver) == 0)
        {
            g.step();
        }
        assert!(
            (0..2).any(|h| count(&g.world, h, giver) > 0),
            "{} ({}) was built by tick {}",
            giver,
            kind.2,
            g.world.tick
        );
    }
}

// All kinds at once: a house ends with one building of every kind and
// none twice (it does not build a second building for a kind it has).
#[test]
fn builds_one_building_of_every_kind() {
    let all: Vec<usize> = (0..KINDS.len()).collect();
    let mut g = start(&all);
    let mut best = [0usize; 2];
    while g.world.tick < 30_000 && g.outcome().is_none() {
        g.step();
        for h in 0..2u8 {
            let n = KINDS.iter().filter(|k| count(&g.world, h, k.1) > 0).count();
            best[usize::from(h)] = best[usize::from(h)].max(n);
        }
        for h in 0..2u8 {
            for k in KINDS {
                assert!(
                    count(&g.world, h, k.1) <= 1,
                    "house {h} has a second {} at tick {}",
                    k.1,
                    g.world.tick
                );
            }
        }
        if best.iter().any(|&n| n == KINDS.len()) {
            break;
        }
    }
    assert!(
        best.iter().any(|&n| n == KINDS.len()),
        "a house built every kind (best {best:?}) by tick {}",
        g.world.tick
    );
}
