//! Claim tests for S-0312's crews and survivors (claims 13, 18, 19 (g),
//! 21, 24 and 25, F4 and F5) and S-0408 claim 7's sale survivors, on
//! synthetic data: invented infantry, vehicle and building types, invented
//! sides, divisors and refunds (the refund override, so that K is exact).
//! Kills go through damage resolution and sales through the sale order or
//! the sale's own release step; the draws are checked by replaying the
//! specs' draw lists on a copy of the random stream.

use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_rules::schema::{Ruleset, Side};
use glasnost_rules::sim::{BuildingType, LocomotionClass, Locomotor, SequenceEntry, UnitType};
use glasnost_sim::combat::damage::{self, Hit, HitResult};
use glasnost_sim::combat::{death, survivors};
use glasnost_sim::rng::Rng;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::trace;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};

const SIZE: i32 = 24;

fn seq(count: i32) -> SequenceEntry {
    SequenceEntry {
        start: 0,
        count,
        multiplier: 0,
        direction: None,
    }
}

fn soldier(strength: i32) -> UnitType {
    let mut t = UnitType {
        strength: Some(strength),
        speed: Some(4),
        locomotion_class: Some(LocomotionClass::Foot),
        locomotor: Some(Locomotor::Walk),
        ..Default::default()
    };
    for k in [
        "stand", "walk", "go_prone", "prone", "crawl", "get_up", "die_1", "die_2",
    ] {
        t.sequences.insert(k.into(), seq(4));
    }
    t
}

/// A crewed 3 × 3 test building with refund override `k` and a build-up
/// sprite (so that it can be sold).
fn building(k: i32) -> BuildingType {
    let mut b = BuildingType {
        strength: Some(500),
        foundation: Some([3, 3]),
        crewed: Some(true),
        ..Default::default()
    };
    b.econ.soylent = Some(k);
    b.econ.buildup = Some(true);
    b.econ.buildup_frames = Some(10);
    b
}

/// The infantry types and their strengths.
const INFANTRY: [(&str, i32); 6] = [
    ("TECH", 60),
    ("ACREW", 80),
    ("SCREW", 90),
    ("TCREW", 70),
    ("ENGI", 75),
    ("SOLDIER", 100),
];

/// The ruleset of these tests: crew types TECH (technician), ACREW, SCREW,
/// TCREW (the crews of sides 0, 1 and 2) and ENGI (engineer, with the
/// engineer flag); SOLDIER; crewed 3 × 3 buildings BOX, YARD (produces
/// buildings), GUNBOX (primary weapon Gun), GAR (occupiable, its occupants
/// may fire), all of refund 250, and PLAIN (no crewed flag); crewed
/// vehicles VAN (unarmed) and GUNVAN (Gun). Sides First, Second, Third and
/// Fourth: Alphaland is of First, Betaland of Second. Divisors 100.
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let push = |r: &mut Ruleset, reg: &str, n: &str| {
        r.registries.entry(reg.into()).or_default().push(n.into());
    };
    for (n, _) in INFANTRY {
        push(&mut r, "infantry", n);
    }
    for n in ["BOX", "YARD", "GUNBOX", "GAR", "PLAIN"] {
        push(&mut r, "buildings", n);
    }
    for n in ["VAN", "GUNVAN"] {
        push(&mut r, "vehicles", n);
    }
    r.sides = ["First", "Second", "Third", "Fourth"]
        .iter()
        .map(|n| Side {
            name: (*n).into(),
            members: vec![],
        })
        .collect();
    let s = &mut r.sim;
    s.countries.get_mut("Alphaland").unwrap().econ.side = Some("First".into());
    s.countries.get_mut("Betaland").unwrap().econ.side = Some("Second".into());
    for (n, st) in INFANTRY {
        s.infantry.insert(n.into(), soldier(st));
    }
    s.infantry.get_mut("ENGI").unwrap().engineer = Some(true);
    s.buildings.insert("BOX".into(), building(250));
    s.buildings.insert(
        "YARD".into(),
        BuildingType {
            factory: Some("building".into()),
            ..building(250)
        },
    );
    s.buildings.insert(
        "GUNBOX".into(),
        BuildingType {
            primary: Some("Gun".into()),
            ..building(250)
        },
    );
    s.buildings.insert(
        "GAR".into(),
        BuildingType {
            can_be_occupied: Some(true),
            can_occupy_fire: Some(true),
            max_number_occupants: Some(2),
            ..building(250)
        },
    );
    s.buildings.insert(
        "PLAIN".into(),
        BuildingType {
            crewed: None,
            ..building(250)
        },
    );
    let van = UnitType {
        strength: Some(200),
        speed: Some(5),
        rot: Some(5),
        cost: Some(600),
        locomotion_class: Some(LocomotionClass::Wheel),
        locomotor: Some(Locomotor::Drive),
        crewed: Some(true),
        ..Default::default()
    };
    s.vehicles.insert(
        "GUNVAN".into(),
        UnitType {
            primary: Some("Gun".into()),
            ..van.clone()
        },
    );
    s.vehicles.insert("VAN".into(), van);
    let g = &mut s.general;
    g.technician = Some("TECH".into());
    g.allied_crew = Some("ACREW".into());
    g.soviet_crew = Some("SCREW".into());
    g.third_crew = Some("TCREW".into());
    g.engineer = Some("ENGI".into());
    g.allied_survivor_divisor = Some(100);
    g.soviet_survivor_divisor = Some(100);
    g.third_survivor_divisor = Some(100);
    g.crew_escape = Some(glasnost_rules::sim::Decimal("1".into()));
    r
}

/// Gives Alphaland (house 0's country) side `name`, or no side.
fn alpha_side(r: &mut Ruleset, name: Option<&str>) {
    r.sim.countries.get_mut("Alphaland").unwrap().econ.side = name.map(Into::into);
}

fn game(r: &Ruleset, seed: u32) -> World {
    testkit::game_with(r, SIZE, seed)
}

fn spawn(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, house, testkit::mid(SIZE, dx, dy), 0)
}

fn half(w: &World) -> bool {
    w.rules().quirks.ranged_draw_half_span
}

/// A lethal hit of warhead SA that does not ignore defences.
fn killing() -> Hit {
    let sa = rules().sim.warheads.keys().position(|k| k == "SA").unwrap() as u32;
    Hit {
        amount: 100_000,
        warhead: Some(sa),
        distance: 0,
        attacker: None,
        house: None,
        ignore_defences: false,
        no_escape: false,
    }
}

fn strength(name: &str) -> i32 {
    INFANTRY.iter().find(|(n, _)| *n == name).unwrap().1
}

fn name_of(w: &World, id: ObjectId) -> String {
    let o = w.obj(id).unwrap();
    w.rules().unit(o.kind, o.type_index).unwrap().name.clone()
}

/// The infantry units created after object `after`, in creation order.
fn new_infantry(w: &World, after: ObjectId) -> Vec<ObjectId> {
    w.objects
        .values()
        .filter(|o| o.kind == Kind::Infantry && o.id > after)
        .map(|o| o.id)
        .collect()
}

