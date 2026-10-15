//! Presentation queries for a viewing house *V* (S-0516 claims 26 to 29,
//! 36 to 38). Read-only: nothing here feeds game state.

use super::machine::CloakEvent;
use super::{CloakState, Disguise, DisguiseAs, appears_disguised, detected, fade_fraction, senses};
use crate::rules::Kind;
use crate::world::{HouseId, ObjectId, World};

/// How an object is drawn (S-0516 claim 28, Table B).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Look {
    /// Drawn normally.
    Normal,
    /// Drawn 25 % translucent.
    Translucent25,
    /// Drawn 50 % translucent.
    Translucent50,
    /// The rippling, warped image.
    Rippling,
    /// Not drawn.
    Hidden,
}

/// Whether *V* and the owner are mutually allied.
fn mutual(w: &World, a: Option<HouseId>, b: Option<HouseId>) -> bool {
    w.allied(a, b) && w.allied(b, a)
}

/// The fully cloaked rule of claim 28: 50 % translucent for the owner, a
/// sensing viewer or a mutual ally, otherwise not drawn.
fn cloaked_look(w: &World, id: ObjectId, v: Option<HouseId>) -> Look {
    let Some(o) = w.obj(id) else {
        return Look::Hidden;
    };
    // spec: S-0516 claim 28 (fully cloaked: 50 % for the owner, a sensing viewer, a mutual ally; otherwise not drawn)
    if (v.is_some() && o.owner == v) || senses(w, v, id) || mutual(w, v, o.owner) {
        Look::Translucent50
    } else {
        Look::Hidden
    }
}

/// How *V* sees an object's cloak (S-0516 claim 28).
pub fn look(w: &World, id: ObjectId, v: Option<HouseId>) -> Look {
    let Some(o) = w.obj(id) else {
        return Look::Normal;
    };
    let owner = v.is_some() && o.owner == v;
    if o.kind == Kind::Building {
        let level = o.cloak.stage;
        return match o.cloak.state {
            CloakState::Uncloaked => Look::Normal,
            // spec: S-0516 claim 28 (buildings: levels 1–5 25 %, 6–10 50 %, 11–15 the fully cloaked rule)
            _ if level <= 0 => Look::Normal,
            _ if level <= 5 => Look::Translucent25, // spec: S-0516 claim 28 (levels 1–5)
            _ if level <= 10 => Look::Translucent50, // spec: S-0516 claim 28 (levels 6–10)
            _ => cloaked_look(w, id, v),
        };
    }
    match o.cloak.state {
        CloakState::Uncloaked => Look::Normal,
        CloakState::Cloaked => cloaked_look(w, id, v),
        // spec: S-0516 claim 28 (the fading table covers stage k > 0)
        CloakState::Cloaking | CloakState::Uncloaking if o.cloak.stage <= 0 => Look::Normal,
        CloakState::Cloaking | CloakState::Uncloaking => {
            let x = fade_fraction(o.cloak.stage, w.rules().cloak.stages);
            // spec: S-0516 claim 28 (fading by x: < 64 25 %; 64 to 191 50 %; 192 to 254 rippling, the owner 50 %; ≥ 255 not drawn, the owner 50 %; sensors play no part)
            if x < 64 {
                Look::Translucent25
            } else if x < 192 {
                Look::Translucent50
            } else if owner {
                Look::Translucent50
            } else if x < 255 {
                Look::Rippling
            } else {
                Look::Hidden
            }
        }
    }
}

/// Whether a vehicle's shadow is drawn (S-0516 claim 29): only while it is
/// uncloaked, for every viewer.
pub fn shadow(w: &World, id: ObjectId) -> bool {
    // spec: S-0516 claim 29 (no shadow while the cloak state is anything but uncloaked)
    w.obj(id)
        .is_none_or(|o| o.cloak.state == CloakState::Uncloaked)
}

/// Whether *V* can hover, click or pick an object (S-0516 claims 26 and
/// 36).
pub fn pickable(w: &World, id: ObjectId, v: Option<HouseId>) -> bool {
    let Some(o) = w.obj(id) else { return false };
    let sensed = senses(w, v, id);
    let owner = v.is_some() && o.owner == v;
    // spec: S-0516 claim 26 (a fully cloaked object V neither owns nor senses, allies included)
    if o.cloak.state == CloakState::Cloaked && !owner && !sensed {
        return false;
    }
    // spec: S-0516 claim 26 (a building at fade level 15 that V does not sense)
    if o.kind == Kind::Building && o.cloak.stage >= 15 && !sensed {
        return false;
    }
    // spec: S-0516 claim 36 (a unit disguised as a terrain object or overlay that appears disguised to V cannot be selected)
    if appears_disguised(w, id, v)
        && o.cloak
            .disguise
            .and_then(|d| d.what)
            .is_some_and(|d| matches!(d, DisguiseAs::Terrain(_) | DisguiseAs::Overlay(_)))
    {
        return false;
    }
    true
}

