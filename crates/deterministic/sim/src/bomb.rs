//! Planted bombs, suicide weapons and demolition charges (S-0519 claims 2
//! to 15). A planted bomb is state of its carrier (`CombatState::bomb`); a
//! charge is the delayed-kill countdown of S-0302 claim 16 with a recorded
//! planter (`CombatState::charge_planter`). No site takes a random draw of
//! its own; the detonations and animations take theirs (S-0303).

use serde::{Deserialize, Serialize};

use glasnost_num::geom::root_floor;

use crate::combat::damage::{self, Hit};
use crate::rules::{Behaviour, Kind};
use crate::scatter::{self, ScatterRequest};
use crate::world::{HouseId, ObjectId, Target, World};

/// A planted bomb (S-0519 Terms).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bomb {
    /// The planter, while it exists (claim 8).
    pub planter: Option<ObjectId>,
    /// The planter's house at the moment of planting (claim 2).
    pub house: Option<HouseId>,
    /// The plant tick *t*.
    pub planted: i32,
}

/// Whether the object carries a planted bomb.
pub fn carries(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| o.combat.bomb.is_some())
}

/// A bomb-planting projectile detonates (claim 2): the planter must still
/// exist and be infantry, the target a unit, infantry unit, aircraft or
/// building that carries no bomb. The distance is not tested.
pub fn plant(w: &mut World, firer: Option<ObjectId>, target: Option<Target>) {
    let (Some(f), Some(Target::Object(x))) = (firer, target) else {
        return;
    };
    // spec: S-0519 claim 2 (the firer still exists and is an infantry unit)
    let Some(planter) = w.obj(f).filter(|o| o.kind == Kind::Infantry) else {
        return;
    };
    let house = planter.owner;
    // spec: S-0519 claim 2 (the target is a unit, an infantry unit, an aircraft or a building that carries no planted bomb)
    if w.obj(x).is_none_or(|o| {
        o.combat.bomb.is_some()
            || !matches!(
                o.kind,
                Kind::Vehicle | Kind::Infantry | Kind::Aircraft | Kind::Building
            )
    }) {
        return;
    }
    let tick = w.tick;
    if let Some(ob) = w.obj_mut(x) {
        // spec: S-0519 claim 2 (the bomb records its planter, the planter's house, its carrier and the plant tick)
        ob.combat.bomb = Some(Bomb {
            planter: Some(f),
            house,
            planted: tick,
        });
    }
    // spec: S-0519 claim 2 (the bomb-attach sound for the local player's planter is presentation: not built)
}

/// A disarm projectile detonates (claim 7): the bomb is removed with no
/// damage, no animation and no credit.
pub fn disarm(w: &mut World, target: Option<Target>) {
    if let Some(Target::Object(x)) = target
        && let Some(ob) = w.obj_mut(x)
    {
        ob.combat.bomb = None;
    }
}

/// Claim 3: whether a bomb-planting weapon is illegal against a target that
/// carries a bomb, or a disarm weapon against one that carries none.
pub fn illegal_target(w: &World, weapon: crate::combat::data::WeaponId, target: Target) -> bool {
    let Target::Object(x) = target else {
        return false;
    };
    let Some(h) = w.rules().combat.warhead_of(weapon) else {
        return false;
    };
    let carries = carries(w, x);
    // spec: S-0519 claim 3 (bomb-planting: illegal against a carrier; disarm: illegal against a target with none)
    (h.ivan_bomb && carries) || (h.bomb_disarm && !carries)
}

/// The carrier's own turn (claim 4): a timed bomb goes off in the first turn
/// on the map in a tick later than *t* + *D* (F1). A carrier off the map
/// cannot set it off (claim 6).
pub fn turn(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    let Some(b) = o.combat.bomb else { return };
    if !o.on_map {
        return;
    }
    let due = b.planted.wrapping_add(w.rules().combat.general.ivan.delay);
    // spec: S-0519 F1 (n > t + D, an exact integer comparison)
    if w.tick.wrapping_sub(due) > 0 {
        detonate(w, id);
    }
}