/// The draws of each survivor's extra locomotion (S-0208 claim 6, r46: an
/// accepted scatter whose destination comes from the nearby-cell search
/// gives the standing survivor one more locomotion at once, inside its
/// request, so its draws, claim 3's position choice or a failed search's,
/// follow the request's own draw). Read from the movement trace switched
/// on for the run; the infantry created after `after`, in creation order,
/// 0 for one with no extra locomotion.
fn extra_draws(w: &World, after: ObjectId) -> std::vec::IntoIter<u64> {
    let ev = trace::take(w);
    new_infantry(w, after)
        .into_iter()
        .map(|s| {
            ev.iter()
                .filter(|e| e.unit == s)
                .map(|e| match e.what {
                    trace::What::ExtraLocomotion { draws, .. } => draws,
                    _ => 0,
                })
                .sum()
        })
        .collect::<Vec<u64>>()
        .into_iter()
}

/// Takes `n` draws for an extra locomotion.
fn skip_n(rr: &mut Rng, n: u64) {
    for _ in 0..n {
        rr.raw();
    }
}

/// Claim 19 (a)'s draws for a 3 × 3 building: two discarded draws over 0
/// to 1 (`quirks.building_smudge_discarded_draws`), then the percent draw.
fn smudge_a(w: &World, rr: &mut Rng) {
    let h = half(w);
    if w.rules().quirks.building_smudge_discarded_draws {
        rr.ranged(0, 1, h);
        rr.ranged(0, 1, h);
    }
    rr.percent();
}

/// One expected survivor: its cell, type name and health.
type Expected = (CellXY, String, i32);

/// Replays one survivor round of claims 18 and 19 (g) over the cells of a
/// 3 × 3 foundation at `tl` from cell `skip` on, with count `c`, bound `g`,
/// the type draws of `ty`, cells that cannot take a survivor in `full`
/// and cells not clear for movement in `unclear`.
#[allow(clippy::too_many_arguments)]
fn replay_round(
    w: &World,
    rr: &mut Rng,
    tl: CellXY,
    skip: usize,
    c: i32,
    g: i32,
    over: bool,
    ty: &mut dyn FnMut(&mut Rng) -> Option<&'static str>,
    extra: &mut dyn Iterator<Item = u64>,
    full: &[CellXY],
    unclear: &[CellXY],
) -> Vec<Expected> {
    let h = half(w);
    let mut out = Vec::new();
    let cells = damage::foundation_order(tl, (3, 3));
    for &cell in cells.iter().skip(skip) {
        if (out.len() as i32) < c
            && !over
            && rr.ranged(0, g, h) == 1
            && let Some(name) = ty(rr)
        {
            let _creation = rr.raw();
            // spec: S-0312 claim 18 (revision 51: the centre class's R(0, 3) row draw of S-0208 claim 3; none of these cells is refused by the table's first row)
            let _row = rr.ranged(0, 3, h);
            if !full.contains(&cell) {
                let hp = rr.ranged(5, strength(name), h);
                let _scatter = rr.ranged(0, 4, h);
                // spec: S-0208 claim 6 (r46: the extra locomotion inside the scatter request, after its draw)
                skip_n(rr, extra.next().unwrap_or(0));
                out.push((cell, name.to_string(), hp));
            }
        }
        if !unclear.contains(&cell) {
            rr.percent();
            rr.raw();
        }
    }
    out
}

/// Kills building `id` and checks the draws and survivors against the
/// replay: the round from `skip` with the given count, bound and types.
#[allow(clippy::too_many_arguments)]
fn kill_and_check(
    w: &mut World,
    id: ObjectId,
    skip_draws: &mut dyn FnMut(&World, &mut Rng),
    skip: usize,
    c: i32,
    g: i32,
    ty: &mut dyn FnMut(&mut Rng) -> Option<&'static str>,
    full: &[CellXY],
    unclear: &[CellXY],
) -> Vec<ObjectId> {
    let tl = w.obj(id).unwrap().pos.cell();
    let after = w.objects.keys().copied().max().unwrap();
    let pre = w.clone();
    trace::switch_on(w);
    assert_eq!(damage::apply(w, id, killing()), HitResult::NowDead);
    let mut extra = extra_draws(w, after);
    let mut rr = pre.rng.clone();
    smudge_a(&pre, &mut rr);
    skip_draws(&pre, &mut rr);
    let want = replay_round(
        &pre, &mut rr, tl, skip, c, g, false, ty, &mut extra, full, unclear,
    );
    assert_eq!(w.rng, rr, "draws");
    let got = new_infantry(w, after);
    let got_v: Vec<Expected> = got
        .iter()
        .filter(|&&s| w.obj(s).is_some_and(|o| o.on_map))
        .map(|&s| {
            let o = w.obj(s).unwrap();
            (o.pos.cell(), name_of(w, s), o.health)
        })
        .collect();
    let want_tail: Vec<Expected> = want;
    assert_eq!(&got_v[got_v.len() - want_tail.len()..], &want_tail[..]);
    got.into_iter()
        .rev()
        .take(want_tail.len())
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}

fn no_skip(_: &World, _: &mut Rng) {}

// ---------------------------------------------------------------- F5

/// C of building `name` of house `house` with refund `k`, the changed
/// owner mark `captured`, through F5 on the building in play.
fn count_of(r: &Ruleset, name: &str, house: u8, k: i32, captured: bool) -> i32 {
    let mut r = r.clone();
    r.sim.buildings.get_mut(name).unwrap().econ.soylent = Some(k);
    let mut w = game(&r, 11);
    let id = spawn(&mut w, Kind::Building, name, house, 0, 0);
    w.obj_mut(id).unwrap().combat.captured = captured;
    let refund = glasnost_sim::seams::building_refund(&w, id);
    // spec: S-0312 F5 (K: the type's refund for its owner as it stands, S-0408 claim 1; the refund override gives K = v)
    assert_eq!(refund, Some(k));
    let src = survivors::source(&w, id).unwrap();
    survivors::count(&w, &src, refund)
}

// spec: S-0312 F5 (C = min(max(trunc(K / q), 1), 5): quotients below 1, between 1 and 5 and above 5 give 1, the truncated quotient and 5; K ≤ 0 gives 1; scenario D13 variant)
#[test]
fn f5_count_bounds() {
    let r = rules();
    for (k, c) in [
        (50, 1),
        (99, 1),
        (100, 1),
        (299, 2),
        (500, 5),
        (900, 5),
        (0, 1),
        (-300, 1),
    ] {
        assert_eq!(count_of(&r, "BOX", 0, k, false), c, "K = {k}");
    }
}

