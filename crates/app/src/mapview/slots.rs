//! Building slot animations as the game's set-up starts them (S-0108
//! claims 10 and 11, Terms and Import mapping), drawn as animations of
//! their own (S-0102 claim 17, S-0109 claims 1, 8, 10, 12, 19 and 20).
//! Only the initial image frame is drawn: animation playback is not
//! implemented, so no slot animation advances.

use glasnost_ini::Document;
use glasnost_ini::value;

/// The art keys of the slots a building placed during the game's set-up
/// starts: *idle* and *active-1* … *active-4*, in that order.
// spec: S-0108 claim 11 (set-up starts idle and active-1 … active-4); Import mapping (base keys)
pub const SETUP_SLOTS: [&[u8]; 5] = [
    b"IdleAnim",
    b"ActiveAnim",
    b"ActiveAnimTwo",
    b"ActiveAnimThree",
    b"ActiveAnimFour",
];

/// Bytes a slot's animation names are cut to (S-0108 Import mapping).
const NAME_LIMIT: usize = 15; // spec: S-0108 Import mapping (names cut to 15 bytes)

/// One slot's definition, as one read of the building's art section
/// leaves it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SlotDef {
    /// Normal animation name.
    pub normal: Vec<u8>,
    /// Damaged name.
    pub damaged: Vec<u8>,
    /// Occupied name.
    pub occupied: Vec<u8>,
    /// Screen displacement in pixels.
    pub x: i32,
    /// Screen displacement in pixels.
    pub y: i32,
    /// Depth adjustment (S-0102 claim 17).
    pub z_adjust: i32,
    /// Y-sort (S-0102 claim 17).
    pub y_sort: i32,
}

fn key(base: &[u8], suffix: &[u8]) -> Vec<u8> {
    [base, suffix].concat()
}

/// Reads the slot with base key `base` from the art section `image`
/// (S-0108 Import mapping).
pub fn read_slot(art: &Document, image: &[u8], base: &[u8]) -> SlotDef {
    let name = |suffix: &[u8]| {
        art.value(image, &key(base, suffix))
            .map(|v| value::text(v, NAME_LIMIT).to_vec())
            .unwrap_or_default()
    };
    let normal = name(b"");
    let mut damaged = name(b"Damaged");
    let mut occupied = name(b"Garrisoned");
    // spec: S-0108 Import mapping (an empty damaged or occupied name takes the normal name)
    if damaged.is_empty() {
        damaged.clone_from(&normal);
    }
    if occupied.is_empty() {
        occupied.clone_from(&normal);
    }
    let mut def = SlotDef {
        normal,
        damaged,
        occupied,
        ..SlotDef::default()
    };
    // spec: S-0108 Import mapping (the other keys take effect only when a name is non-empty;
    // defaults 0)
    if !def.normal.is_empty() || !def.damaged.is_empty() || !def.occupied.is_empty() {
        let int = |suffix: &[u8]| {
            art.value(image, &key(base, suffix))
                .and_then(value::integer)
                .unwrap_or(0)
        };
        def.x = int(b"X");
        def.y = int(b"Y");
        def.z_adjust = int(b"ZAdjust");
        def.y_sort = int(b"YSort");
    }
    def
}

/// The name a start uses (S-0108 Terms, *variant choice*).
pub fn variant(def: &SlotDef, damaged: bool, occupied: bool, allow_occupied: bool) -> &[u8] {
    // spec: S-0108 Terms (damaged: damaged name; not damaged, occupied, occupied allowed:
    // occupied name; otherwise the normal name)
    if damaged {
        &def.damaged
    } else if occupied && allow_occupied {
        &def.occupied
    } else {
        &def.normal
    }
}

/// A render layer (S-0102 claim 3; glossary *render layer*), in drawing
/// order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum Layer {
    /// Underground.
    Underground,
    /// Surface.
    Surface,
    /// Ground.
    Ground,
    /// Air.
    #[default]
    Air,
    /// Top.
    Top,
}

