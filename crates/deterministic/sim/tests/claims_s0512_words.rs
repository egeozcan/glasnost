//! Claim tests of S-0512 revision 7: the kind and link vocabulary (claim 24;
//! Q-0730) and the aim-action vocabulary (claim 23; Q-0731), on synthetic
//! rules (`tests/sw_common`).

mod sw_common;

use sw_common::*;

#[test]
fn claim23_action_vocabulary_words_are_distinct_actions_and_other_words_are_none() {
    // spec: S-0512 claim 23 and Import mapping (every listed word is its own action, case-insensitive; `none` and a word outside the vocabulary are no action)
    let mut r = ruleset();
    let set = |r: &mut glasnost_rules::schema::Ruleset, name: &str, a: &str| {
        r.sim.superweapons.get_mut(name).unwrap().action = Some(a.into());
    };
    set(&mut r, "SHARED", "ChronoWarp");
    set(&mut r, "SPY", "IVANBOMB");
    set(&mut r, "DROP", "Para_Drop");
    set(&mut r, "STORM", "bogus");
    set(&mut r, "CURTAIN", "NONE");
    let w = game(&r);
    let a = |i: u32| w.rules().superweapons.types[i as usize].action.clone();
    assert_eq!(a(SHARED).as_deref(), Some("chronowarp"));
    assert_eq!(a(SPY).as_deref(), Some("ivanbomb"));
    assert_eq!(a(DROP), None, "punctuation is not ignored");
    assert_eq!(a(STORM), None, "an unrecognised word sets no action");
    assert_eq!(a(CURTAIN), None);
    for word in glasnost_rules::sim_econ::SUPERWEAPON_ACTION_WORDS {
        assert!(glasnost_rules::sim_econ::superweapon_action_known(
            &word.to_ascii_uppercase()
        ));
    }
    assert!(!glasnost_rules::sim_econ::superweapon_action_known(""));
}

#[test]
fn claim24_kind_words_are_the_twelve_literal_words() {
    // spec: S-0512 claim 24 (twelve words, kind numbers 0 to 11 in table order, case-insensitive whole-word match; no punctuation variants)
    use glasnost_sim::superweapons::{SwKind, kind_of_word};
    let words = [
        ("multimissile", SwKind::Missile),
        ("ironcurtain", SwKind::IronCurtain),
        ("lightningstorm", SwKind::Storm),
        ("chronosphere", SwKind::Chrono),
        ("chronowarp", SwKind::ChronoWarp),
        ("paradrop", SwKind::Paradrop),
        ("amerparadrop", SwKind::AmerParadrop),
        ("psychicdominator", SwKind::Dominator),
        ("spyplane", SwKind::SpyPlane),
        ("geneticconverter", SwKind::Genetic),
        ("forceshield", SwKind::ForceShield),
        ("psychicreveal", SwKind::PsychicReveal),
    ];
    for (n, (w, k)) in words.iter().enumerate() {
        assert_eq!(kind_of_word(w), Some(*k));
        assert_eq!(kind_of_word(&w.to_ascii_uppercase()), Some(*k));
        assert_eq!(k.number(), n as u32);
        assert_eq!(SwKind::from_number(n as u32), Some(*k));
    }
    for bad in [
        "iron-curtain",
        "iron_curtain",
        "iron curtain",
        "parachutedrop",
        "spyaircraft",
        "chronospherewarp",
        "None",
        "",
    ] {
        assert_eq!(kind_of_word(bad), None, "{bad:?}");
    }
}

#[test]
fn claim24_unknown_kind_and_link_words_leave_no_kind_and_no_link() {
    // spec: S-0512 claim 24 (a type with an unusable word has no kind and no link; the layering that keeps an earlier word is the importer's)
    let mut r = ruleset();
    r.sim.superweapons.get_mut("CURTAIN").unwrap().kind = Some("Iron-Curtain".into());
    r.sim.superweapons.get_mut("WARP").unwrap().pre_dependent = Some("Chrono Sphere".into());
    let w = game(&r);
    let ty = &w.rules().superweapons.types;
    assert_eq!(ty[CURTAIN as usize].kind, None);
    assert_eq!(ty[WARP as usize].pre_dependent, None);
    assert!(!w.rules().superweapons.warnings.is_empty());
}