// spec: S-0312 F5 (q is the divisor of the owner's side, doubled when the building has ever changed owner, the captured mark of S-0314 Terms)
#[test]
fn f5_side_divisor_and_changed_owner() {
    let mut r = rules();
    r.sim.general.allied_survivor_divisor = Some(50);
    r.sim.general.soviet_survivor_divisor = Some(125);
    // House 0 is of side 0 (q = 50), house 1 of side 1 (q = 125).
    assert_eq!(count_of(&r, "BOX", 0, 200, false), 4);
    assert_eq!(count_of(&r, "BOX", 1, 200, false), 1);
    assert_eq!(count_of(&r, "BOX", 1, 500, false), 4);
    // Doubled q: 200 / 100 = 2, 500 / 250 = 2.
    assert_eq!(count_of(&r, "BOX", 0, 200, true), 2);
    assert_eq!(count_of(&r, "BOX", 1, 500, true), 2);
    // Side 2 (Third).
    alpha_side(&mut r, Some("Third"));
    r.sim.general.third_survivor_divisor = Some(60);
    assert_eq!(count_of(&r, "BOX", 0, 200, false), 3);
}

// spec: S-0312 F5 (C = 0 when the type lacks the crewed flag, the owner's side is not 0, 1 or 2, the owner's country has no side, or q = 0)
#[test]
fn f5_zero_cases() {
    let r = rules();
    assert_eq!(count_of(&r, "PLAIN", 0, 250, false), 0);
    let mut r4 = rules();
    alpha_side(&mut r4, Some("Fourth"));
    assert_eq!(count_of(&r4, "BOX", 0, 250, false), 0);
    let mut none = rules();
    alpha_side(&mut none, None);
    assert_eq!(count_of(&none, "BOX", 0, 250, false), 0);
    let mut q0 = rules();
    q0.sim.general.allied_survivor_divisor = Some(0);
    assert_eq!(count_of(&q0, "BOX", 0, 250, false), 0);
    // With the side's own divisor 0 the doubled q is 0 as well.
    q0.sim.general.allied_survivor_divisor = Some(0);
    assert_eq!(count_of(&q0, "BOX", 0, 250, true), 0);
}

// spec: S-0312 claim 18 and claim 19 (g) (C = 0: no survivor and no per-cell smudge step; only claim 19 (a)'s draws are taken) for each case of F5's C = 0, through the kill
#[test]
fn claim_18_no_round_when_c_is_zero() {
    let mut cases: Vec<(Ruleset, &str)> = Vec::new();
    cases.push((rules(), "PLAIN"));
    let mut r = rules();
    alpha_side(&mut r, Some("Fourth"));
    cases.push((r, "BOX"));
    let mut r = rules();
    r.sim.general.allied_survivor_divisor = Some(0);
    cases.push((r, "BOX"));
    for (r, name) in cases {
        let mut w = game(&r, 11);
        let id = spawn(&mut w, Kind::Building, name, 0, 0, 0);
        let after = w.objects.keys().copied().max().unwrap();
        let mut rr = w.rng.clone();
        smudge_a(&w, &mut rr);
        assert_eq!(damage::apply(&mut w, id, killing()), HitResult::NowDead);
        assert_eq!(w.rng, rr, "{name}");
        assert!(new_infantry(&w, after).is_empty());
    }
}

// spec: S-0312 claim 18 and F5 (killing damage that ignores defences leaves the building without crew: no survivors and no smudge steps; claim 3's delayed-kill expiry hit is such a hit)
#[test]
fn claim_18_ignoring_defences_no_crew() {
    let r = rules();
    let mut w = game(&r, 11);
    let id = spawn(&mut w, Kind::Building, "BOX", 0, 0, 0);
    let after = w.objects.keys().copied().max().unwrap();
    let mut rr = w.rng.clone();
    smudge_a(&w, &mut rr);
    let hit = Hit {
        ignore_defences: true,
        ..killing()
    };
    assert_eq!(damage::apply(&mut w, id, hit), HitResult::NowDead);
    assert_eq!(w.rng, rr);
    assert!(new_infantry(&w, after).is_empty());
}

// ---------------------------------------------------------------- claim 18, 19 (g)

// spec: S-0312 claim 18 (per cell in S-0302's order: R(0, 2) while fewer than C are released; a result of 1 releases one; a building that has never changed owner and does not produce buildings takes one percent draw that changes nothing, then F4 (side 0, unarmed: the side's crew, no draw); the creation draw; health R(5, S); the scatter request of claim 12) and claim 19 (g) (then each cell's smudge step: a percent draw and the nearby point's raw value, also after C survivors are out); claim 25 (move for a player house)
#[test]
fn claim_18_round_draws_and_cell_order() {
    for (k, c) in [(100, 1), (250, 2), (900, 5)] {
        for seed in 1..=12 {
            let mut r = rules();
            r.sim.buildings.get_mut("BOX").unwrap().econ.soylent = Some(k);
            let mut w = game(&r, seed);
            let id = spawn(&mut w, Kind::Building, "BOX", 0, 0, 0);
            let mut ty = |rr: &mut Rng| {
                rr.percent();
                Some("ACREW")
            };
            let got = kill_and_check(&mut w, id, &mut no_skip, 0, c, 2, &mut ty, &[], &[]);
            assert!(got.len() as i32 <= c);
            for s in got {
                let o = w.obj(s).unwrap();
                assert_eq!(o.owner, Some(HouseId(0)));
                assert_eq!(o.beh.current, Some(Behaviour::Move), "seed {seed}");
            }
        }
    }
}

// spec: S-0312 claim 18 (g = 1 with a charge planter, plus 6 when the building has ever changed owner; a changed owner takes no percent draw p, only F4's)
#[test]
fn claim_18_chance_bound() {
    for (planter, captured) in [(true, false), (false, true), (true, true)] {
        for seed in 1..=8 {
            let r = rules();
            let mut w = game(&r, seed);
            let id = spawn(&mut w, Kind::Building, "BOX", 0, 0, 0);
            if planter {
                let p = spawn(&mut w, Kind::Infantry, "SOLDIER", 1, 6, 6);
                w.obj_mut(id).unwrap().combat.charge_planter = Some(p);
            }
            w.obj_mut(id).unwrap().combat.captured = captured;
            // K = 250, q = 100 (200 when captured).
            let c = if captured { 1 } else { 2 };
            let g = if planter { 1 } else { 2 } + if captured { 6 } else { 0 };
            let mut ty = move |rr: &mut Rng| {
                if !captured {
                    rr.percent();
                }
                Some("ACREW")
            };
            kill_and_check(&mut w, id, &mut no_skip, 0, c, g, &mut ty, &[], &[]);
        }
    }
}

/// F4 for side 0 and an armed object: a percent draw, below 15 the
/// technician, otherwise the side crew.
fn f4_armed_side0(rr: &mut Rng) -> &'static str {
    if rr.percent() < 15 { "TECH" } else { "ACREW" }
}

