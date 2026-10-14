//! Map triggers (S-0011) on synthetic maps: tags, polling, timers, latches,
//! repeat types, variables, the owner-house raise and the actions of the
//! subset. The variables (actions 56 and 57) serve as the observable.

use std::sync::Arc;

use glasnost_map::Map;
use glasnost_map::triggers::{Action, Event, Owner, TagDef, TriggerDef, Triggers};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::rules::SimRules;
use glasnost_sim::testkit;
use glasnost_sim::triggers::{self, Carrier, Raise};
use glasnost_sim::world::World;
use glasnost_sim::{save, tick};

fn ev(code: i32, value: i32) -> Event {
    Event {
        code,
        kind: 0,
        value,
        type_name: None,
    }
}

fn act(code: i32, number: i32) -> Action {
    Action {
        code,
        number,
        ..Default::default()
    }
}

fn set(v: i32) -> Action {
    act(56, v)
}

fn clear(v: i32) -> Action {
    act(57, v)
}

fn trig(events: Vec<Event>, actions: Vec<Action>) -> TriggerDef {
    TriggerDef {
        events,
        actions,
        ..TriggerDef::created(b"T".to_vec())
    }
}

fn tag(repeat: i32, chain: Vec<u32>) -> TagDef {
    TagDef {
        id: b"TAG".to_vec(),
        repeat,
        name: Vec::new(),
        first: chain.first().copied(),
        chain,
    }
}

fn map_with(t: Triggers) -> Map {
    let mut m = testkit::empty_map(20);
    m.triggers = t;
    m
}

fn game_rules(t: Triggers, rules: Arc<SimRules>) -> World {
    glasnost_sim::load::new_game(
        rules,
        &map_with(t),
        testkit::catalogue(),
        &testkit::setup(7),
    )
    .expect("synthetic game loads")
}

fn game(t: Triggers) -> World {
    let rules = Arc::new(SimRules::new(&testkit::ruleset(), "synthetic-test"));
    game_rules(t, rules)
}

fn triggers(defs: Vec<TriggerDef>, tags: Vec<TagDef>) -> Triggers {
    Triggers {
        triggers: defs,
        tags,
        ..Default::default()
    }
}

/// One repeating tag over the definitions' first trigger, carried by cell (4, 4).
fn carried(defs: Vec<TriggerDef>) -> Triggers {
    let mut t = triggers(defs, vec![tag(2, vec![0])]);
    t.cell_tags = vec![((4, 4), 0)];
    t
}

fn var(w: &World, v: usize) -> bool {
    w.triggers.variables[v]
}

/// Steps until the variable is set; the tick in which it was set.
fn tick_set(w: &mut World, v: usize, limit: i32) -> Option<i32> {
    while w.tick < limit {
        let t = w.tick;
        tick::step(w);
        if var(w, v) {
            return Some(t);
        }
    }
    None
}

// spec: S-0011 claim 13 (a one-time polled trigger whose only event is elapsed time v fires in tick 15v)
#[test]
fn elapsed_time_fires_in_tick_15v() {
    let mut w = game(triggers(
        vec![trig(vec![ev(13, 4)], vec![set(0)])],
        vec![tag(0, vec![0])],
    ));
    assert_eq!(tick_set(&mut w, 0, 200), Some(60));
}

// spec: S-0011 claim 13 (a nonpositive length has already run out in tick 0)
#[test]
fn nonpositive_elapsed_time_fires_in_tick_0() {
    let mut w = game(triggers(
        vec![trig(vec![ev(13, -3)], vec![set(0)])],
        vec![tag(0, vec![0])],
    ));
    assert_eq!(tick_set(&mut w, 0, 10), Some(0));
}