/// Sets a bomb off (claims 5 and 6): on the map, one detonation of the
/// bomb damage with the bomb warhead at the carrier, the planter as
/// attacker; off the map it is removed with no effect.
pub fn detonate(w: &mut World, carrier: ObjectId) {
    let Some(o) = w.obj(carrier) else { return };
    let Some(b) = o.combat.bomb else { return };
    let (on_map, at) = (o.on_map, o.pos);
    let hut = o.kind == Kind::Building && crate::bridges::hut::is_hut(w, carrier);
    if let Some(ob) = w.obj_mut(carrier) {
        // The bomb ends first: a carrier the detonation kills sets nothing off a second time (S-0312 claim 7).
        ob.combat.bomb = None;
    }
    if !on_map {
        // spec: S-0519 claim 6 (a bomb set off while its carrier is off the map is removed with no detonation, no animation and no damage)
        return;
    }
    let ivan = w.rules().combat.general.ivan.clone();
    // spec: S-0519 Edge cases (no bomb warhead: unspecified, our engine does nothing)
    if let Some(wh) = ivan.warhead {
        // spec: S-0519 claim 5 (one detonation at the carrier's position with the bomb damage and warhead; the attacker is the planter while it exists, no attacking house, defences apply)
        let source = b.planter.filter(|&p| w.obj(p).is_some());
        crate::combat::projectile::detonate_area(w, wh, ivan.damage, at, source, None, true);
    }
    if hut && w.obj(carrier).is_some() {
        // spec: S-0519 claim 5 (a bridge repair hut also demolishes its bridge, S-0511 claim 16)
        crate::bridges::hut::demolish(w, carrier);
    }
}

/// A killing hit on the carrier (claim 5 and S-0312 claim 7): a bomb it
/// carries goes off once on the map and is removed silently off it.
pub fn carrier_killed(w: &mut World, id: ObjectId) {
    if carries(w, id) {
        detonate(w, id);
    }
}

/// An object leaves play (claims 7 and 8): a carrier removed other than by
/// death loses its bomb silently; a bomb whose planter leaves forgets it.
pub fn removed(w: &mut World, id: ObjectId) {
    for ob in w.objects.values_mut() {
        if let Some(b) = ob.combat.bomb.as_mut()
            && b.planter == Some(id)
        {
            // spec: S-0519 claim 8 (the bomb forgets its planter but keeps the planter's house)
            b.planter = None;
        }
    }
    if let Some(ob) = w.obj_mut(id) {
        // spec: S-0519 claim 7 (removal other than by death: no detonation, no animation)
        ob.combat.bomb = None;
    }
}

/// A building changes owner (claim 7): a bomb on a type that is not
/// garrisonable is disarmed without a detonation.
pub fn building_owner_changed(w: &mut World, b: ObjectId) {
    if !crate::structures::garrison::garrisonable(w, b)
        && let Some(ob) = w.obj_mut(b)
    {
        ob.combat.bomb = None;
    }
}

/// The detonate order (S-0213 claim 11): the bomb is resolved at once.
pub fn detonate_order(w: &mut World, actor: ObjectId) {
    // spec: S-0519 claim 9 (the bomb goes off at once, or is removed silently with the carrier off the map)
    detonate(w, actor);
}

