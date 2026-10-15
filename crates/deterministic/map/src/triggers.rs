//! The map's trigger data (S-0011 Import mapping; S-0009 claims 31 and 32),
//! as the importer resolves it: trigger and tag definitions by registration
//! index (S-0011 claim 7), every reference already resolved to an index or
//! to none, chains cut before a repeated member (claim 1), the cell tags in
//! entry order and the map trigger variables' initial states. The runtime is
//! the simulation's (`glasnost_sim::triggers`).

use serde::{Deserialize, Serialize};

/// Map trigger variables (S-0011 claim 22; S-0009 claim 32).
pub const VARIABLES: usize = 100; // spec: S-0011 Constants (map trigger variables usable by events and actions: 100)

/// The owner-country field of a trigger definition (S-0011 claim 5, I1).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Owner {
    /// No owner country (`<random>`, a created definition, or our engine's
    /// result for a name that names no country).
    #[default]
    None,
    /// `<none>`: the first country of the country list
    /// (`quirks.none_owner_is_first_country`, resolved by the simulation).
    NoneMarker,
    /// A country named by its type name or name field, matched without
    /// regard to letter case by the simulation, which knows the countries.
    Name(Vec<u8>),
}

/// One event of a trigger definition (S-0011 Terms, I1).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
    /// Event code (I3).
    pub code: i32,
    /// Parameter kind.
    pub kind: i32,
    /// The event's number (0 for kinds other than 0 and 2).
    pub value: i32,
    /// Type name (kind 2 only), cut to 24 bytes.
    pub type_name: Option<Vec<u8>>,
}

/// One action of a trigger definition (S-0011 Terms, I1, I2).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Action {
    /// Action code (I4).
    pub code: i32,
    /// Parameter kind (I2).
    pub kind: i32,
    /// The number slot (kinds 0, 11, 5 from the eighth item, 9 from the
    /// eighth item). Kinds 6, 7 and 8 fill it from a name lookup; that name
    /// is kept in `name`, since the sound, speech and theme lists are the
    /// presentation's.
    pub number: i32,
    /// The second number slot (kind 11 only).
    pub number2: i32,
    /// The fourth to seventh items, read as integers (S-0011 I2; the
    /// local-area rectangle of code 40, S-0524 I1).
    pub items: [i32; 4],
    /// The waypoint slot (F5 of the eighth item), −1 when it decodes to no
    /// letter; 0 (waypoint A) when the eighth item is missing.
    pub waypoint: i32,
    /// A trigger reference (kind 2) as a registration index.
    pub trigger: Option<u32>,
    /// A tag reference (kind 3) as a registration index.
    pub tag: Option<u32>,
    /// A team reference (kinds 1 and 5), as text: the team system is
    /// outside S-0011.
    pub team: Option<Vec<u8>>,
    /// The text label (kind 4): trimmed and cut to 31 bytes.
    pub text: Vec<u8>,
    /// The looked-up name (kinds 6, 7, 8: speech, sound, theme) or the type
    /// name (kinds 9, 10, cut to 24 bytes).
    pub name: Vec<u8>,
}

/// One trigger definition (S-0011 Terms, I1).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TriggerDef {
    /// The ID (the key).
    pub id: Vec<u8>,
    /// Owner country.
    pub owner: Owner,
    /// Linked trigger, by registration index.
    pub link: Option<u32>,
    /// Disabled flag.
    pub disabled: bool,
    /// Difficulty flags for positions 0, 1, 2.
    pub difficulty: [bool; 3], // spec: S-0011 I1 (fields 5, 6, 7: difficulty flags for positions 0, 1, 2)
    /// The carry-over flag (S-0011 I1 field 8; S-0524 claims 10 and 11).
    pub carry_over: bool,
    /// Events, in file order.
    pub events: Vec<Event>,
    /// Actions, in file order.
    pub actions: Vec<Action>,
}

impl TriggerDef {
    /// The empty definition a reference to an undefined ID creates (S-0011
    /// Edge cases, *References that create definitions*): no events, no
    /// actions, no owner country, not disabled, every difficulty flag set.
    pub fn created(id: Vec<u8>) -> Self {
        Self {
            id,
            owner: Owner::None,
            link: None,
            disabled: false,
            // spec: S-0011 Edge cases (a created trigger definition: all three difficulty flags set)
            difficulty: [true; 3],
            // spec: S-0011 I1 (field 8 absent: clear)
            carry_over: false,
            events: Vec::new(),
            actions: Vec::new(),
        }
    }
}