// spec: S-0011 claim 1 (a trigger that is a member of no chain never fires; two chains hold independent instances)
#[test]
fn only_chained_triggers_run() {
    let defs = vec![
        trig(vec![ev(8, 0)], vec![set(0)]),
        trig(vec![ev(13, 1)], vec![set(1)]),
    ];
    // Tag 0 holds trigger 1; tag 1 holds trigger 1 as well, disabled there by tag 0's... none: both fire.
    let mut w = game(triggers(defs, vec![tag(0, vec![1]), tag(2, vec![1])]));
    for _ in 0..40 {
        tick::step(&mut w);
    }
    assert!(!var(&w, 0), "trigger 0 is in no chain");
    assert!(var(&w, 1));
    // The repeating tag's instance fires every 15 ticks; the one-time tag's once.
    assert_eq!(w.triggers.tags.len(), 1, "the one-time tag is gone");
}

// spec: S-0011 claim 3 (a disabled definition or a clear difficulty flag starts disabled and never fires)
#[test]
fn enabled_at_start() {
    let mut off = trig(vec![ev(8, 0)], vec![set(0)]);
    off.disabled = true;
    let mut hard = trig(vec![ev(8, 0)], vec![set(1)]);
    hard.difficulty = [true, false, true];
    let on = trig(vec![ev(8, 0)], vec![set(2)]);
    let mut w = game(triggers(
        vec![off, hard, on],
        vec![tag(0, vec![0]), tag(0, vec![1]), tag(0, vec![2])],
    ));
    for _ in 0..3 {
        tick::step(&mut w);
    }
    // spec: S-0011 claim 4 (absent AIDifficulty: position 1)
    assert_eq!(w.triggers.difficulty, 1);
    assert_eq!((var(&w, 0), var(&w, 1), var(&w, 2)), (false, false, true));
}

// spec: S-0011 claim 4 (a setting outside 0 … 2 selects no difficulty flag: every not-disabled instance starts enabled)
#[test]
fn difficulty_outside_range() {
    let mut r = testkit::ruleset();
    r.sim.general.economy.ai_difficulty = Some(5);
    let rules = Arc::new(SimRules::new(&r, "synthetic-test"));
    let mut t = trig(vec![ev(8, 0)], vec![set(0)]);
    t.difficulty = [false; 3];
    let mut w = game_rules(triggers(vec![t], vec![tag(0, vec![0])]), rules);
    tick::step(&mut w);
    assert!(var(&w, 0));
}

// spec: S-0011 claims 6 and 7 (the poll follows registration order: a later tag sees an earlier tag's effects in the same tick)
#[test]
fn poll_follows_registration_order() {
    let defs = vec![
        trig(vec![ev(8, 0)], vec![set(0)]),
        trig(vec![ev(36, 0)], vec![set(1)]),
        trig(vec![ev(36, 2)], vec![set(3)]),
        trig(vec![ev(8, 0)], vec![set(2)]),
    ];
    let tags = vec![
        tag(0, vec![0]),
        tag(0, vec![1]),
        tag(0, vec![2]),
        tag(0, vec![3]),
    ];
    let mut w = game(triggers(defs, tags));
    tick::step(&mut w);
    assert!(
        var(&w, 0) && var(&w, 1),
        "tag 1 sees tag 0's variable in tick 0"
    );
    assert!(
        var(&w, 2) && !var(&w, 3),
        "tag 2 was polled before tag 3 set its variable"
    );
    tick::step(&mut w);
    assert!(var(&w, 3), "the next tick's poll sees it");
}

// spec: S-0011 claim 10 (one-time fires once and goes; repeating fires every time; another repeat value never fires)
#[test]
fn repeat_types() {
    let defs = vec![trig(vec![ev(8, 0)], vec![])];
    let mut w = game(triggers(defs.clone(), vec![tag(0, vec![0])]));
    for _ in 0..5 {
        tick::step(&mut w);
    }
    assert_eq!(w.triggers.fired, 1);
    assert!(w.triggers.tags.is_empty());
    let mut w = game(triggers(defs.clone(), vec![tag(2, vec![0])]));
    for _ in 0..5 {
        tick::step(&mut w);
    }
    assert_eq!(w.triggers.fired, 5);
    let mut w = game(triggers(defs, vec![tag(7, vec![0])]));
    for _ in 0..5 {
        tick::step(&mut w);
    }
    assert_eq!(w.triggers.fired, 0);
}

