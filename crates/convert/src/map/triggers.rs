//! The map's triggers, events, actions, tags, cell tags and trigger
//! variables (S-0011 Import mapping, I1 and I2; S-0009 claims 31 and 32),
//! resolved into [`glasnost_map::triggers::Triggers`]: every definition by
//! registration index (S-0011 claim 7), every reference resolved, the chains
//! built and cut (claim 1). The importer's warnings and safe fallbacks are
//! here; the simulation receives indexed data only.

use glasnost_ini::value;
use glasnost_map::triggers::{
    Action, Event, Owner, TagDef, TriggerDef, Triggers, VARIABLES, waypoint_letters,
};

use super::{Decoder, show};

/// Bytes of a tag value that are read (S-0011 Import mapping).
const TAG_VALUE_LIMIT: usize = 127; // spec: S-0011 Import mapping (a tag value has a text limit of 127 bytes)
/// Bytes of an event's or an action's type name (S-0011 Constants).
const TYPE_NAME_LIMIT: usize = 24; // spec: S-0011 Constants (event type-name length: 24 bytes)
/// Bytes of an action's text label after trimming (S-0011 Constants).
const ACTION_TEXT_LIMIT: usize = 31; // spec: S-0011 Constants (action text length: 31 bytes, after trimming)
/// Bytes of a tag name (S-0011 Constants).
const TAG_NAME_LIMIT: usize = 48; // spec: S-0011 Constants (tag names: 48 bytes)
/// Items of one action (S-0011 Import mapping, *Actions*).
const ACTION_ITEMS: usize = 8; // spec: S-0011 Import mapping (actions of 8 items each)
/// Items an action needs before its last (S-0011 Edge cases).
const ACTION_REQUIRED: usize = 7; // spec: S-0011 Edge cases (an action that ends before its seventh item is unparseable)
/// Fields of a trigger entry below which the importer warns (S-0011 Edge
/// cases, *Missing disabled field*).
const TRIGGER_FIELDS_WARN: usize = 7; // spec: S-0011 Edge cases (warns about a trigger entry with fewer than seven fields)
/// Variable-name entries read (S-0009 claim 32).
const VARIABLE_ENTRIES: usize = 100; // spec: S-0009 Constants (variable-name entries read: the first 100)
/// Longest specified variable name (S-0009 claim 32).
const VARIABLE_NAME_LIMIT: usize = 39; // spec: S-0009 Constants (longest specified variable name: 39 characters)

/// Splits a value into items at commas, skipping empty items (S-0011
/// Import mapping); `empty` counts the skipped ones, for the warning.
fn items(v: &[u8]) -> (Vec<Vec<u8>>, usize) {
    let mut empty = 0;
    let mut out = Vec::new();
    for it in v.split(|&b| b == b',') {
        if it.is_empty() {
            empty += 1;
        } else {
            out.push(it.to_vec());
        }
    }
    // A value ending in no comma has no empty last item; one ending in a comma does.
    (out, empty)
}

/// The `<none>` and `none` markers, in any letter case (S-0011 I1, I2).
fn none_marker(t: &[u8]) -> bool {
    // spec: S-0011 I2 (`<none>` or `none`, in any letter case, names nothing)
    t.eq_ignore_ascii_case(b"<none>") || t.eq_ignore_ascii_case(b"none")
}

/// An integer item (S-0011 Import mapping: S-0003 claim 3's decimal form only).
fn int(t: &[u8]) -> i32 {
    // spec: S-0011 Import mapping (integer items: the decimal form of S-0003 claim 3 only)
    value::decimal_integer(t)
}

fn cut(t: &[u8], n: usize) -> Vec<u8> {
    t[..t.len().min(n)].to_vec()
}

/// The registries while they are built.
struct Build {
    triggers: Vec<TriggerDef>,
    tags: Vec<TagDef>,
    /// Tag definitions that the tag list defined (the rest are created).
    tag_defined: Vec<bool>,
}