// spec: S-0312 claim 18 (a building that produces buildings and has never changed owner: one percent draw p, an engineer when p < 25, otherwise F4's type with F4's own draw when it takes one) and F4 (armed: the primary weapon of claim 6's second choice)
#[test]
fn claim_18_engineer_row() {
    let mut r = rules();
    let yard = r.sim.buildings.get_mut("YARD").unwrap();
    yard.primary = Some("Gun".into());
    yard.econ.soylent = Some(900);
    let mut seen = (0, 0, 0);
    for seed in 1..=24 {
        let mut w = game(&r, seed);
        let id = spawn(&mut w, Kind::Building, "YARD", 0, 0, 0);
        let mut ty = |rr: &mut Rng| {
            if rr.percent() < 25 {
                Some("ENGI")
            } else {
                Some(f4_armed_side0(rr))
            }
        };
        for s in kill_and_check(&mut w, id, &mut no_skip, 0, 5, 2, &mut ty, &[], &[]) {
            match name_of(&w, s).as_str() {
                "ENGI" => seen.0 += 1,
                "TECH" => seen.1 += 1,
                _ => seen.2 += 1,
            }
        }
    }
    assert!(seen.0 > 0 && seen.1 > 0 && seen.2 > 0, "{seen:?}");
}

// spec: S-0312 claim 18 (a changed owner: F4's draw only, no engineer); F4 (side 0, armed by the occupants' fire: S-0312 Terms, a building infantry can occupy whose occupants can fire from it)
#[test]
fn claim_18_changed_owner_and_garrison_armed() {
    for seed in 1..=8 {
        let mut r = rules();
        r.sim.buildings.get_mut("GAR").unwrap().econ.soylent = Some(900);
        r.sim.buildings.get_mut("GAR").unwrap().factory = Some("building".into());
        let mut w = game(&r, seed);
        let id = spawn(&mut w, Kind::Building, "GAR", 0, 0, 0);
        w.obj_mut(id).unwrap().combat.captured = true;
        assert!(survivors::armed(&w, id));
        // q doubled: 900 / 200 = 4.
        let mut ty = |rr: &mut Rng| Some(f4_armed_side0(rr));
        kill_and_check(&mut w, id, &mut no_skip, 0, 4, 8, &mut ty, &[], &[]);
    }
}

// spec: S-0312 claim 18 (no chance draw once the owner's game is over, S-0414's defeated state; the smudge steps go on)
#[test]
fn claim_18_owner_defeated() {
    let r = rules();
    let mut w = game(&r, 5);
    let id = spawn(&mut w, Kind::Building, "BOX", 0, 0, 0);
    w.houses[0].econ.result.defeated = true;
    let tl = w.obj(id).unwrap().pos.cell();
    let after = w.objects.keys().copied().max().unwrap();
    let mut rr = w.rng.clone();
    smudge_a(&w, &mut rr);
    let mut ty = |_: &mut Rng| Some("ACREW");
    let none = &mut std::iter::empty();
    let want = replay_round(&w, &mut rr, tl, 0, 2, 2, true, &mut ty, none, &[], &[]);
    assert!(want.is_empty());
    damage::apply(&mut w, id, killing());
    assert_eq!(w.rng, rr);
    assert!(new_infantry(&w, after).is_empty());
}

// spec: S-0312 claim 18 (a survivor for which no standing position is free takes its creation draw, never enters play, takes no health draw and does not count against C) and claim 19 (g) (revision 51: a foundation cell that is not clear for movement takes no smudge step; a full cell is not refused by the first row of S-0208's table O, so it takes the row draw first)
#[test]
fn claim_18_full_cell_and_unclear_cell() {
    for seed in 1..=16 {
        let r = rules();
        let mut w = game(&r, seed);
        let id = spawn(&mut w, Kind::Building, "BOX", 0, 0, 0);
        let tl = w.obj(id).unwrap().pos.cell();
        let cells = damage::foundation_order(tl, (3, 3));
        // The five standing positions of the first and the fifth cell are
        // taken; the fifth cell is also water (so that no survivor stands
        // in water, which would change its scatter request's draws).
        let full = [cells[0], cells[4]];
        let wet = cells[4];
        let soldier = w.rules().index_of(Kind::Infantry, "SOLDIER").unwrap();
        let filler =
            glasnost_sim::objects::create(&mut w, Kind::Infantry, soldier, Some(HouseId(1)));
        for c in full {
            for p in 0..5 {
                w.cells
                    .mark_infantry(c, glasnost_sim::world::Layer::Ground, p, filler, HouseId(1));
            }
        }
        w.cells.grid.get_mut(wet).unwrap().land = land::WATER;
        let mut ty = |rr: &mut Rng| {
            rr.percent();
            Some("ACREW")
        };
        let got = kill_and_check(&mut w, id, &mut no_skip, 0, 2, 2, &mut ty, &full, &[wet]);
        assert!(
            got.iter()
                .all(|&s| !full.contains(&w.obj(s).unwrap().pos.cell()))
        );
    }
}

// spec: S-0312 claim 18 (a survivor is put at the position the table chose with no entry test, so no occupant or cell outcome refuses it there; the absence of an entry test when the survivor's cell holds another house's infantry) with S-0209 claim 4 (rows WR2 and WR3: another house's infantry in the record refuses an infantry mover's entry) (M8 seam sweep 5a)
#[test]
fn claim_18_no_entry_test_against_another_houses_infantry() {
    use glasnost_sim::cells::position;
    use glasnost_sim::occupancy::{self, Outcome};
    let ground = glasnost_sim::world::Layer::Ground;
    let mut placed = 0;
    for seed in 1..=16 {
        let r = rules();
        let mut w = game(&r, seed);
        let id = spawn(&mut w, Kind::Building, "BOX", 0, 0, 0);
        let tl = w.obj(id).unwrap().pos.cell();
        let cells = damage::foundation_order(tl, (3, 3));
        // The right position of every foundation cell, and of a cell outside
        // it, taken by Beta's infantry, so that every row of S-0208's table O
        // for the centre class still has a free side position.
        let outside = testkit::mid(SIZE, 6, 6);
        let soldier = w.rules().index_of(Kind::Infantry, "SOLDIER").unwrap();
        let filler =
            glasnost_sim::objects::create(&mut w, Kind::Infantry, soldier, Some(HouseId(1)));
        for &c in cells.iter().chain([&outside]) {
            w.cells
                .mark_infantry(c, ground, position::RIGHT, filler, HouseId(1));
        }
        // The entry test an Alpha infantry unit meets there: not clear.
        let probe = spawn(&mut w, Kind::Infantry, "SOLDIER", 0, -6, -6);
        let level = w.cells.cell(outside).map_or(0, |x| i32::from(x.level));
        assert_ne!(
            occupancy::infantry_outcome(&w, probe, outside, level),
            Outcome::Clear
        );
        glasnost_sim::objects::remove(&mut w, probe);
        let mut ty = |rr: &mut Rng| {
            rr.percent();
            Some("ACREW")
        };
        let got = kill_and_check(&mut w, id, &mut no_skip, 0, 2, 2, &mut ty, &[], &[]);
        for &s in &got {
            let o = w.obj(s).unwrap();
            assert!(cells.contains(&o.pos.cell()), "in a foundation cell");
            assert_eq!(o.owner, Some(HouseId(0)));
            placed += 1;
        }
    }
    assert!(placed > 0, "some runs put a survivor down");
}