// spec: S-0011 claim 10 (an all-carriers tag with no carrier never fires, quirks.all_carriers_without_carrier)
#[test]
fn all_carriers_without_carrier() {
    let defs = vec![trig(vec![ev(8, 0)], vec![set(0)])];
    let mut w = game(triggers(defs.clone(), vec![tag(1, vec![0])]));
    for _ in 0..3 {
        tick::step(&mut w);
    }
    assert!(!var(&w, 0));
    // spec: S-0011 Edge cases (fixed behaviour: a carrier count of 0 counts like 1)
    let mut r = testkit::ruleset();
    r.globals.quirks.all_carriers_without_carrier = false;
    let rules = Arc::new(SimRules::new(&r, "synthetic-test"));
    let mut w = game_rules(triggers(defs, vec![tag(1, vec![0])]), rules);
    tick::step(&mut w);
    assert!(var(&w, 0));
}

// spec: S-0011 claim 11 (the observable effects of chain members follow reverse chain order)
#[test]
fn chain_members_reverse_order() {
    let defs = vec![
        TriggerDef {
            link: Some(1),
            ..trig(vec![ev(8, 0)], vec![set(0)])
        },
        trig(vec![ev(8, 0)], vec![clear(0)]),
    ];
    let mut w = game(triggers(defs, vec![tag(2, vec![0, 1])]));
    tick::step(&mut w);
    assert!(
        var(&w, 0),
        "the first member's set comes after the second's clear"
    );
}

// spec: S-0011 claim 11 (a member's actions can stop another member from firing: the later member disables the first)
#[test]
fn chain_member_disables_earlier() {
    let mut defs = vec![
        trig(vec![ev(8, 0)], vec![set(0)]),
        trig(vec![ev(8, 0)], vec![act(54, 0)]),
    ];
    defs[1].actions[0].trigger = Some(0);
    let mut w = game(triggers(defs, vec![tag(2, vec![0, 1])]));
    tick::step(&mut w);
    assert!(!var(&w, 0));
}

// spec: S-0011 claim 14 (one ranged draw per random-delay event with v ≠ 0 when the timer is armed)
#[test]
fn random_delay_draws() {
    let base = game(triggers(vec![], vec![]));
    let defs = vec![trig(vec![ev(51, 10), ev(51, 0), ev(51, 6)], vec![set(0)])];
    let mut w = game(triggers(defs, vec![tag(0, vec![0])]));
    assert_eq!(w.rng.taken() - base.rng.taken(), 2);
    // spec: S-0011 F2 (v = 10 gives lengths from 75 to 225 ticks)
    let len = w.triggers.tags[&0].instances[0].length;
    assert!((75..=225).contains(&len), "{len}");
    assert_eq!(len % 15, 0);
    assert_eq!(tick_set(&mut w, 0, 300), Some(len));
}

// spec: S-0011 claim 15 (a repeating polled trigger with only a timer event fires every D ticks)
#[test]
fn repeating_timer_rearms() {
    let defs = vec![trig(vec![ev(13, 2)], vec![])];
    let mut w = game(triggers(defs, vec![tag(2, vec![0])]));
    for _ in 0..91 {
        tick::step(&mut w);
    }
    // Ticks 30, 60, 90.
    assert_eq!(w.triggers.fired, 3);
}

// spec: S-0011 claim 16 (game time v: satisfied in every tick t with v ≤ t quo 15, not affected by arming)
#[test]
fn game_time() {
    let defs = vec![trig(vec![ev(47, 3)], vec![])];
    let mut w = game(triggers(defs, vec![tag(2, vec![0])]));
    for _ in 0..50 {
        tick::step(&mut w);
    }
    // Ticks 45 … 49.
    assert_eq!(w.triggers.fired, 5);
}