impl Build {
    fn trigger_by_id(&self, id: &[u8]) -> Option<u32> {
        self.triggers
            .iter()
            .position(|t| t.id.eq_ignore_ascii_case(id))
            .map(|i| i as u32)
    }

    fn tag_by_id(&self, id: &[u8]) -> Option<u32> {
        self.tags
            .iter()
            .position(|t| t.id.eq_ignore_ascii_case(id))
            .map(|i| i as u32)
    }

    /// The trigger an ID names, created when undefined (S-0011 Edge cases,
    /// *References that create definitions*); `created` reports it.
    fn trigger_or_create(&mut self, id: &[u8], created: &mut bool) -> u32 {
        if let Some(i) = self.trigger_by_id(id) {
            return i;
        }
        *created = true;
        self.triggers.push(TriggerDef::created(id.to_vec()));
        (self.triggers.len() - 1) as u32
    }

    /// The tag an ID names, registered (with an empty chain) when not yet
    /// registered (S-0011 claim 7).
    fn tag_or_register(&mut self, id: &[u8]) -> u32 {
        if let Some(i) = self.tag_by_id(id) {
            return i;
        }
        self.tags.push(TagDef {
            id: id.to_vec(),
            repeat: 0,
            name: Vec::new(),
            first: None,
            chain: Vec::new(),
        });
        self.tag_defined.push(false);
        (self.tags.len() - 1) as u32
    }
}

/// An action reference as read (S-0011 I2): none, a short registration
/// index, or an ID.
enum Ref {
    None,
    Index(i32),
    Id(Vec<u8>),
}

fn reference(t: &[u8]) -> Ref {
    // spec: S-0011 I2 (a decimal integer prefix −1, regardless of length, names nothing)
    if int(t) == -1 {
        return Ref::None;
    }
    // spec: S-0011 I2 and Edge cases (shorter than 3 characters: the decimal-prefix registration index, 0 if no digits)
    if t.len() < 3 {
        return Ref::Index(int(t));
    }
    if none_marker(t) {
        return Ref::None;
    }
    Ref::Id(t.to_vec())
}