// spec: S-0312 Edge cases (Departure: missing survivor type: no survivor in that cell, no creation or health draw)
#[test]
fn departure_missing_survivor_type() {
    for seed in 1..=8 {
        let mut r = rules();
        r.sim.general.allied_crew = None;
        let mut w = game(&r, seed);
        let id = spawn(&mut w, Kind::Building, "BOX", 0, 0, 0);
        let mut ty = |rr: &mut Rng| {
            rr.percent();
            None
        };
        assert!(kill_and_check(&mut w, id, &mut no_skip, 0, 2, 2, &mut ty, &[], &[]).is_empty());
    }
}

// spec: S-0312 claim 18 (with a charge planter in the game whose owner the building's owner is not allied with, the survivor is ordered to attack it; an allied planter gives the ordinary order)
#[test]
fn claim_18_attack_order_against_the_planter() {
    for allied in [false, true] {
        let mut found = false;
        for seed in 1..=12 {
            let r = rules();
            let mut setup = testkit::setup(seed);
            if allied {
                setup.alliances = vec![(0, 1)];
            }
            let mut w = glasnost_sim::load::new_game(
                std::sync::Arc::new(glasnost_sim::rules::SimRules::new(&r, "s0312-survivors")),
                &testkit::empty_map(SIZE),
                testkit::catalogue(),
                &setup,
            )
            .unwrap();
            let id = spawn(&mut w, Kind::Building, "BOX", 0, 0, 0);
            let p = spawn(&mut w, Kind::Infantry, "SOLDIER", 1, 6, 6);
            w.obj_mut(id).unwrap().combat.charge_planter = Some(p);
            assert_eq!(w.allied(Some(HouseId(0)), Some(HouseId(1))), allied);
            let mut ty = |rr: &mut Rng| {
                rr.percent();
                Some("ACREW")
            };
            for s in kill_and_check(&mut w, id, &mut no_skip, 0, 2, 1, &mut ty, &[], &[]) {
                found = true;
                let o = w.obj(s).unwrap();
                if allied {
                    assert_eq!(o.beh.current, Some(Behaviour::Move));
                } else {
                    assert_eq!(o.beh.current, Some(Behaviour::Attack));
                    assert_eq!(o.target, Some(Target::Object(p)));
                }
            }
        }
        assert!(found);
    }
}

// spec: S-0312 claim 25 (a survivor without an attack order: move for a player house, human-controlled or an AI house; hunt for the neutral or special house; glossary *AI house*, *Test setups*: a test-only computer-controlled player house keeps the original's hunt)
#[test]
fn claim_25_survivor_orders_by_owner() {
    for (house, human, want) in [
        (0u8, true, Behaviour::Move),
        (0, false, Behaviour::Hunt),
        (2, false, Behaviour::Hunt),
        (3, false, Behaviour::Hunt),
    ] {
        let mut found = false;
        for seed in 1..=12 {
            let r = rules();
            let mut w = testkit::game_with_non_players(&r, SIZE, seed);
            if house == 0 && !human {
                // A test-only computer-controlled player house (glossary
                // *AI house*, *Test setups*); an AI house of our engine
                // is human-controlled in the simulation.
                w.houses[0].human = false;
            }
            let id = spawn(&mut w, Kind::Building, "BOX", house, 0, 0);
            // The special house's country is of side 1, the others of side 0.
            let crew = if house == 3 { "SCREW" } else { "ACREW" };
            let mut ty = |rr: &mut Rng| {
                rr.percent();
                Some(crew)
            };
            for s in kill_and_check(&mut w, id, &mut no_skip, 0, 2, 2, &mut ty, &[], &[]) {
                found = true;
                assert_eq!(w.obj(s).unwrap().beh.current, Some(want), "house {house}");
            }
        }
        assert!(found, "house {house}");
    }
}

// spec: S-0312 claim 24 and claim 19 (g) (absorbed passengers come out first, one per cell in the cell order whether placed or not; the survivors and the smudge steps start after the last cell used)
#[test]
fn claim_24_survivors_start_after_absorbed_passengers() {
    for seed in 1..=8 {
        let r = rules();
        let mut w = game(&r, seed);
        let id = spawn(&mut w, Kind::Building, "BOX", 0, 0, 0);
        let tl = w.obj(id).unwrap().pos.cell();
        let cells = damage::foundation_order(tl, (3, 3));
        // Two passengers inside; the first cell cannot take one (a vehicle
        // stands there), so the first passenger dies, and the second
        // comes out in the second cell.
        let mut list = Vec::new();
        for _ in 0..2 {
            let p = spawn(&mut w, Kind::Infantry, "SOLDIER", 0, 8, 8);
            glasnost_sim::objects::take_inside(&mut w, p);
            w.obj_mut(p).unwrap().structure.absorbed_in = Some(id);
            list.push(p);
        }
        w.obj_mut(id).unwrap().structure.absorbed = list.clone();
        w.cells
            .claim_vehicle(cells[0], glasnost_sim::world::Layer::Ground, true);
        let h = half(&w);
        let mut passengers = |_: &World, rr: &mut Rng| {
            // Cell 1: refused by the vehicle-present flag, no draw. Cell 2:
            // R(0, 3) for the row, then the scatter request.
            rr.ranged(0, 3, h);
            rr.ranged(0, 4, h);
        };
        let mut ty = |rr: &mut Rng| {
            rr.percent();
            Some("ACREW")
        };
        kill_and_check(&mut w, id, &mut passengers, 2, 2, 2, &mut ty, &[], &[]);
        assert!(w.obj(list[0]).is_none_or(|o| o.health <= 0 || !o.on_map));
        assert_eq!(w.obj(list[1]).unwrap().pos.cell(), cells[1]);
    }
}

// ---------------------------------------------------------------- claim 21

// spec: S-0312 claim 21 and Edge cases (quirks.building_removal_second_round on: an exploding building awaiting removal takes, after it has left play, a second round of claims 18 and 19 (g) from the first cell, C afresh by F5; with killing damage that ignored defences C = 0 and the round takes no draw)
#[test]
fn claim_21_second_round() {
    for ignore in [false, true] {
        for seed in 1..=8 {
            let mut r = rules();
            r.sim.buildings.get_mut("BOX").unwrap().combat.explodes = Some(true);
            let mut w = game(&r, seed);
            let id = spawn(&mut w, Kind::Building, "BOX", 0, 0, 0);
            let tl = w.obj(id).unwrap().pos.cell();
            let hit = Hit {
                ignore_defences: ignore,
                ..killing()
            };
            assert_eq!(damage::apply(&mut w, id, hit), HitResult::NowDead);
            assert!(w.obj(id).is_some(), "awaits removal");
            let after = w.objects.keys().copied().max().unwrap();
            let pre = w.clone();
            trace::switch_on(&mut w);
            death::removal_turn(&mut w, id);
            assert!(w.obj(id).is_none(), "left play");
            let mut extra = extra_draws(&w, after);
            let mut rr = pre.rng.clone();
            let mut ty = |rr: &mut Rng| {
                rr.percent();
                Some("ACREW")
            };
            let c = if ignore { 0 } else { 2 };
            let want = if c == 0 {
                Vec::new()
            } else {
                replay_round(
                    &pre,
                    &mut rr,
                    tl,
                    0,
                    c,
                    2,
                    false,
                    &mut ty,
                    &mut extra,
                    &[],
                    &[],
                )
            };
            assert_eq!(w.rng, rr, "seed {seed}");
            let got: Vec<Expected> = new_infantry(&w, after)
                .iter()
                .map(|&s| {
                    let o = w.obj(s).unwrap();
                    (o.pos.cell(), name_of(&w, s), o.health)
                })
                .collect();
            assert_eq!(got, want);
        }
    }
}