// spec: S-0011 claims 17 and 35 (a repeating any-event fires at every poll and again at every eighth tick when its tag is raised by its owner house)
#[test]
fn any_event_and_owner_house_raise() {
    let defs = vec![TriggerDef {
        owner: Owner::Name(b"alphaland".to_vec()),
        ..trig(vec![ev(8, 0)], vec![])
    }];
    let mut w = game(triggers(defs.clone(), vec![tag(2, vec![0])]));
    assert_eq!(w.triggers.house_tags[0], vec![0]);
    for _ in 0..16 {
        tick::step(&mut w);
    }
    // 16 polls, owner-house raises in ticks 0 and 8.
    assert_eq!(w.triggers.fired, 18);
    // spec: S-0011 claim 35 (a tag whose first trigger's owner country nobody plays is never raised this way)
    let none = vec![TriggerDef {
        owner: Owner::Name(b"Nowhere".to_vec()),
        ..defs[0].clone()
    }];
    let w = game(triggers(none, vec![tag(2, vec![0])]));
    assert!(w.triggers.house_tags.iter().all(Vec::is_empty));
}

// spec: S-0011 claim 35 (for tags assigned to one house, the effects follow reverse registration order)
#[test]
fn house_raise_reverse_registration_order() {
    let owned = |a| TriggerDef {
        owner: Owner::Name(b"Alphaland".to_vec()),
        ..trig(vec![ev(8, 0)], vec![a])
    };
    let defs = vec![owned(clear(3)), owned(set(3))];
    let mut w = game(triggers(defs, vec![tag(2, vec![0]), tag(2, vec![1])]));
    tick::step(&mut w);
    // Poll: clear, set; house raise (tick 0): set, clear.
    assert!(!var(&w, 3));
    tick::step(&mut w);
    // Tick 1: only the poll.
    assert!(var(&w, 3));
}

// spec: S-0011 claim 22 (a change re-arms the timers of the instances with a variable event for that variable; an unchanged assignment does not)
#[test]
fn variable_change_rearms() {
    let defs = vec![
        trig(vec![ev(13, 2), ev(37, 9)], vec![set(0)]),
        trig(vec![ev(13, 1)], vec![set(9)]),
    ];
    let mut w = game(triggers(defs, vec![tag(0, vec![0]), tag(0, vec![1])]));
    // Tick 15: variable 9 set, re-arming trigger 0 (its clear event is then false anyway).
    for _ in 0..16 {
        tick::step(&mut w);
    }
    assert!(var(&w, 9));
    assert_eq!(w.triggers.tags[&0].instances[0].start, 15);
}

// spec: S-0011 claim 25 (enable re-arms the timer, even when already enabled; disable keeps it running)
#[test]
fn enable_rearms() {
    let mut defs = vec![
        trig(vec![ev(13, 4)], vec![set(0)]),
        trig(vec![ev(13, 2)], vec![act(53, 0)]),
    ];
    defs[1].actions[0].trigger = Some(0);
    let mut w = game(triggers(defs, vec![tag(0, vec![0]), tag(0, vec![1])]));
    // Enabled again in tick 30: fires in tick 90, not 60.
    assert_eq!(tick_set(&mut w, 0, 200), Some(90));
}

// spec: S-0011 claim 31 (mission timer set, extend, shorten, stop and start)
#[test]
fn mission_timer() {
    let defs = vec![
        trig(vec![ev(13, 0)], vec![act(27, 10)]),
        trig(vec![ev(13, 2)], vec![act(25, 2), act(24, 0)]),
        trig(vec![ev(13, 4)], vec![act(23, 0), act(26, 100)]),
    ];
    let mut w = game(triggers(
        defs,
        vec![tag(0, vec![0]), tag(0, vec![1]), tag(0, vec![2])],
    ));
    tick::step(&mut w);
    assert_eq!(w.triggers.mission.remaining(w.tick), 149);
    for _ in 0..30 {
        tick::step(&mut w);
    }
    // Tick 30: remaining 120 + 30, then stopped.
    let m = w.triggers.mission.clone();
    assert!(!m.running);
    assert_eq!(m.remaining(1000), 150);
    for _ in 0..30 {
        tick::step(&mut w);
    }
    // Tick 60: started, shortened below 0: 0.
    assert_eq!(w.triggers.mission.remaining(w.tick), 0);
}