/// Whether the viewing house sees the carrier's planted bomb (claim 10, F2):
/// it is the planter's house, or one of its bomb detectors on the map is
/// closer than its bomb-sight range.
pub fn visible(w: &World, viewer: HouseId, carrier: ObjectId) -> bool {
    let Some(b) = w.obj(carrier).and_then(|o| o.combat.bomb) else {
        return false;
    };
    // spec: S-0519 claim 10 (the planter's house sees it)
    if b.house == Some(viewer) {
        return true;
    }
    let r = w.rules();
    // spec: S-0519 F2 (the positions are the reference points of S-0204 claim 4)
    let at = w.reference_point(carrier);
    w.objects.values().any(|d| {
        let s = crate::combat::veterancy::type_combat(w, d.id).map_or(0, |tc| tc.bomb_sight);
        if d.owner != Some(viewer) || !d.on_map || s <= 0 {
            return false;
        }
        let p = w.reference_point(d.id);
        let sq = |a: i32| i64::from(a) * i64::from(a);
        // spec: S-0519 F2 (N = Δx² + Δy² + Δz²; d = trunc(r(N)), the table root; the detector sees the bomb when d < 256 s)
        let n =
            sq(p.x.wrapping_sub(at.x)) + sq(p.y.wrapping_sub(at.y)) + sq(p.z.wrapping_sub(at.z));
        let dist = root_floor(n as u64, r.quirks.table_root_distance, r.mode) as i64;
        dist < 256 * i64::from(s) // spec: S-0519 Constants (detection scale 256 leptons per cell)
    })
}

/// Row G3's test (S-0214, S-0519 claim 9): the object carries a bomb of a
/// kind the general rules let the player set off. Every planted bomb is a
/// timed bomb, so only the timed-bomb permission can allow it.
pub fn detonable(w: &World, id: ObjectId) -> bool {
    carries(w, id) && w.rules().combat.general.ivan.can_detonate_time_bomb
}

// ---------------------------------------------------------------------------
// Demolition charges

/// The charge countdown *K* in ticks (F4): trunc(RZ₅₃(*c* × 900)).
pub fn charge_countdown(w: &World) -> i32 {
    let r = w.rules();
    // spec: S-0519 F4 (K = trunc(RZ₅₃(c × 900)); 900 ticks per minute)
    r.combat
        .general
        .ivan
        .c4_delay
        .mul(glasnost_num::F64::from_i64(900, r.mode), r.mode)
        .to_i32_common()
}

/// The flash length *L* in ticks (F4), presentation only.
pub fn charge_flash(w: &World) -> i32 {
    let r = w.rules();
    // spec: S-0519 F4 (L = trunc(RZ₅₃(c × 900) × 0.5); halving is exact)
    r.combat
        .general
        .ivan
        .c4_delay
        .mul(glasnost_num::F64::from_i64(900, r.mode), r.mode)
        .mul(crate::consts::HALF, r.mode)
        .to_i32_common()
}

/// Whether the walker may plant a charge: the type flag, or with
/// `quirks.charge_needs_type_flag` off the veterancy ability too (claim 13).
pub(crate) fn has_charges(w: &World, id: ObjectId) -> bool {
    let flag = w
        .obj(id)
        .and_then(|o| w.rules().unit(o.kind, o.type_index))
        .is_some_and(|u| u.raw.c4.unwrap_or(false));
    flag || (!w.rules().quirks.charge_needs_type_flag
        && crate::combat::veterancy::has_ability(w, id, crate::combat::data::Ability::C4))
}