// ---------------------------------------------------------------- claim 13, F4

/// Kills vehicle `name` of house 0 (Alphaland of side `side`) and returns
/// the crew's type name, with the draws checked against `ty`.
fn crew_of(name: &str, side: Option<&str>, ty: &mut dyn FnMut(&mut Rng) -> &'static str) {
    for seed in 1..=8 {
        let mut r = rules();
        alpha_side(&mut r, side);
        let mut w = game(&r, seed);
        let id = spawn(&mut w, Kind::Vehicle, name, 0, 0, 0);
        let after = w.objects.keys().copied().max().unwrap();
        let mode = w.rules().mode;
        let h = half(&w);
        let mut rr = w.rng.clone();
        rr.unit(mode);
        let want = ty(&mut rr);
        rr.raw();
        rr.ranged(0, 3, h);
        let hp = rr.ranged(5, strength(want) / 2, h);
        rr.ranged(0, 4, h);
        damage::apply(&mut w, id, killing());
        assert_eq!(w.rng, rr, "{name} {side:?} seed {seed}");
        let got = new_infantry(&w, after);
        assert_eq!(got.len(), 1);
        assert_eq!(name_of(&w, got[0]), want);
        assert_eq!(w.obj(got[0]).unwrap().health, hp);
    }
}

// spec: S-0312 claim 13 and F4 (the crew type: no side, the technician with no draw; side 0, 1 or 2 unarmed, that side's crew with no draw; armed, a percent draw below 15 the technician, otherwise the side crew; another side, the technician, armed with a discarded percent draw); the draw order chance, type, creation, position (revision 51: R(0, 3) for the centre class), health
#[test]
fn claim_13_f4_crew_types() {
    crew_of("VAN", None, &mut |_| "TECH");
    crew_of("GUNVAN", None, &mut |_| "TECH");
    crew_of("VAN", Some("First"), &mut |_| "ACREW");
    crew_of("VAN", Some("Second"), &mut |_| "SCREW");
    crew_of("VAN", Some("Third"), &mut |_| "TCREW");
    crew_of("GUNVAN", Some("First"), &mut f4_armed_side0);
    crew_of("GUNVAN", Some("Third"), &mut |rr| {
        if rr.percent() < 15 { "TECH" } else { "TCREW" }
    });
    crew_of("VAN", Some("Fourth"), &mut |_| "TECH");
    crew_of("GUNVAN", Some("Fourth"), &mut |rr| {
        rr.percent();
        "TECH"
    });
}

// ---------------------------------------------------------------- S-0408 claim 7

/// Replays the sale survivors of a 3 × 3 building: `c` attempts, each its
/// type draws (`ty`, which returns the type and whether it is an
/// engineer; repeated while it gives a second engineer), the creation
/// draw, R(0, 8) for the cell and R(0, 3) for the row (an empty cell), and
/// the scatter request's draw. Returns the types in order.
fn replay_sale(
    w: &World,
    rr: &mut Rng,
    c: i32,
    ty: &mut dyn FnMut(&mut Rng) -> Option<&'static str>,
    extra: &mut dyn Iterator<Item = u64>,
) -> Vec<&'static str> {
    let h = half(w);
    let mut out = Vec::new();
    let mut engineer = false;
    for _ in 0..c {
        let mut t = None;
        for _ in 0..100 {
            let x = ty(rr);
            if x == Some("ENGI") && engineer {
                continue;
            }
            engineer |= x == Some("ENGI");
            t = Some(x);
            break;
        }
        let Some(Some(t)) = t else { continue };
        rr.raw();
        rr.ranged(0, 8, h);
        rr.ranged(0, 3, h);
        rr.ranged(0, 4, h);
        // spec: S-0208 claim 6 (r46: the extra locomotion inside the scatter request, after its draw)
        skip_n(rr, extra.next().unwrap_or(0));
        out.push(t);
    }
    out
}

// spec: S-0408 claim 7 (exactly C attempts, no chance draw; per attempt the type choice, the creation draw, R(0, n − 1) for the cell and R(0, 3) for the row; full strength, facing 0, the scatter request, move whoever owns it) and S-0312 F5 (the same C as a destruction of the same building)
#[test]
fn s0408_claim_7_sale_survivors() {
    for (k, c) in [(100, 1), (250, 2), (900, 5)] {
        for seed in 1..=6 {
            let mut r = rules();
            r.sim.buildings.get_mut("BOX").unwrap().econ.soylent = Some(k);
            let mut w = testkit::game_with_non_players(&r, SIZE, seed);
            for house in [0u8, 2] {
                let id = spawn(
                    &mut w,
                    Kind::Building,
                    "BOX",
                    house,
                    0,
                    6 * i32::from(house),
                );
                let src = survivors::source(&w, id).unwrap();
                let refund = glasnost_sim::seams::building_refund(&w, id);
                assert_eq!(survivors::count(&w, &src, refund), c);
                let after = w.objects.keys().copied().max().unwrap();
                let pre = w.clone();
                trace::switch_on(&mut w);
                assert_eq!(survivors::sold(&mut w, id), c);
                let mut extra = extra_draws(&w, after);
                let mut rr = pre.rng.clone();
                let mut ty = |rr: &mut Rng| {
                    rr.percent();
                    Some("ACREW")
                };
                let want = replay_sale(&pre, &mut rr, c, &mut ty, &mut extra);
                assert_eq!(w.rng, rr);
                let got = new_infantry(&w, after);
                assert_eq!(got.len(), want.len());
                for s in got {
                    let o = w.obj(s).unwrap();
                    assert_eq!(o.health, strength("ACREW"));
                    assert_eq!(o.body.value(w.tick), 0);
                    assert_eq!(o.beh.current, Some(Behaviour::Move));
                    let f = damage::foundation_order(w.obj(id).unwrap().pos.cell(), (3, 3));
                    assert!(f.contains(&o.pos.cell()));
                }
            }
        }
    }
}