// spec: S-0011 claims 32 and 33 (sound at a waypoint is presentation; an undefined marker stands for cell (0, 0))
#[test]
fn sound_at_waypoint_effect() {
    let a = Action {
        code: 99,
        kind: 7,
        name: b"BirdSound".to_vec(),
        number: -1,
        waypoint: 3,
        ..Default::default()
    };
    let mut t = triggers(vec![trig(vec![ev(13, 1)], vec![a])], vec![tag(0, vec![0])]);
    t.variables = vec![None; 100];
    let mut m = map_with(t.clone());
    m.waypoints.insert(3, (5, 6));
    let mut w = glasnost_sim::load::new_game(
        Arc::new(SimRules::new(&testkit::ruleset(), "synthetic-test")),
        &m,
        testkit::catalogue(),
        &testkit::setup(7),
    )
    .unwrap();
    let mut seen = Vec::new();
    for _ in 0..16 {
        tick::step(&mut w);
        seen.extend(w.effects.drain(..).filter_map(|e| match e {
            Effect::Trigger {
                code, name, pos, ..
            } => Some((code, name, pos)),
            _ => None,
        }));
    }
    assert_eq!(seen.len(), 1);
    assert_eq!(seen[0].0, 99);
    assert_eq!(seen[0].1, "BirdSound");
    let p = seen[0].2.unwrap();
    assert_eq!((p.x, p.y), (5 * 256 + 128, 6 * 256 + 128));
    // An undefined waypoint: cell (0, 0).
    let mut w = game(t);
    let mut pos = None;
    for _ in 0..16 {
        tick::step(&mut w);
        for e in w.effects.drain(..) {
            if let Effect::Trigger { pos: p, .. } = e {
                pos = p;
            }
        }
    }
    let p = pos.unwrap();
    assert_eq!((p.x, p.y), (128, 128));
}

// spec: S-0011 claim 9 (a latchable event of a repeating tag stays satisfied after its first satisfaction, quirks.persistent_event_latch)
#[test]
fn persistent_latch() {
    let defs = vec![trig(vec![ev(48, 0)], vec![])];
    let raise = |w: &mut World, code| {
        triggers::raise(
            w,
            0,
            Raise {
                code,
                causer: None,
                carrier: None,
            },
        )
    };
    let mut w = game(carried(defs.clone()));
    // spec: S-0011 claim 34 (a tag that is not polled exists through its carrier, here a cell)
    // Not polled (48 is code-matched): nothing fires by itself.
    tick::step(&mut w);
    assert_eq!(w.triggers.fired, 0);
    raise(&mut w, Some(48));
    assert_eq!(w.triggers.fired, 1);
    raise(&mut w, Some(8));
    assert_eq!(
        w.triggers.fired, 2,
        "the latch satisfies a raise of any code"
    );
    // spec: S-0011 Edge cases (fixed behaviour: the latches are cleared each time the instance fires)
    let mut r = testkit::ruleset();
    r.globals.quirks.persistent_event_latch = false;
    let rules = Arc::new(SimRules::new(&r, "synthetic-test"));
    let mut w = game_rules(carried(defs), rules);
    raise(&mut w, Some(48));
    raise(&mut w, Some(8));
    assert_eq!(w.triggers.fired, 1);
}

// spec: S-0011 claim 9 (positions from 33 on share the latch 32 lower; arming clears a timer position's latch, and with it the aliased one, quirks.event_latch_aliasing)
#[test]
fn latch_aliasing() {
    let mut events = vec![ev(13, 0)];
    events.extend((2..=32).map(|_| ev(8, 0)));
    events.push(ev(48, 0));
    let defs = vec![trig(events, vec![])];
    let raise = |w: &mut World, code| {
        triggers::raise(
            w,
            0,
            Raise {
                code,
                causer: None,
                carrier: None,
            },
        )
    };
    let mut w = game(triggers(defs.clone(), vec![tag(2, vec![0])]));
    let f0 = w.triggers.fired;
    raise(&mut w, Some(48));
    assert_eq!(w.triggers.fired, f0 + 1);
    // The re-arming of the firing cleared position 1, and so the aliased latch of 33.
    raise(&mut w, Some(8));
    assert_eq!(w.triggers.fired, f0 + 1);
    let mut r = testkit::ruleset();
    r.globals.quirks.event_latch_aliasing = false;
    let rules = Arc::new(SimRules::new(&r, "synthetic-test"));
    let mut w = game_rules(triggers(defs, vec![tag(2, vec![0])]), rules);
    let f0 = w.triggers.fired;
    raise(&mut w, Some(48));
    raise(&mut w, Some(8));
    assert_eq!(w.triggers.fired, f0 + 2);
}