/// An infantry unit's arrival at its destination building in *sabotage*
/// (claim 13): the entered-by raise, then the table: a building being sold
/// or protected refuses (movement aborted, uncloaked, rearm, scatter); one
/// with a countdown running refuses (destination cleared, rearm, scatter);
/// otherwise the charge is planted (claim 14), also with rearm and scatter.
/// Returns whether the arrival was handled.
pub fn charge_arrival(w: &mut World, walker: ObjectId) -> bool {
    let Some(o) = w.obj(walker) else { return false };
    if o.kind != Kind::Infantry || !o.on_map || o.beh.current != Some(Behaviour::Sabotage) {
        return false;
    }
    // spec: S-0519 claim 13 (the walker is in sabotage, its type has explosive charges, and its cell holds the building that is its destination)
    if !has_charges(w, walker) {
        return false;
    }
    let cell = o.pos.cell();
    let Some(Target::Object(b)) = o.dest else {
        return false;
    };
    if w.obj(b)
        .is_none_or(|x| x.kind != Kind::Building || !x.on_map)
        || !w.cells.list(cell, crate::world::Layer::Ground).contains(&b)
    {
        return false;
    }
    // spec: S-0519 claim 13 (the building's trigger tag first gets the entered-by event, with the walker as the cause)
    crate::mp_triggers::entry_raise(w, b, walker);
    if w.obj(walker).is_none() || w.obj(b).is_none() {
        return true;
    }
    let (selling, running) = {
        let ob = w.obj(b).expect("checked");
        (
            ob.beh.current == Some(Behaviour::Selling),
            ob.combat.delay_kill_until.is_some(),
        )
    };
    let refused_first = selling || crate::superweapons::protected(w, b);
    if refused_first {
        // spec: S-0519 claim 13 (row 1: no charge; its movement is aborted; it stops being cloaked at once, without a sound)
        if let Some(ob) = w.obj_mut(walker) {
            ob.dest = None;
            ob.path.steps.clear();
        }
        crate::cloak::machine::uncloak(w, walker, false);
    } else if running {
        // spec: S-0519 claim 13 (row 2: no charge, the running countdown and its planter stay; its destination is cleared, no abort and no change to its cloak)
        if let Some(ob) = w.obj_mut(walker) {
            ob.dest = None;
        }
    } else {
        plant_charge(w, walker, b);
        // spec: S-0519 claim 13 (row 3: planted, with the effects on the walker of row 1)
        if let Some(ob) = w.obj_mut(walker) {
            ob.dest = None;
            ob.path.steps.clear();
        }
        crate::cloak::machine::uncloak(w, walker, false);
    }
    // spec: S-0519 claim 13 (rearm: in every row the countdown is replaced by the arrival delay of the second weapon, chosen with the present burst counter; arrival is not a shot and does not advance it)
    let d = crate::combat::fire::arrival_delay(w, walker);
    let t = w.tick;
    if let Some(ob) = w.obj_mut(walker) {
        ob.combat.rearm_until = t.wrapping_add(d);
        ob.combat.rearm_delay = d;
    }
    // spec: S-0519 claim 13 (scatter: forced and move-anywhere, the threat point the building's centre)
    let centre = w.reference_point(b);
    scatter::request(
        w,
        walker,
        ScatterRequest {
            threat: Some(centre),
            forced: true,
            anywhere: true,
        },
    );
    true
}

/// Plants the charge (claim 14): the countdown *K* starts in this tick and
/// the walker is recorded as the building's charge planter.
fn plant_charge(w: &mut World, walker: ObjectId, b: ObjectId) {
    let k = charge_countdown(w);
    let l = charge_flash(w);
    let now = w.tick;
    if let Some(ob) = w.obj_mut(b) {
        // spec: S-0519 claim 14 (a pending countdown of K ticks, the planter recorded)
        ob.combat.delay_kill_until = Some(now.wrapping_add(k));
        ob.combat.charge_planter = Some(walker);
    }
    // spec: S-0519 claims 14 and 25 (the building flashes for L ticks: its flash count becomes L)
    crate::mark::start_flash(w, b, l);
}

/// A suicide shot (claim 11): the firer takes one hit of its type's
/// strength with the demolition warhead, at distance 0, with no attacker
/// and no attacking house, ignoring defences and with no no-escape marker.
pub fn suicide(w: &mut World, firer: ObjectId) {
    let amount = damage::strength(w, firer);
    // spec: S-0519 claim 11 (the hit: the strength of the firer's type, distance 0, the demolition warhead, no attacker, no house, ignores defences, no no-escape marker)
    damage::apply(
        w,
        firer,
        Hit {
            amount,
            warhead: w.rules().combat.general.c4_warhead,
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: true,
            no_escape: false,
        },
    );
}