/// One tag definition (S-0011 Terms, Import mapping *Tags*).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagDef {
    /// The ID (the key).
    pub id: Vec<u8>,
    /// Repeat type (0 one-time, 1 all-carriers, 2 repeating; claim 10).
    pub repeat: i32,
    /// Name, cut to 48 bytes (matched by pre-placed objects' tag fields).
    pub name: Vec<u8>,
    /// The first trigger, by registration index.
    pub first: Option<u32>,
    /// The chain (claim 1): the first trigger, then each member's linked
    /// trigger, cut before the first repeated member.
    pub chain: Vec<u32>,
}

/// The map's trigger data.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Triggers {
    /// Trigger definitions by registration index.
    pub triggers: Vec<TriggerDef>,
    /// Tag definitions by registration index (claim 7).
    pub tags: Vec<TagDef>,
    /// Cell tags in entry order (S-0009 claim 31): each cell once, with
    /// the tag definition it carries.
    pub cell_tags: Vec<((i16, i16), u32)>,
    /// The map trigger variables' initial states set by the map (S-0009
    /// claim 32); `None` leaves the state as it was.
    pub variables: Vec<Option<bool>>,
}

impl Triggers {
    /// The tag definition a pre-placed object's tag field names (S-0011
    /// claim 2): the first in registration order whose ID or name equals the
    /// field without regard to letter case; none creates nothing.
    pub fn object_tag(&self, field: &[u8]) -> Option<u32> {
        self.tags
            .iter()
            .position(|t| t.id.eq_ignore_ascii_case(field) || t.name.eq_ignore_ascii_case(field))
            .map(|i| i as u32)
    }
}

/// Formulas F5 of S-0011: the waypoint number of a waypoint parameter's
/// letters, −1 when the first character is not a letter.
pub fn waypoint_letters(text: &[u8]) -> i32 {
    // spec: S-0011 F5 (L(c): A = 0 … Z = 25, upper and lower case alike)
    let l = |c: Option<&u8>| {
        c.filter(|c| c.is_ascii_alphabetic())
            .map(|c| i32::from(c.to_ascii_uppercase() - b'A'))
    };
    match (l(text.first()), l(text.get(1))) {
        // spec: S-0011 F5 (w = −1 when c₁ is not a letter)
        (None, _) => -1,
        // spec: S-0011 F5 (w = L(c₁) when only c₁ is a letter)
        (Some(a), None) => a,
        // spec: S-0011 F5 (w = 26 × L(c₁) + L(c₂) + 26 when both are letters)
        (Some(a), Some(b)) => 26 * a + b + 26,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // spec: S-0011 F5 (A … Z give 0 … 25, AA … AZ give 26 … 51, ZZ gives 701)
    #[test]
    fn waypoint_letters_f5() {
        assert_eq!(waypoint_letters(b"A"), 0);
        assert_eq!(waypoint_letters(b"z"), 25);
        assert_eq!(waypoint_letters(b"AA"), 26);
        assert_eq!(waypoint_letters(b"aZ"), 51);
        assert_eq!(waypoint_letters(b"ZZ"), 701);
        assert_eq!(waypoint_letters(b"B1"), 1);
        assert_eq!(waypoint_letters(b"ABC"), 27);
        assert_eq!(waypoint_letters(b"1A"), -1);
        assert_eq!(waypoint_letters(b""), -1);
    }

    // spec: S-0011 claim 2 (an object's tag field matches the ID or the name, first in registration order)
    #[test]
    fn object_tag_matches_id_or_name() {
        let tag = |id: &[u8], name: &[u8]| TagDef {
            id: id.to_vec(),
            repeat: 0,
            name: name.to_vec(),
            first: None,
            chain: Vec::new(),
        };
        let t = Triggers {
            tags: vec![tag(b"01000001", b"Alpha"), tag(b"alpha", b"Beta")],
            ..Default::default()
        };
        assert_eq!(t.object_tag(b"ALPHA"), Some(0));
        assert_eq!(t.object_tag(b"beta"), Some(1));
        assert_eq!(t.object_tag(b"none"), None);
    }
}