/// The drawing fields of an animation type (S-0109 and S-0102 Import
/// mapping; S-0103 Import mapping).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnimType {
    /// Start *a*.
    pub start: i32,
    /// End field as given.
    pub end: Option<i32>,
    /// Loop end as given.
    pub loop_end: Option<i32>,
    /// Reversed.
    pub reverse: bool,
    /// Shadow type.
    pub shadow: bool,
    /// Drawn flat.
    pub flat: bool,
    /// Drawn tiled (stacked).
    pub tiled: bool,
    /// Layer of an unattached animation.
    pub layer: Layer,
    /// Vertical draw shift in pixels.
    pub y_draw_offset: i32,
    /// Detail field.
    pub detail: i32,
    /// Translucent flag.
    pub translucent: bool,
    /// Translucency field.
    pub translucency: i32,
    /// Translucency detail field.
    pub translucency_detail: i32,
    /// Alternative palette flag.
    pub alt_palette: bool,
    /// Drawn with its building (palette row 3 of S-0103 claim 8).
    pub cell_drawer: bool,
    /// Drawn at brightness 1000.
    pub normal_light: bool,
    /// New-theater type.
    pub new_theater: bool,
    /// Rate field ρ as given (S-0109 claim 2).
    pub rate: Option<i32>,
    /// Normalised to the game speed (S-0109 claim 4).
    pub normalized: bool,
    /// Loop start λ<sub>s</sub>.
    pub loop_start: i32,
    /// Loop count *L*.
    pub loop_count: i32,
    /// Ping-pong flag (S-0109 claim 9).
    pub ping_pong: bool,
    /// Random loop delay pair, in updates (S-0109 claim 11).
    pub random_loop_delay: (i32, i32),
    /// Follow-up type name (S-0109 claim 15).
    pub next: Option<Vec<u8>>,
    /// Animation depth adjustment (S-0102 claim 13).
    pub z_adjust: i32,
    /// Animation sort adjustment (S-0102 claim 5).
    pub y_sort_adjust: i32,
}

/// Reads an animation type's playback and drawing fields from `image`, which
/// must be its own art section, the section named by its type name, whatever
/// section its image name names (S-0007 claim 34).
pub fn read_anim(art: &Document, image: &[u8]) -> AnimType {
    let int = |k: &[u8]| art.value(image, k).and_then(value::integer);
    let flag = |k: &[u8], d: bool| art.value(image, k).and_then(value::boolean).unwrap_or(d);
    // spec: S-0102 Import mapping (Layer: underground, surface, ground, air, top; absent air)
    let layer = art.value(image, b"Layer").map_or(Layer::Air, |v| {
        let v = value::text(v, usize::MAX);
        [
            (&b"underground"[..], Layer::Underground),
            (b"surface", Layer::Surface),
            (b"ground", Layer::Ground),
            (b"air", Layer::Air),
            (b"top", Layer::Top),
        ]
        .iter()
        .find(|(w, _)| w.eq_ignore_ascii_case(v))
        .map_or(Layer::Air, |(_, l)| *l)
    });
    AnimType {
        // spec: S-0109 Import mapping (Start absent 0; End and LoopEnd as given, claim 25)
        start: int(b"Start").unwrap_or(0),
        end: int(b"End"),
        loop_end: int(b"LoopEnd"),
        // spec: S-0109 Import mapping (Reverse, Shadow: absent no)
        reverse: flag(b"Reverse", false),
        shadow: flag(b"Shadow", false),
        // spec: S-0102 Import mapping (Flat, Tiled absent no; YDrawOffset absent 0)
        flat: flag(b"Flat", false),
        tiled: flag(b"Tiled", false),
        layer,
        y_draw_offset: int(b"YDrawOffset").unwrap_or(0),
        // spec: S-0109 Import mapping (DetailLevel, TranslucencyDetailLevel, Translucency absent 0;
        // Translucent absent no)
        detail: int(b"DetailLevel").unwrap_or(0),
        translucent: flag(b"Translucent", false),
        translucency: int(b"Translucency").unwrap_or(0),
        translucency_detail: int(b"TranslucencyDetailLevel").unwrap_or(0),
        // spec: S-0103 Import mapping (AltPalette absent no, ShouldUseCellDrawer absent yes,
        // UseNormalLight absent no)
        alt_palette: flag(b"AltPalette", false),
        cell_drawer: flag(b"ShouldUseCellDrawer", true),
        normal_light: flag(b"UseNormalLight", false),
        // spec: S-0109 Import mapping (Theater, NewTheater); Terms (new-theater type: the flag
        // without the theater flag)
        new_theater: flag(b"NewTheater", false) && !flag(b"Theater", false),
        // spec: S-0109 Import mapping (Rate absent: delay 1; Normalized, PingPong absent no;
        // LoopStart, LoopCount absent 0; RandomLoopDelay absent 0,0; Next absent none)
        rate: int(b"Rate"),
        normalized: flag(b"Normalized", false),
        loop_start: int(b"LoopStart").unwrap_or(0),
        loop_count: int(b"LoopCount").unwrap_or(0),
        ping_pong: flag(b"PingPong", false),
        random_loop_delay: art.value(image, b"RandomLoopDelay").map_or((0, 0), |v| {
            let ([d1, d2], _) = value::int_tuple_or::<2>(v, [0, 0]);
            (d1, d2)
        }),
        next: art
            .value(image, b"Next")
            .map(|v| value::text(v, usize::MAX).to_vec())
            .filter(|n| !n.is_empty() && !value::is_no_type(n)),
        // spec: S-0102 Import mapping (ZAdjust, YSortAdjust absent 0)
        z_adjust: int(b"ZAdjust").unwrap_or(0),
        y_sort_adjust: int(b"YSortAdjust").unwrap_or(0),
    }
}