// spec: S-0408 claim 7 (the type choice of a building that produces buildings: p, an engineer below 25; a second engineer repeats the whole choice with its draws, so a sale releases at most one engineer)
#[test]
fn s0408_claim_7_one_engineer() {
    let mut engineers = 0;
    for seed in 1..=16 {
        let mut r = rules();
        r.sim.buildings.get_mut("YARD").unwrap().econ.soylent = Some(900);
        let mut w = game(&r, seed);
        let id = spawn(&mut w, Kind::Building, "YARD", 0, 0, 0);
        let after = w.objects.keys().copied().max().unwrap();
        let pre = w.clone();
        trace::switch_on(&mut w);
        survivors::sold(&mut w, id);
        let mut extra = extra_draws(&w, after);
        let mut rr = pre.rng.clone();
        let mut ty = |rr: &mut Rng| {
            if rr.percent() < 25 {
                Some("ENGI")
            } else {
                Some("ACREW")
            }
        };
        let want = replay_sale(&pre, &mut rr, 5, &mut ty, &mut extra);
        assert_eq!(w.rng, rr, "seed {seed}");
        let names: Vec<String> = new_infantry(&w, after)
            .iter()
            .map(|&s| name_of(&w, s))
            .collect();
        assert_eq!(names, want);
        let e = names.iter().filter(|n| *n == "ENGI").count();
        assert!(e <= 1);
        engineers += e;
    }
    assert!(engineers > 0);
}

// spec: S-0408 Edge cases (A crew choice that always gives an engineer: at most 100 type choices for one survivor, then none for that attempt, its draws taken)
#[test]
fn s0408_engineer_choice_limit() {
    let mut r = rules();
    r.sim.buildings.get_mut("BOX").unwrap().econ.soylent = Some(200);
    r.sim.general.allied_crew = Some("ENGI".into());
    let mut w = game(&r, 3);
    let id = spawn(&mut w, Kind::Building, "BOX", 0, 0, 0);
    let after = w.objects.keys().copied().max().unwrap();
    let pre = w.clone();
    trace::switch_on(&mut w);
    survivors::sold(&mut w, id);
    let mut extra = extra_draws(&w, after);
    let mut rr = pre.rng.clone();
    let mut ty = |rr: &mut Rng| {
        rr.percent();
        Some("ENGI")
    };
    let want = replay_sale(&pre, &mut rr, 2, &mut ty, &mut extra);
    assert_eq!(want, vec!["ENGI"]);
    assert_eq!(w.rng, rr);
    assert_eq!(new_infantry(&w, after).len(), 1);
}

// spec: S-0408 claims 6, 7 and 9 (the sale's release step puts out the sale survivors, C of S-0312 F5, through the sale order)
#[test]
fn s0408_claim_7_through_the_sale_order() {
    let mut r = rules();
    r.sim.buildings.get_mut("BOX").unwrap().econ.soylent = Some(300);
    let mut w = game(&r, 9);
    testkit::run(&mut w, 4);
    let id = spawn(&mut w, Kind::Building, "BOX", 0, 0, 0);
    let after = w.objects.keys().copied().max().unwrap();
    assert!(glasnost_sim::orders::issue(
        &mut w,
        HouseId(0),
        glasnost_sim::orders::Order::Sell { actor: id }
    ));
    for _ in 0..4 {
        glasnost_sim::tick::step(&mut w);
    }
    let names: Vec<String> = new_infantry(&w, after)
        .iter()
        .map(|&s| name_of(&w, s))
        .collect();
    assert_eq!(names, vec!["ACREW"; 3]);
}

// spec: S-0312 claim 18 and S-0408 claim 7 (a destroyed and a sold building of the same type, owner and refund take the same C; the sale makes exactly C attempts, the destruction releases at most C)
#[test]
fn sale_and_destruction_share_the_count() {
    for k in [100, 250, 400, 900] {
        let mut r = rules();
        r.sim.buildings.get_mut("BOX").unwrap().econ.soylent = Some(k);
        let c = (k / 100).clamp(1, 5);
        let (mut most, mut sold_most) = (0, 0);
        for seed in 1..=24 {
            let mut w = game(&r, seed);
            let id = spawn(&mut w, Kind::Building, "BOX", 0, 0, 0);
            let mut ty = |rr: &mut Rng| {
                rr.percent();
                Some("ACREW")
            };
            let got = kill_and_check(&mut w, id, &mut no_skip, 0, c, 2, &mut ty, &[], &[]);
            most = most.max(got.len() as i32);
            let mut w = game(&r, seed);
            let id = spawn(&mut w, Kind::Building, "BOX", 0, 0, 0);
            // At most C placed: an attempt whose cell row is full is lost.
            let n = survivors::sold(&mut w, id);
            assert!(n <= c);
            sold_most = sold_most.max(n);
        }
        assert!(most <= c);
        assert_eq!(sold_most, c, "K = {k}");
        if c <= 2 {
            assert_eq!(most, c, "K = {k}");
        }
    }
}

// spec: S-0312 claim 18 and Import mapping (revision 51, Q-2135 answered: Crewed absent means no for every kind of type; Crewed set: crewed)
#[test]
fn crewed_absent_on_a_building_is_no() {
    let r = rules();
    let w = game(&r, 1);
    for (name, crewed) in [("PLAIN", false), ("BOX", true)] {
        let ti = w.rules().index_of(Kind::Building, name).unwrap();
        assert_eq!(w.rules().buildings[ti as usize].building.crewed, crewed);
    }
}

// spec: S-0312 F5 (C = 0 when the building has no crew: the no-crew mark, set only by killing damage that ignores defences; revision 51, Q-2131 answered: no map field sets it) and claim 18 and S-0408 claim 7 (no survivors, no smudge steps; a sale releases none)
#[test]
fn f5_no_crew_mark() {
    let r = rules();
    for sale in [false, true] {
        let mut w = game(&r, 4);
        let id = spawn(&mut w, Kind::Building, "BOX", 0, 0, 0);
        w.obj_mut(id).unwrap().combat.no_crew = true;
        let src = survivors::source(&w, id).unwrap();
        let refund = glasnost_sim::seams::building_refund(&w, id);
        assert_eq!(survivors::count(&w, &src, refund), 0);
        let after = w.objects.keys().copied().max().unwrap();
        let mut rr = w.rng.clone();
        if sale {
            assert_eq!(survivors::sold(&mut w, id), 0);
        } else {
            smudge_a(&w, &mut rr);
            assert_eq!(damage::apply(&mut w, id, killing()), HitResult::NowDead);
        }
        assert_eq!(w.rng, rr);
        assert!(new_infantry(&w, after).is_empty());
    }
}