// spec: S-0011 claim 12 (a raise of a tag while it is being raised has no effect) and State and ordering (a one-time tag's raising carrier loses the tag)
#[test]
fn cell_carrier_loses_one_time_tag() {
    let mut t = triggers(
        vec![trig(vec![ev(48, 0)], vec![set(0)])],
        vec![tag(0, vec![0])],
    );
    t.cell_tags = vec![((4, 4), 0)];
    let mut w = game(t);
    // spec: S-0011 claim 34 (the cell tag created the tag at loading)
    assert_eq!(w.triggers.tags[&0].carriers, 1);
    triggers::raise(
        &mut w,
        0,
        Raise {
            code: Some(48),
            causer: None,
            carrier: Some(Carrier::Cell((4, 4))),
        },
    );
    assert!(var(&w, 0));
    assert!(w.triggers.cells.is_empty());
    tick::step(&mut w);
    assert!(w.triggers.tags.is_empty(), "deleted at the end of the tick");
}

// spec: S-0011 claim 21 (type exists with a count, and type does not exist, by the exact type name)
#[test]
fn type_events() {
    let named = |code, value, name: &[u8]| Event {
        code,
        kind: 2,
        value,
        type_name: Some(name.to_vec()),
    };
    let defs = vec![
        trig(vec![named(60, 1, b"CART")], vec![set(0)]),
        trig(vec![named(61, 0, b"CART")], vec![set(1)]),
        trig(vec![named(61, 0, b"cart")], vec![set(2)]),
    ];
    let mut w = game(triggers(
        defs,
        vec![tag(0, vec![0]), tag(0, vec![1]), tag(0, vec![2])],
    ));
    tick::step(&mut w);
    assert!(!var(&w, 0));
    assert!(var(&w, 1));
    // spec: S-0011 claim 21 (no type with that exact name: neither event is ever judged true)
    assert!(!var(&w, 2));
    testkit::spawn(
        &mut w,
        glasnost_sim::rules::Kind::Vehicle,
        "CART",
        0,
        testkit::mid(20, 0, 0),
        0,
    );
    tick::step(&mut w);
    assert!(var(&w, 0));
}

// The state hash survives a save and load in the middle of a timer.
// spec: S-0011 claim 14 (the draws at arming) and S-0011 claim 15 (re-arming)
#[test]
fn save_load_mid_timer() {
    let defs = vec![trig(vec![ev(51, 6)], vec![])];
    let mut w = game(triggers(defs, vec![tag(2, vec![0])]));
    for _ in 0..50 {
        tick::step(&mut w);
    }
    let bytes = save::save(&w);
    let mut v = save::load(&bytes, w.rules_arc(), testkit::catalogue()).unwrap();
    assert_eq!(save::state_hash(&v), save::state_hash(&w));
    for _ in 0..200 {
        tick::step(&mut w);
        tick::step(&mut v);
    }
    assert_eq!(save::state_hash(&v), save::state_hash(&w));
    assert!(w.triggers.fired >= 2);
}

/// A repeating tag whose only trigger is destroyed-by-anything with no
/// action, carried by a vehicle; the firings of `destruction_raises`.
fn destruction_firings(r: glasnost_rules::schema::Ruleset, repeat: i32, attacker: bool) -> u64 {
    let extra = r.globals.quirks.vehicle_destruction_extra_raises;
    let rules = Arc::new(SimRules::new(&r, "synthetic-test"));
    let mut w = game_rules(
        triggers(
            vec![trig(vec![ev(48, 0)], vec![])],
            vec![tag(repeat, vec![0])],
        ),
        rules,
    );
    let v = testkit::spawn(
        &mut w,
        glasnost_sim::rules::Kind::Vehicle,
        "CART",
        0,
        glasnost_map::CellXY::new(3, 3),
        0,
    );
    triggers::object_tag(&mut w, v, Some(b"TAG"));
    let f0 = w.triggers.fired;
    triggers::destruction_raises(&mut w, v, attacker, extra);
    w.triggers.fired - f0
}