impl Decoder<'_> {
    /// The trigger data of the map.
    pub(super) fn triggers(&mut self, cell_tags: &[((i16, i16), Vec<u8>)]) -> Triggers {
        let mut b = Build {
            triggers: Vec::new(),
            tags: Vec::new(),
            tag_defined: Vec::new(),
        };
        // The trigger list's entries, in entry order, each ID once.
        let mut entries: Vec<(Vec<u8>, Vec<u8>, u32)> = Vec::new();
        if let Some(section) = self.doc.section(b"Triggers") {
            for (entry, winner) in section.list_positions() {
                if entries
                    .iter()
                    .any(|(k, _, _)| k.eq_ignore_ascii_case(&entry.key))
                {
                    continue;
                }
                if none_marker(&entry.key) {
                    // spec: S-0011 Edge cases (a trigger list key written as the "none" marker: our engine skips the entry and warns)
                    self.warn(
                        Some(entry.line),
                        "[Triggers]: a key written as the none marker is skipped (S-0011 Edge cases, a departure)",
                    );
                    continue;
                }
                entries.push((entry.key.clone(), winner.value.clone(), entry.line));
            }
        }
        // Registration: the trigger list's IDs, then created definitions.
        for (id, _, _) in &entries {
            b.triggers.push(TriggerDef::created(id.clone()));
        }
        // Short trigger and tag references are resolved once the registries are complete.
        let mut short_triggers: Vec<(usize, usize, i32)> = Vec::new();
        let mut short_tags: Vec<(usize, usize, i32)> = Vec::new();
        for (ti, (id, v, line)) in entries.clone().into_iter().enumerate() {
            let line = Some(line);
            let (f, empty) = items(&v);
            if empty > 0 {
                // spec: S-0011 Import mapping (the importer warns about every empty item)
                self.warn(
                    line,
                    format!("[Triggers] {}: empty items are skipped", show(&id)),
                );
            }
            if f.len() < TRIGGER_FIELDS_WARN {
                // spec: S-0011 Edge cases (the importer warns about a trigger entry with fewer than seven fields)
                self.warn(
                    line,
                    format!("[Triggers] {}: fewer than seven fields", show(&id)),
                );
            }
            let owner = match f.first() {
                // spec: S-0011 claim 5 (`<none>` in any letter case: the first country, resolved by the simulation)
                Some(o) if o.eq_ignore_ascii_case(b"<none>") => {
                    self.warn(
                        line,
                        format!(
                            "[Triggers] {}: owner <none> gives the first country (S-0011 claim 5)",
                            show(&id)
                        ),
                    );
                    Owner::NoneMarker
                }
                // spec: S-0011 claim 5 (`<random>` in any letter case names no country)
                Some(o) if o.eq_ignore_ascii_case(b"<random>") => {
                    self.warn(
                        line,
                        format!(
                            "[Triggers] {}: owner <random> names no country (S-0011 claim 5)",
                            show(&id)
                        ),
                    );
                    Owner::None
                }
                Some(o) => Owner::Name(o.clone()),
                None => Owner::None,
            };
            let link = match f.get(1) {
                None => {
                    // spec: S-0011 I1 (field 2 absent: an unparseable entry; our engine: no link)
                    self.warn(
                        line,
                        format!(
                            "[Triggers] {}: no linked-trigger field; no link (S-0011 Edge cases, a departure)",
                            show(&id)
                        ),
                    );
                    None
                }
                Some(l) if none_marker(l) => None,
                // spec: S-0011 I2 (the linked trigger uses the ID lookup regardless of length)
                Some(l) => {
                    let mut created = false;
                    let i = b.trigger_or_create(l, &mut created);
                    if created {
                        self.warn(
                            line,
                            format!(
                                "[Triggers] {}: linked trigger {} is not defined; an empty one is created (S-0011 Edge cases)",
                                show(&id),
                                show(l)
                            ),
                        );
                    }
                    Some(i)
                }
            };
            // spec: S-0011 I1 (field 4 absent: disabled) and Edge cases (missing disabled field)
            let disabled = f.get(3).is_none_or(|d| int(d) != 0);
            // spec: S-0011 I1 (fields 5 … 7 absent: clear)
            let difficulty = [4, 5, 6].map(|i| f.get(i).is_some_and(|d| int(d) != 0));
            // spec: S-0011 I1 (field 8, the carry-over flag: non-zero = set; absent: clear)
            let carry_over = f.get(7).is_some_and(|d| int(d) != 0);
            let events = self.events(&id);
            let mut actions = Vec::new();
            for (ai, (a, refs)) in self.actions(&id).into_iter().enumerate() {
                let mut a = a;
                match refs {
                    (2, Ref::Index(n)) => short_triggers.push((ti, ai, n)),
                    (2, Ref::Id(r)) => {
                        let mut created = false;
                        a.trigger = Some(b.trigger_or_create(&r, &mut created));
                        if created {
                            self.warn(
                                line,
                                format!(
                                    "[Actions] {}: trigger {} is not defined; an empty one is created (S-0011 Edge cases)",
                                    show(&id),
                                    show(&r)
                                ),
                            );
                        }
                    }
                    // spec: S-0011 claim 7 (an action parameter that gives a tag ID registers that tag definition, in trigger-list order)
                    (3, Ref::Index(n)) => short_tags.push((ti, ai, n)),
                    (3, Ref::Id(r)) => a.tag = Some(b.tag_or_register(&r)),
                    (_, Ref::Index(_)) | (_, Ref::Id(_)) | (_, Ref::None) => {}
                }
                actions.push(a);
            }
            let t = &mut b.triggers[ti];
            t.owner = owner;
            t.link = link;
            t.disabled = disabled;
            t.difficulty = difficulty;
            t.carry_over = carry_over;
            t.events = events;
            t.actions = actions;
        }
        // spec: S-0011 claim 7 (the tag list then registers the rest in its entry order)
        self.tag_list(&mut b);
        // spec: S-0009 claim 31 and S-0011 claim 7 (a cell tag naming an unregistered ID registers an empty tag definition)
        let mut cells = Vec::new();
        for (c, v) in cell_tags {
            // spec: S-0011 Import mapping (a cell-tag value is matched against tag IDs by the ID-only rule)
            let i = match b.tag_by_id(v) {
                Some(i) => i,
                None => {
                    self.warn(
                        None,
                        format!(
                            "[CellTags] {}: names no defined tag; an empty tag is attached (S-0009 claim 31)",
                            show(v)
                        ),
                    );
                    b.tag_or_register(v)
                }
            };
            cells.push((*c, i));
        }
        // Short references: registration indexes into the complete registries.
        for (ti, ai, n) in short_triggers {
            let id = b.triggers[ti].id.clone();
            let r = usize::try_from(n).ok().filter(|&i| i < b.triggers.len());
            self.short_warning(&id, n, r.is_some());
            b.triggers[ti].actions[ai].trigger = r.map(|i| i as u32);
        }
        for (ti, ai, n) in short_tags {
            let id = b.triggers[ti].id.clone();
            let r = usize::try_from(n).ok().filter(|&i| i < b.tags.len());
            self.short_warning(&id, n, r.is_some());
            b.triggers[ti].actions[ai].tag = r.map(|i| i as u32);
        }
        for (i, defined) in b.tag_defined.iter().enumerate() {
            if !defined && !cells.iter().any(|&(_, t)| t as usize == i) {
                // spec: S-0011 Edge cases (a tag reference giving an undefined tag ID creates an empty tag definition; the importer warns)
                self.warn(
                    None,
                    format!(
                        "[Actions]: tag {} is not defined; an empty tag definition is created (S-0011 Edge cases)",
                        show(&b.tags[i].id)
                    ),
                );
            }
        }
        // spec: S-0011 claim 1 (the chain: the first trigger, then each member's linked trigger; our engine cuts it before the first repeated member)
        for ti in 0..b.tags.len() {
            let mut chain: Vec<u32> = Vec::new();
            let mut next = b.tags[ti].first;
            while let Some(m) = next {
                if chain.contains(&m) {
                    self.warn(
                        None,
                        format!(
                            "[Tags] {}: the chain comes back to {}; it ends before it (S-0011 claim 1, a departure)",
                            show(&b.tags[ti].id),
                            show(&b.triggers[m as usize].id)
                        ),
                    );
                    break;
                }
                chain.push(m);
                next = b.triggers[m as usize].link;
            }
            b.tags[ti].chain = chain;
        }
        Triggers {
            triggers: b.triggers,
            tags: b.tags,
            cell_tags: cells,
            variables: self.variables(),
        }
    }

    fn short_warning(&mut self, id: &[u8], n: i32, ok: bool) {
        // spec: S-0011 Edge cases (the importer warns about the short-index form; an index outside the registry is empty, a departure)
        self.warn(
            None,
            if ok {
                format!(
                    "[Actions] {}: a reference shorter than 3 characters is registration index {n} (S-0011 Edge cases)",
                    show(id)
                )
            } else {
                format!(
                    "[Actions] {}: registration index {n} is outside the registry; no reference (S-0011 Edge cases, a departure)",
                    show(id)
                )
            },
        );
    }

    /// The events of trigger `id` (S-0011 Import mapping, *Events*).
    fn events(&mut self, id: &[u8]) -> Vec<Event> {
        let Some(e) = self.doc.get(b"Events", id) else {
            return Vec::new();
        };
        let line = Some(e.line);
        let (f, empty) = items(&e.value.clone());
        if empty > 0 {
            // spec: S-0011 Import mapping (the importer warns about every empty item)
            self.warn(
                line,
                format!("[Events] {}: empty items are skipped", show(id)),
            );
        }
        let n = f.first().map_or(0, |n| int(n));
        let mut out = Vec::new();
        let mut i = 1;
        while out.len() < usize::try_from(n).unwrap_or(0) {
            // spec: S-0011 Edge cases (an event that ends before its value item: our engine reads only the complete events)
            if i + 3 > f.len() {
                break;
            }
            let code = int(&f[i]);
            let kind = int(&f[i + 1]);
            let v = int(&f[i + 2]);
            i += 3;
            let ev = match kind {
                // spec: S-0011 Import mapping (kind 0: the value as the event's number)
                0 => Event {
                    code,
                    kind,
                    value: v,
                    type_name: None,
                },
                // spec: S-0011 Import mapping (kind 2: the value as the number, and the fourth item as the type name cut to 24 bytes)
                2 => {
                    let Some(name) = f.get(i) else { break };
                    i += 1;
                    Event {
                        code,
                        kind,
                        value: v,
                        type_name: Some(cut(name, TYPE_NAME_LIMIT)),
                    }
                }
                // spec: S-0011 Import mapping (kind 1 and other kinds: the event's number is 0)
                _ => Event {
                    code,
                    kind,
                    value: 0,
                    type_name: None,
                },
            };
            out.push(ev);
        }
        if n < 0 || out.len() < usize::try_from(n).unwrap_or(0) {
            // spec: S-0011 Edge cases (an event count that is negative or larger than the values that follow: our engine reads only the complete events and warns)
            self.warn(
                line,
                format!(
                    "[Events] {}: count {n} but {} complete events (S-0011 Edge cases, a departure)",
                    show(id),
                    out.len()
                ),
            );
        }
        out
    }

    /// The actions of trigger `id` (S-0011 Import mapping, *Actions*, I2),
    /// each with its unresolved reference: (kind, reference).
    fn actions(&mut self, id: &[u8]) -> Vec<(Action, (i32, Ref))> {
        let Some(e) = self.doc.get(b"Actions", id) else {
            return Vec::new();
        };
        let line = Some(e.line);
        let (f, empty) = items(&e.value.clone());
        if empty > 0 {
            // spec: S-0011 Import mapping (the importer warns about every empty item)
            self.warn(
                line,
                format!("[Actions] {}: empty items are skipped", show(id)),
            );
        }
        let n = f.first().map_or(0, |n| int(n));
        let want = usize::try_from(n).unwrap_or(0);
        let mut out = Vec::new();
        let mut i = 1;
        while out.len() < want {
            // spec: S-0011 Edge cases (an action that ends before its seventh item: our engine reads only the complete actions)
            if i + ACTION_REQUIRED > f.len() {
                break;
            }
            let item = |k: usize| f.get(i + k).cloned();
            // spec: S-0011 Import mapping (only the eighth item may be missing, and only in the last action)
            let last = out.len() + 1 == want;
            let eighth = if last || i + ACTION_ITEMS <= f.len() {
                item(ACTION_ITEMS - 1)
            } else {
                None
            };
            let code = int(&f[i]);
            let kind = int(&f[i + 1]);
            let v = f[i + 2].clone();
            // spec: S-0011 I2 (the fourth to seventh items are read as integers; S-0524 I1, code 40's rectangle)
            let items = [3, 4, 5, 6].map(|k| f.get(i + k).map_or(0, |t| int(t)));
            i += ACTION_ITEMS;
            let mut a = Action {
                code,
                kind,
                items,
                ..Default::default()
            };
            // spec: S-0011 I2 (a missing eighth item leaves the waypoint slot at 0, waypoint A)
            let letters = |e: &Option<Vec<u8>>| e.as_deref().map_or(0, waypoint_letters);
            let mut r = Ref::None;
            match kind {
                // spec: S-0011 I2 (kinds 0 and 11: the value as an integer into the number slot; kind 0's eighth item as waypoint letters, kind 11's as an integer into the second number slot)
                0 => {
                    a.number = int(&v);
                    a.waypoint = letters(&eighth);
                }
                11 => {
                    a.number = int(&v);
                    a.number2 = eighth.as_deref().map_or(0, int);
                }
                // spec: S-0011 I2 (kinds 1 and 5: a team reference; kind 1's eighth item as waypoint letters, kind 5's as an integer into the number slot)
                1 | 5 => {
                    a.team = match reference(&v) {
                        Ref::None => None,
                        _ => Some(v.clone()),
                    };
                    if kind == 1 {
                        a.waypoint = letters(&eighth);
                    } else {
                        a.number = eighth.as_deref().map_or(0, int);
                    }
                }
                // spec: S-0011 I2 (kinds 2 and 3: a trigger or tag reference; the eighth item as waypoint letters)
                2 | 3 => {
                    r = reference(&v);
                    a.waypoint = letters(&eighth);
                }
                // spec: S-0011 I2 (kind 4: a text label, characters with codes 1 … 32 dropped at both ends, cut to 31 bytes; −1 = empty)
                4 => {
                    if int(&v) != -1 {
                        let s = v
                            .iter()
                            .position(|&c| !(1..=32).contains(&c))
                            .unwrap_or(v.len());
                        let e = v
                            .iter()
                            .rposition(|&c| !(1..=32).contains(&c))
                            .map_or(s, |p| p + 1);
                        a.text = cut(&v[s..e.max(s)], ACTION_TEXT_LIMIT);
                    }
                    a.waypoint = letters(&eighth);
                }
                // spec: S-0011 I2 (kinds 6, 7, 8: a speech, sound or theme name looked up into the number slot; the presentation holds those lists, so the name is kept)
                6..=8 => {
                    a.name = v.clone();
                    a.number = -1;
                    a.waypoint = letters(&eighth);
                }
                // spec: S-0011 I2 (kinds 9 and 10: a type name cut to 24 bytes; kind 9's eighth item as an integer into the number slot, kind 10's as waypoint letters)
                9 => {
                    a.name = cut(&v, TYPE_NAME_LIMIT);
                    a.number = eighth.as_deref().map_or(0, int);
                }
                10 => {
                    a.name = cut(&v, TYPE_NAME_LIMIT);
                    a.waypoint = letters(&eighth);
                }
                // Kinds outside I2: the value as an integer, the eighth item as waypoint letters (no covered code reads them).
                _ => {
                    a.number = int(&v);
                    a.waypoint = letters(&eighth);
                }
            }
            out.push((a, (kind, r)));
        }
        if n < 0 || out.len() < want {
            // spec: S-0011 Edge cases (an action count that is negative or larger than the values that follow: our engine reads only the complete actions and warns)
            self.warn(
                line,
                format!(
                    "[Actions] {}: count {n} but {} complete actions (S-0011 Edge cases, a departure)",
                    show(id),
                    out.len()
                ),
            );
        }
        out
    }

    /// The tag list (S-0011 Import mapping, *Tags*).
    fn tag_list(&mut self, b: &mut Build) {
        let Some(section) = self.doc.section(b"Tags") else {
            return;
        };
        let list: Vec<(Vec<u8>, Vec<u8>, u32)> = section
            .list_positions()
            .map(|(e, w)| (e.key.clone(), w.value.clone(), e.line))
            .collect();
        for (key, v, line) in list {
            let line = Some(line);
            // spec: S-0011 Import mapping (a key whose value is empty is skipped)
            if v.is_empty() {
                continue;
            }
            let v = cut(&v, TAG_VALUE_LIMIT);
            let (f, empty) = items(&v);
            if empty > 0 {
                // spec: S-0011 Import mapping (the importer warns about every empty item)
                self.warn(
                    line,
                    format!("[Tags] {}: empty items are skipped", show(&key)),
                );
            }
            if f.len() < 3 {
                // spec: S-0011 Edge cases (a tag entry that ends before its third item: our engine skips it and warns)
                self.warn(
                    line,
                    format!("[Tags] {}: fewer than three items; skipped (S-0011 Edge cases, a departure)", show(&key)),
                );
                continue;
            }
            let i = b.tag_or_register(&key) as usize;
            if b.tag_defined[i] {
                continue;
            }
            let first = if none_marker(&f[2]) {
                None
            } else {
                // spec: S-0011 I2 (a tag's first trigger uses the ID lookup regardless of length)
                let mut created = false;
                let t = b.trigger_or_create(&f[2], &mut created);
                if created {
                    self.warn(
                        line,
                        format!(
                            "[Tags] {}: first trigger {} is not defined; an empty one is created (S-0011 Edge cases)",
                            show(&key),
                            show(&f[2])
                        ),
                    );
                }
                Some(t)
            };
            let repeat = int(&f[0]);
            if !(0..=2).contains(&repeat) {
                // spec: S-0011 claim 10 (any other repeat value never fires, and the importer warns)
                self.warn(
                    line,
                    format!(
                        "[Tags] {}: repeat type {repeat} never fires (S-0011 claim 10)",
                        show(&key)
                    ),
                );
            }
            let t = &mut b.tags[i];
            t.repeat = repeat;
            // spec: S-0011 Import mapping (the tag name, cut to 48 bytes)
            t.name = cut(&f[1], TAG_NAME_LIMIT);
            t.first = first;
            b.tag_defined[i] = true;
        }
    }

    /// The map trigger variables' initial states (S-0009 claim 32).
    fn variables(&mut self) -> Vec<Option<bool>> {
        let mut out = vec![None; VARIABLES];
        let Some(section) = self.doc.section(b"VariableNames") else {
            return out;
        };
        let list: Vec<(Vec<u8>, Vec<u8>, u32)> = section
            .list_positions()
            .map(|(e, w)| (e.key.clone(), w.value.clone(), e.line))
            .collect();
        // spec: S-0009 claim 32 (only the first 100 entries, in entry order, take effect)
        for (key, v, line) in list.into_iter().take(VARIABLE_ENTRIES) {
            let line = Some(line);
            // spec: S-0009 claim 32 (the key read leniently as its leading integer is the variable's number)
            let n = value::decimal_integer(&key);
            let f: Vec<&[u8]> = v.split(|&c| c == b',').collect();
            if f.iter().all(|i| i.is_empty()) {
                // spec: S-0009 claim 32 (a value holding no item at all: no effect, and the importer warns)
                self.warn(
                    line,
                    format!(
                        "[VariableNames] {}: no item; no effect (S-0009 claim 32)",
                        show(&key)
                    ),
                );
                continue;
            }
            let Some(slot) = usize::try_from(n).ok().filter(|&i| i < VARIABLES) else {
                // spec: S-0009 claim 32 (a variable number outside 0 … 99: no effect, and the importer warns)
                self.warn(
                    line,
                    format!("[VariableNames] {}: variable {n} lies outside 0 … 99; no effect (S-0009 claim 32)", show(&key)),
                );
                continue;
            };
            if f[0].len() > VARIABLE_NAME_LIMIT {
                // spec: S-0009 claim 32 (a name longer than 39 characters is cut, and the importer warns)
                self.warn(line, format!("[VariableNames] {}: name longer than 39 characters is cut (S-0009 claim 32)", show(&key)));
            }
            // spec: S-0009 claim 32 (the second item, a leading integer, gives the initial state: set when not 0)
            if let Some(s) = f.get(1) {
                out[slot] = Some(value::decimal_integer(s) != 0);
            }
        }
        out
    }
}