// spec: S-0312 claim 19 (g) (revision 51, Q-2133 answered: a foundation cell is clear for movement unless the object-present flag of its record is set, it holds an overlay type with the wall flag, or the tracked class's multiplier of its land type is exactly 0; infantry positions and the vehicle-present flag are ignored, and a multiplier merely below S-0211's 0.01 does not fail it)
#[test]
fn claim_19g_clear_for_movement() {
    use glasnost_rules::sim::{Decimal, LandRow, LandType, OverlayType};
    let mut r = rules();
    r.registries
        .entry("overlays".into())
        .or_default()
        .push("FENCE".into());
    r.sim.overlays.insert(
        "FENCE".into(),
        OverlayType {
            wall: Some(true),
            ..Default::default()
        },
    );
    let track = |v: &str| LandRow {
        track: Some(Decimal(v.into())),
        ..Default::default()
    };
    r.sim.land_types.insert(LandType::Water, track("0"));
    r.sim.land_types.insert(LandType::Rough, track("0.001"));
    for seed in 1..=8 {
        let mut w = game(&r, seed);
        let id = spawn(&mut w, Kind::Building, "BOX", 0, 0, 0);
        // The owner is defeated, so no survivor chance is drawn and only the
        // smudge steps of claim 19 (g) remain.
        w.houses[0].econ.result.defeated = true;
        let tl = w.obj(id).unwrap().pos.cell();
        let cells = damage::foundation_order(tl, (3, 3));
        let ground = glasnost_sim::world::Layer::Ground;
        // Cell 1: the object-present flag. Cell 2: a wall overlay. Cell 3:
        // a track multiplier of exactly 0. Cell 4: a track multiplier below
        // 0.01 but above 0. Cell 5: infantry positions and the
        // vehicle-present flag, which this test ignores.
        w.cells.occ_mut(cells[1], ground).unwrap().others = 1;
        w.cells.grid.get_mut(cells[2]).unwrap().overlay = 0;
        w.cells.grid.get_mut(cells[3]).unwrap().land = land::WATER;
        w.cells.grid.get_mut(cells[4]).unwrap().land = land::ROUGH;
        let soldier = w.rules().index_of(Kind::Infantry, "SOLDIER").unwrap();
        let filler =
            glasnost_sim::objects::create(&mut w, Kind::Infantry, soldier, Some(HouseId(1)));
        for p in 0..5 {
            w.cells
                .mark_infantry(cells[5], ground, p, filler, HouseId(1));
        }
        w.cells.claim_vehicle(cells[5], ground, true);
        let after = w.objects.keys().copied().max().unwrap();
        let mut rr = w.rng.clone();
        smudge_a(&w, &mut rr);
        let mut ty = |_: &mut Rng| Some("ACREW");
        let unclear = [cells[1], cells[2], cells[3]];
        let none = &mut std::iter::empty();
        let want = replay_round(&w, &mut rr, tl, 0, 2, 2, true, &mut ty, none, &[], &unclear);
        assert!(want.is_empty());
        assert_eq!(damage::apply(&mut w, id, killing()), HitResult::NowDead);
        assert_eq!(w.rng, rr, "seed {seed}");
        assert!(new_infantry(&w, after).is_empty());
    }
}

// spec: S-0312 claim 19 (g) (for a high-bridge cell, S-0211 claim 15, the record read is the bridge-deck record and test 3 is not made; for every other cell it is the ground record; the high-bridge case is rated unmeasured) (M8 seam sweep 5a). The deck is set on the cells directly, as intake 64's deck tests do.
#[test]
fn claim_19g_a_high_bridge_cell_reads_the_deck_record() {
    use glasnost_rules::sim::{Decimal, LandRow, LandType};
    let mut r = rules();
    r.sim.land_types.insert(
        LandType::Water,
        LandRow {
            track: Some(Decimal("0".into())),
            ..Default::default()
        },
    );
    for seed in 1..=8 {
        let mut w = game(&r, seed);
        let id = spawn(&mut w, Kind::Building, "BOX", 0, 0, 0);
        // No survivor chance is drawn: only the smudge steps remain.
        w.houses[0].econ.result.defeated = true;
        let tl = w.obj(id).unwrap().pos.cell();
        let cells = damage::foundation_order(tl, (3, 3));
        let (ground, deck) = (
            glasnost_sim::world::Layer::Ground,
            glasnost_sim::world::Layer::Deck,
        );
        for &c in &cells[1..=4] {
            let i = w.cells.idx(c).unwrap();
            w.cells.deck[i] = true;
        }
        // Cell 1: the deck record's object-present flag fails it. Cell 2:
        // the ground record's flag under the deck does not. Cell 3: water
        // with a track multiplier of exactly 0 under the deck does not (no
        // test 3). Cell 4: a deck cell with nothing set. Cell 5, no deck:
        // the ground record's flag fails it (control).
        w.cells.occ_mut(cells[1], deck).unwrap().others = 1;
        w.cells.occ_mut(cells[2], ground).unwrap().others = 1;
        w.cells.grid.get_mut(cells[3]).unwrap().land = land::WATER;
        w.cells.occ_mut(cells[5], ground).unwrap().others = 1;
        let mut rr = w.rng.clone();
        smudge_a(&w, &mut rr);
        let mut ty = |_: &mut Rng| Some("ACREW");
        let unclear = [cells[1], cells[5]];
        let none = &mut std::iter::empty();
        let want = replay_round(&w, &mut rr, tl, 0, 2, 2, true, &mut ty, none, &[], &unclear);
        assert!(want.is_empty());
        assert_eq!(damage::apply(&mut w, id, killing()), HitResult::NowDead);
        assert_eq!(w.rng, rr, "seed {seed}");
    }
}

// spec: S-0312 claim 24 and D16 variant f (revision 51: the release of absorbed passengers tests only the no-crew state, not the crewed flag: a building type without the crewed flag releases its absorbed passengers as any other does, and no survivors, C = 0 by F5, so no smudge step of claim 19 (g))
#[test]
fn claim_24_release_ignores_the_crewed_flag() {
    for seed in 1..=8 {
        let r = rules();
        let mut w = game(&r, seed);
        let id = spawn(&mut w, Kind::Building, "PLAIN", 0, 0, 0);
        let tl = w.obj(id).unwrap().pos.cell();
        let cells = damage::foundation_order(tl, (3, 3));
        let mut list = Vec::new();
        for _ in 0..2 {
            let p = spawn(&mut w, Kind::Infantry, "SOLDIER", 0, 8, 8);
            glasnost_sim::objects::take_inside(&mut w, p);
            w.obj_mut(p).unwrap().structure.absorbed_in = Some(id);
            list.push(p);
        }
        w.obj_mut(id).unwrap().structure.absorbed = list.clone();
        let after = w.objects.keys().copied().max().unwrap();
        let h = half(&w);
        let mut rr = w.rng.clone();
        smudge_a(&w, &mut rr);
        // Each passenger: R(0, 3) for the row, then its scatter request.
        for _ in 0..2 {
            rr.ranged(0, 3, h);
            rr.ranged(0, 4, h);
        }
        assert_eq!(damage::apply(&mut w, id, killing()), HitResult::NowDead);
        assert_eq!(w.rng, rr, "seed {seed}");
        // The list holds the most recent entry first; it comes out first, in
        // the first cell.
        assert_eq!(w.obj(list[0]).unwrap().pos.cell(), cells[0]);
        assert_eq!(w.obj(list[1]).unwrap().pos.cell(), cells[1]);
        assert!(new_infantry(&w, after).is_empty(), "no survivors");
    }
}