// spec: S-0011 claim 19 and scenario T7 (a fresh repeating destroyed-by-anything trigger fires seven times for a normal vehicle death with an attacker and four without; S-0312 claim 29's groups)
#[test]
fn vehicle_destruction_firings() {
    assert_eq!(destruction_firings(testkit::ruleset(), 2, true), 7);
    assert_eq!(destruction_firings(testkit::ruleset(), 2, false), 4);
}

// spec: S-0011 claim 19 (a common-group-only death fires twice; quirks.vehicle_destruction_extra_raises off omits the extra vehicle group)
#[test]
fn common_group_only_firings() {
    let mut r = testkit::ruleset();
    r.globals.quirks.vehicle_destruction_extra_raises = false;
    assert_eq!(destruction_firings(r.clone(), 2, true), 2);
    assert_eq!(destruction_firings(r, 2, false), 2);
}

// spec: S-0011 claim 19 (persistent latching off: twice with the extra group, once without; repeat type 0: one firing)
#[test]
fn destruction_firings_without_the_latch() {
    let mut r = testkit::ruleset();
    r.globals.quirks.persistent_event_latch = false;
    assert_eq!(destruction_firings(r.clone(), 2, true), 2);
    r.globals.quirks.vehicle_destruction_extra_raises = false;
    assert_eq!(destruction_firings(r, 2, true), 1);
    assert_eq!(destruction_firings(testkit::ruleset(), 0, true), 1);
}

// spec: S-0011 claim 18 (a unit's arrival raises the cell's tag with the entered-by code and the unit as causing object; any house)
#[test]
fn cell_entered_raises_the_cell_tag() {
    let mut t = triggers(
        vec![trig(vec![ev(1, -1)], vec![set(0)])],
        vec![tag(0, vec![0])],
    );
    t.cell_tags = vec![((4, 4), 0)];
    let mut w = game(t);
    let v = testkit::spawn(
        &mut w,
        glasnost_sim::rules::Kind::Vehicle,
        "CART",
        0,
        glasnost_map::CellXY::new(4, 4),
        0,
    );
    triggers::cell_entered(&mut w, v, (5, 4));
    assert!(!var(&w, 0), "an untagged cell raises nothing");
    triggers::cell_entered(&mut w, v, (4, 4));
    assert!(var(&w, 0));
    assert!(
        !w.triggers.cells.contains_key(&(4, 4)),
        "the one-time tag's raising cell loses it"
    );
}

// spec: S-0011 claim 15 (Q-0269: an effective self-enable action arms the instance again, with its own draws, and its length replaces the satisfaction-caused one; both armings have the firing tick as arming tick)
#[test]
fn self_enable_rearms_again() {
    // A repeating random-delay trigger (value 10); the self-enabling case
    // takes one more arming draw at each firing than the plain case.
    let run = |self_enable: bool| {
        let mut en = act(53, 0);
        en.trigger = Some(0);
        let actions = if self_enable {
            vec![en, set(0)]
        } else {
            vec![set(0)]
        };
        let mut w = game(triggers(
            vec![trig(vec![ev(51, 10)], actions)],
            vec![tag(2, vec![0])],
        ));
        let before = w.rng.clone();
        let t = tick_set(&mut w, 0, 400).expect("fires");
        let inst = w.triggers.tags[&0].instances[0].clone();
        (before, w.rng.clone(), t, inst.start)
    };
    let (b0, a0, t0, s0) = run(false);
    let (b1, a1, t1, s1) = run(true);
    assert_eq!(format!("{b0:?}"), format!("{b1:?}"));
    assert_eq!(t0, t1, "the first delay is the same draw");
    assert_eq!((s0, s1), (t0, t1));
    assert_ne!(
        format!("{a0:?}"),
        format!("{a1:?}"),
        "the self-enable arming takes its own draws"
    );
}