/// The sprite end *E*<sub>s</sub> for a sprite of `n` image frames
/// (S-0109 Terms, claim 10).
pub fn sprite_end(t: &AnimType, n: i32) -> i32 {
    // spec: S-0109 claim 10 (a shadow type's sprite end is ⌊N / 2⌋)
    if t.shadow { n / 2 } else { n }
}

/// The end field *E* in play for a sprite of `n` image frames, the first
/// sprite found and the one drawn being the same (F2's assumed case).
pub fn end_in_play(t: &AnimType, n: i32) -> i32 {
    let es = sprite_end(t, n);
    // spec: S-0109 F2 (End: absent or −1 E_s; 0 kept except for a new-theater type found at the refresh)
    match t.end {
        None | Some(-1) => es,
        Some(0) if t.new_theater => es,
        Some(v) => v,
    }
}

/// The image frame an animation of type `t` shows when it is created, its
/// sprite holding `n` image frames (F2's assumed case).
pub fn first_frame(t: &AnimType, n: i32) -> i32 {
    if !t.reverse {
        // spec: S-0109 claims 1 and 12, F3 (k₀ = 0: image frame a; no random start)
        return t.start;
    }
    let end = end_in_play(t, n);
    // spec: S-0109 F2 (LoopEnd: absent E_s, not End; −1 E; 0 kept except for a new-theater type)
    let loop_end = match t.loop_end {
        None => sprite_end(t, n),
        Some(-1) => end,
        Some(0) if t.new_theater => end,
        Some(v) => v,
    };
    // spec: S-0109 claim 8, F3 (k₀ = λ_e − 1 for a reversed animation: image frame a + λ_e − 1)
    t.start + loop_end - 1
}

/// Whether an animation of type `t` is drawn at the player's detail level
/// `detail` (S-0109 claim 19).
pub fn shown_at(t: &AnimType, detail: i32) -> bool {
    // spec: S-0109 claim 19 (not drawn when the detail field is greater than the player's level)
    t.detail <= detail
}

/// The translucency code of a newly created slot animation of a building
/// whose fade level is 0 (S-0109 claim 20, *t* = 0, stage *k* = 0), at the
/// player's detail level `detail`, its sprite holding `n` image frames.
pub fn first_translucency(t: &AnimType, detail: i32, n: i32) -> u8 {
    translucency(t, detail, 0, end_in_play(t, n))
}

/// The fade code of a translucent-flag type at stage `k` with end field
/// `e` in play (S-0109 claim 20, F4's integer reduction).
pub fn fade_code(k: i32, e: i32) -> u8 {
    let (k, e) = (i64::from(k), i64::from(e));
    // spec: S-0109 F4 (E > 0: 5k ≥ 3E, 5k > 2E, 5k > E; E = 0: k > 0 gives 3; E < 0: 5k > 3E,
    // 5k > 2E, 5k > E)
    let three = match e.cmp(&0) {
        std::cmp::Ordering::Greater => 5 * k >= 3 * e,
        std::cmp::Ordering::Equal => k > 0,
        std::cmp::Ordering::Less => 5 * k > 3 * e,
    };
    if three {
        3
    } else if e != 0 && 5 * k > 2 * e {
        2
    } else if e != 0 && 5 * k > e {
        1
    } else {
        0
    }
}