/// Whether a cloak event takes an object out of *V*'s selection (S-0516
/// claim 26).
pub fn deselects(w: &World, id: ObjectId, event: CloakEvent, v: Option<HouseId>) -> bool {
    let Some(o) = w.obj(id) else { return true };
    let owner = v.is_some() && o.owner == v;
    match event {
        // spec: S-0516 claim 26 (an object that starts cloaking leaves a non-owner's selection even when sensed)
        CloakEvent::Started => !owner,
        // spec: S-0516 claim 26 (a fully cloaked object re-evaluated leaves a non-owner's selection when not sensed)
        CloakEvent::Reevaluated => {
            !owner && o.cloak.state == CloakState::Cloaked && !senses(w, v, id)
        }
    }
}

/// Whether an object shows on *V*'s radar as far as cloak goes (S-0516
/// claim 27): another house's cloaked object only where *V* senses it,
/// allies included.
pub fn on_radar(w: &World, id: ObjectId, v: Option<HouseId>) -> bool {
    let Some(o) = w.obj(id) else { return false };
    if v.is_some() && o.owner == v {
        return true;
    }
    // spec: S-0516 claim 27 (another house's cloaked object appears only where V senses its cell)
    o.cloak.state != CloakState::Cloaked || senses(w, v, id)
}

/// The owner's view of its disguised unit by *t* (S-0516 Table B): whether
/// the disguise is shown, and the translucency.
pub fn owner_phase(t: i32) -> (bool, Look) {
    // spec: S-0516 Table B (t mod 256: 0–3 real 25 %, 4–7 real 50 %, 8–11 disguise 50 %, 12–15 disguise 25 %, 16–47 disguise, 48–51 disguise 25 %, 52–55 disguise 50 %, 56–59 real 50 %, 60–63 real 25 %, 64–255 real)
    match t.rem_euclid(256) {
        0..=3 => (false, Look::Translucent25),
        4..=7 => (false, Look::Translucent50),
        8..=11 => (true, Look::Translucent50),
        12..=15 => (true, Look::Translucent25),
        16..=47 => (true, Look::Normal),
        48..=51 => (true, Look::Translucent25),
        52..=55 => (true, Look::Translucent50),
        56..=59 => (false, Look::Translucent50),
        60..=63 => (false, Look::Translucent25),
        _ => (false, Look::Normal),
    }
}

/// What *V* is shown of a disguised unit (S-0516 claim 37): the disguise
/// to draw and the translucency, or `None` for the real unit.
pub fn shown_disguise(w: &World, id: ObjectId, v: Option<HouseId>) -> Option<(Disguise, Look)> {
    let o = w.obj(id)?;
    let d = o.cloak.disguise?;
    let owner = v.is_some() && o.owner == v;
    // spec: S-0516 claim 37 (a) (a running fake blink shows every non-owner the real unit)
    if !owner && w.tick < o.cloak.blink_until {
        return None;
    }
    if owner {
        // spec: S-0516 claim 37 (b) and F6 (t = (now − t_disguised) mod 256)
        let (show, look) = owner_phase(w.tick.wrapping_sub(d.since));
        return show.then_some((d, look));
    }
    // spec: S-0516 claim 37 (c) (others: the disguise unless V has detector coverage of its cell; allies too)
    (!detected(w, v, super::centre_cell(w, id))).then_some((d, Look::Normal))
}

/// The translucency the owner sees its disguised unit with while the real
/// unit is shown (Table B), or `None`.
pub fn owner_real_look(w: &World, id: ObjectId, v: Option<HouseId>) -> Option<Look> {
    let o = w.obj(id)?;
    let d = o.cloak.disguise?;
    if v.is_none() || o.owner != v {
        return None;
    }
    let (show, look) = owner_phase(w.tick.wrapping_sub(d.since));
    (!show).then_some(look)
}

/// The house whose colour a disguised unit shows on *V*'s radar (S-0516
/// claim 38): the owner's when the owner allies *V*, otherwise the disguise
/// house's. Q-0728 (interim: the owner's when the disguise house is none).
pub fn radar_house(w: &World, id: ObjectId, v: Option<HouseId>) -> Option<HouseId> {
    let o = w.obj(id)?;
    let Some(d) = o.cloak.disguise else {
        return o.owner;
    };
    // spec: S-0516 claim 38 (the owner's colour when the owner allies V, the one-way owner → viewer relation)
    if (v.is_some() && o.owner == v) || w.allied(o.owner, v) {
        return o.owner;
    }
    // spec: S-0516 claim 38 (otherwise the disguise house's colour)
    d.house.or(o.owner)
}

/// Whether the hover health bar shows for a disguised unit that appears
/// disguised to *V* (S-0516 claim 36: only when the disguise has a house).
pub fn hover_bar(w: &World, id: ObjectId, v: Option<HouseId>) -> bool {
    let Some(o) = w.obj(id) else { return false };
    if !appears_disguised(w, id, v) {
        return true;
    }
    // spec: S-0516 claim 36 (the hover health bar, shown only when the disguise has a house)
    o.cloak.disguise.is_some_and(|d| d.house.is_some())
}