/// The translucency code of an animation of type `t` whose translucency
/// level is 0 (S-0109 claim 20: every animation but a building's slot
/// animations), at stage `k` with end field `e` in play, at the player's
/// detail level `detail`.
pub fn translucency(t: &AnimType, detail: i32, k: i32, e: i32) -> u8 {
    // spec: S-0109 claim 20 (below the translucency detail field the code is 0)
    if detail < t.translucency_detail {
        return 0;
    }
    if t.translucent {
        // spec: S-0109 claim 20, F4 (fade by progress)
        fade_code(k, e)
    } else if t.translucency > 0 {
        // spec: S-0109 claim 20 (a field of 25, 50 or 75 gives 1, 2 or 3; any other positive value 0)
        match t.translucency {
            25 => 1,
            50 => 2,
            75 => 3,
            _ => 0,
        }
    } else {
        // spec: S-0109 claim 20 (neither: from the translucency level, 0 when t = 0)
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(text: &str) -> Document {
        Document::parse(text.as_bytes(), Default::default(), &mut Vec::new())
    }

    /// spec: S-0108 Import mapping (empty damaged and occupied names take the normal name;
    /// names cut to 15 bytes; the suffixed keys)
    #[test]
    fn slot_names_and_values() {
        let art = doc(
            "[B]\nActiveAnim=ABCDEFGHIJKLMNOPQ\nActiveAnimX=4\nActiveAnimY=-6\n\
             ActiveAnimZAdjust=-20\nActiveAnimYSort=100\nIdleAnim=I\nIdleAnimDamaged=ID\n\
             IdleAnimGarrisoned=IG\n",
        );
        let a = read_slot(&art, b"B", b"ActiveAnim");
        assert_eq!(a.normal, b"ABCDEFGHIJKLMNO");
        assert_eq!(a.damaged, a.normal);
        assert_eq!(a.occupied, a.normal);
        assert_eq!((a.x, a.y, a.z_adjust, a.y_sort), (4, -6, -20, 100));
        let i = read_slot(&art, b"B", b"IdleAnim");
        assert_eq!(
            (i.damaged.as_slice(), i.occupied.as_slice()),
            (&b"ID"[..], &b"IG"[..])
        );
    }

    /// spec: S-0108 Import mapping (the other keys take effect only when a name is non-empty)
    #[test]
    fn values_without_a_name_keep_their_defaults() {
        let art = doc("[B]\nActiveAnimTwoX=9\nActiveAnimTwoZAdjust=5\n");
        assert_eq!(read_slot(&art, b"B", b"ActiveAnimTwo"), SlotDef::default());
    }

    /// spec: S-0108 Terms (variant choice)
    #[test]
    fn variant_choice() {
        let d = SlotDef {
            normal: b"N".to_vec(),
            damaged: b"D".to_vec(),
            occupied: b"O".to_vec(),
            ..SlotDef::default()
        };
        assert_eq!(variant(&d, true, true, true), b"D");
        assert_eq!(variant(&d, false, true, true), b"O");
        assert_eq!(variant(&d, false, true, false), b"N");
        assert_eq!(variant(&d, false, false, true), b"N");
    }

    /// spec: S-0108 claim 11 (the set-up starts idle and active-1 … active-4)
    #[test]
    fn setup_slots_are_idle_and_the_four_active_slots() {
        assert_eq!(SETUP_SLOTS.len(), 5);
        assert_eq!(SETUP_SLOTS[0], b"IdleAnim");
        assert!(!SETUP_SLOTS.contains(&&b"SpecialAnim"[..]));
        assert!(!SETUP_SLOTS.contains(&&b"ProductionAnim"[..]));
    }

    /// spec: S-0102 Import mapping (Layer absent: air; Flat, Tiled, YDrawOffset);
    /// S-0103 Import mapping (ShouldUseCellDrawer absent yes)
    #[test]
    fn animation_defaults() {
        let t = read_anim(&doc("[A]\n"), b"A");
        assert_eq!(t.layer, Layer::Air);
        assert!(t.cell_drawer && !t.alt_palette && !t.normal_light);
        assert!(!t.flat && !t.tiled && !t.shadow);
        assert_eq!((t.start, t.y_draw_offset, t.detail), (0, 0, 0));
        let t = read_anim(
            &doc("[A]\nLayer=Ground\nYDrawOffset=7\nShouldUseCellDrawer=no\n"),
            b"A",
        );
        assert_eq!(t.layer, Layer::Ground);
        assert_eq!(t.y_draw_offset, 7);
        assert!(!t.cell_drawer);
    }

    /// spec: S-0102 glossary order (underground, surface, ground, air, top)
    #[test]
    fn layers_are_ordered() {
        assert!(Layer::Underground < Layer::Surface);
        assert!(Layer::Surface < Layer::Ground);
        assert!(Layer::Ground < Layer::Air);
        assert!(Layer::Air < Layer::Top);
    }

    /// spec: S-0109 claims 1 and 12 (the first image frame is a); claim 8 (reversed:
    /// a + λ_e − 1); F2 (absent loop end: the sprite end; a shadow type's is ⌊N / 2⌋)
    #[test]
    fn first_frames() {
        let t = read_anim(&doc("[A]\nStart=3\n"), b"A");
        assert_eq!(first_frame(&t, 20), 3);
        let r = read_anim(&doc("[A]\nStart=1\nReverse=yes\n"), b"A");
        assert_eq!(first_frame(&r, 20), 1 + 20 - 1);
        let rs = read_anim(&doc("[A]\nReverse=yes\nShadow=yes\n"), b"A");
        assert_eq!(first_frame(&rs, 20), 10 - 1);
        let rl = read_anim(&doc("[A]\nReverse=yes\nLoopEnd=6\n"), b"A");
        assert_eq!(first_frame(&rl, 20), 5);
        let re = read_anim(&doc("[A]\nReverse=yes\nEnd=4\nLoopEnd=-1\n"), b"A");
        assert_eq!(first_frame(&re, 20), 3);
    }

    /// spec: S-0109 claim 19 (drawn only when the detail field is at most the player's level)
    #[test]
    fn detail_level() {
        let t = read_anim(&doc("[A]\nDetailLevel=2\n"), b"A");
        assert!(shown_at(&t, 2));
        assert!(!shown_at(&t, 1));
    }

    /// spec: S-0109 claim 20 (codes at t = 0, k = 0)
    #[test]
    fn translucency_codes() {
        let code = |s: &str, d: i32| first_translucency(&read_anim(&doc(s), b"A"), d, 20);
        assert_eq!(code("[A]\nTranslucency=50\n", 2), 2);
        assert_eq!(code("[A]\nTranslucency=75\n", 2), 3);
        assert_eq!(code("[A]\nTranslucency=30\n", 2), 0);
        assert_eq!(
            code("[A]\nTranslucency=25\nTranslucencyDetailLevel=2\n", 1),
            0
        );
        assert_eq!(code("[A]\nTranslucent=yes\n", 2), 0);
        assert_eq!(code("[A]\nTranslucent=yes\nEnd=-5\n", 2), 3);
        assert_eq!(code("[A]\n", 2), 0);
    }

    /// spec: S-0109 claim 20 and F4 (a translucent-flag type fades by its
    /// progress k through its end E: code 3 from 5k ≥ 3E, 2 from 5k > 2E,
    /// 1 from 5k > E, for E > 0; E = 0 gives 3 once k > 0)
    #[test]
    fn translucent_fade_by_progress() {
        let codes: Vec<u8> = (0..10).map(|k| fade_code(k, 10)).collect();
        assert_eq!(codes, [0, 0, 0, 1, 1, 2, 3, 3, 3, 3]);
        assert_eq!(fade_code(0, 0), 0);
        assert_eq!(fade_code(1, 0), 3);
        let t = read_anim(&doc("[A]\nTranslucent=yes\n"), b"A");
        assert_eq!(translucency(&t, 2, 6, 10), 3);
    }
}
