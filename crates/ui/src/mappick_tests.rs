//! Tests of the map picker on invented maps (design: map picker (lead's
//! request 2026-11-13); the marker positions S-0009 claim 25 and Formulas).

use super::*;
use crate::draw::Prim;
use crate::geom::MIN_UI;
use crate::setup::{Listed, SetupRules};
use crate::shell::{Screen, Shell};

fn entry(label: &str, starts: u32) -> MapEntry {
    MapEntry {
        label: label.into(),
        starts,
    }
}

fn facts(size: (i32, i32), theater: &str, preview: bool) -> MapFacts {
    MapFacts {
        size,
        theater: theater.into(),
        preview,
    }
}

/// Six invented maps and their facts.
fn sample() -> (Vec<MapEntry>, Vec<MapFacts>) {
    (
        vec![
            entry("Delta Ridge", 4),
            entry("alpha coast", 2),
            entry("Echo Ridge", 8),
            entry("Bravo Flats", 2),
            entry("Charlie Basin", 6),
            entry("Foxtrot Ridge", 4),
        ],
        vec![
            facts((100, 100), "Snow", true),
            facts((50, 60), "Desert", false),
            facts((200, 150), "Snow", true),
            facts((60, 50), "Desert", true),
            facts((120, 120), "Lunar", false),
            facts((90, 80), "Snow", true),
        ],
    )
}

fn query() -> MapQuery {
    MapQuery::default()
}

#[test]
fn the_search_ignores_case_and_needs_every_word() {
    let (m, f) = sample();
    let mut q = query();
    assert_eq!(q.shown(&m, &f), [0, 1, 2, 3, 4, 5], "no search: every map");
    q.search = "RIDGE".into();
    assert_eq!(q.shown(&m, &f), [0, 2, 5]);
    q.search = "  ridge  ECHO ".into();
    assert_eq!(q.shown(&m, &f), [2]);
    q.search = "Coast".into();
    assert_eq!(q.shown(&m, &f), [1]);
    q.search = "nothing here".into();
    assert!(q.shown(&m, &f).is_empty());
}

#[test]
fn typing_keeps_printable_characters_up_to_the_limit() {
    let mut q = query();
    q.type_text("ab\u{8}c\n");
    assert_eq!(q.search, "abc");
    q.backspace();
    assert_eq!(q.search, "ab");
    q.type_text(&"x".repeat(100));
    assert_eq!(q.search.chars().count(), SEARCH_CHARS);
}

#[test]
fn filters_narrow_by_players_size_theater_and_preview() {
    let (m, f) = sample();
    let mut q = query();
    // The player counts offered: every count, then each present.
    assert_eq!(player_options(&m), [0, 2, 4, 6, 8]);
    q.step(FilterKind::Players, 1, &m, &f);
    assert_eq!(q.players, 2);
    assert_eq!(q.shown(&m, &f), [1, 3]);
    q.step(FilterKind::Players, -1, &m, &f);
    assert_eq!(q.players, 0, "back round to every count");
    q.players = 4;
    assert_eq!(q.shown(&m, &f), [0, 5]);
    q.players = 0;
    // Theaters offered: every theater, then each present by name.
    assert_eq!(theater_options(&f), ["", "Desert", "Lunar", "Snow"]);
    q.step(FilterKind::Theater, 3, &m, &f);
    assert_eq!(q.theater, "Snow");
    assert_eq!(q.shown(&m, &f), [0, 2, 5]);
    q.theater.clear();
    q.preview = PreviewFilter::Without;
    assert_eq!(q.shown(&m, &f), [1, 4]);
    q.preview = PreviewFilter::With;
    assert_eq!(q.shown(&m, &f), [0, 2, 3, 5]);
    q.preview = PreviewFilter::Any;
    q.size = SizeClass::Small;
    assert_eq!(q.shown(&m, &f), [1, 3], "the smallest third by area");
    q.size = SizeClass::Large;
    assert_eq!(q.shown(&m, &f), [2, 4]);
    q.size = SizeClass::Medium;
    assert_eq!(q.shown(&m, &f), [0, 5]);
    // Filters and the search together.
    q.size = SizeClass::Any;
    q.search = "ridge".into();
    q.players = 8;
    assert_eq!(q.shown(&m, &f), [2]);
    // A map without facts has none of the filtered properties.
    let mut q = query();
    q.preview = PreviewFilter::Without;
    assert_eq!(q.shown(&m, &[]), [0, 1, 2, 3, 4, 5]);
}

#[test]
fn size_classes_are_thirds_by_area_and_ties_share_a_class() {
    let f: Vec<MapFacts> = [(10, 10), (20, 20), (10, 10), (30, 30), (40, 40), (50, 50)]
        .into_iter()
        .map(|s| facts(s, "", false))
        .collect();
    use SizeClass::*;
    assert_eq!(
        size_classes(6, &f),
        [Small, Medium, Small, Medium, Large, Large],
        "by area rank; the third map has the first's area, so its class"
    );
    assert_eq!(size_classes(1, &f), [Small]);
    assert!(size_classes(0, &f).is_empty());
}

#[test]
fn sorts_by_name_players_and_size() {
    let (m, f) = sample();
    let mut q = query();
    assert_eq!(q.value_label(FilterKind::Sort), "As found");
    q.step(FilterKind::Sort, 1, &m, &f);
    assert_eq!(q.sort, Sort::Name);
    assert_eq!(q.shown(&m, &f), [1, 3, 4, 0, 2, 5], "case ignored");
    q.sort = Sort::Players;
    assert_eq!(q.shown(&m, &f), [1, 3, 0, 5, 4, 2], "ties by name");
    q.sort = Sort::Size;
    assert_eq!(q.shown(&m, &f), [1, 3, 5, 0, 4, 2], "equal areas by name");
    q.step(FilterKind::Sort, 1, &m, &f);
    assert_eq!(q.sort, Sort::Listed);
    // A sort with a filter.
    q.sort = Sort::Name;
    q.search = "ridge".into();
    assert_eq!(q.shown(&m, &f), [0, 2, 5]);
}

fn rules() -> SetupRules {
    SetupRules {
        money: 1000,
        money_range: (500, 2000, 100),
        units: 2,
        units_range: (1, 5),
        bases: true,
        flags: [false; 5],
        game_speed: 3,
        countries: vec!["Alphaland".into(), "Betaland".into()],
        colours: vec![[1, 2, 3, 255]; 8],
    }
}

/// A shell on the setup screen with map `at` chosen.
fn shell_at(at: usize) -> Shell {
    let (m, f) = sample();
    let mut s = Shell::new();
    s.menu(crate::menu::MenuHit::Skirmish, &rules());
    s.setup_with(SetupHit::PickMap(at), &rules(), &m, &f);
    assert_eq!(s.setup.as_ref().unwrap().map, at);
    s
}

fn chosen(s: &Shell) -> usize {
    s.setup.as_ref().unwrap().map
}

#[test]
fn filters_keep_a_matching_selection_or_take_the_first_match() {
    let (m, f) = sample();
    let r = rules();
    let mut s = shell_at(5);
    s.setup_with(SetupHit::OpenMaps, &r, &m, &f);
    assert!(s.picker.open && s.picker.typing);
    // "Foxtrot Ridge" still matches: kept.
    s.type_search("ridge", &m, &f);
    assert_eq!(chosen(&s), 5);
    // It no longer matches: the first match ("Echo Ridge").
    s.type_search(" ech", &m, &f);
    assert_eq!(chosen(&s), 2);
    // Back to a wider search: the selection is kept.
    for _ in 0..4 {
        s.search_backspace(&m, &f);
    }
    assert_eq!(s.picker.query.search, "ridge");
    assert_eq!(chosen(&s), 2);
    // A filter change follows the same rule, in the list's order.
    s.setup_with(SetupHit::Filter(FilterKind::Sort, 1), &r, &m, &f);
    s.setup_with(SetupHit::Filter(FilterKind::Players, 2), &r, &m, &f);
    assert_eq!(s.picker.query.players, 4);
    assert_eq!(s.maps_shown(&m, &f), [0, 5]);
    assert_eq!(chosen(&s), 0, "the first of the list");
    // The arrows step through the list, round.
    s.setup_with(SetupHit::Map(1), &r, &m, &f);
    assert_eq!(chosen(&s), 5);
    s.setup_with(SetupHit::Map(1), &r, &m, &f);
    assert_eq!(chosen(&s), 0);
    // A choice of map from the list.
    s.setup_with(SetupHit::PickMap(5), &r, &m, &f);
    assert_eq!(chosen(&s), 5);
    // Clear empties the search.
    s.setup_with(SetupHit::ClearSearch, &r, &m, &f);
    assert!(s.picker.query.search.is_empty());
    assert_eq!(chosen(&s), 5);
    s.setup_with(SetupHit::MapsDone, &r, &m, &f);
    assert!(!s.picker.open && !s.picker.typing);
    // Typing goes nowhere with the picker closed.
    s.type_search("zzz", &m, &f);
    assert!(s.picker.query.search.is_empty());
}

#[test]
fn with_no_match_the_selection_stays_and_the_game_does_not_start() {
    let (m, f) = sample();
    let r = rules();
    let mut s = shell_at(3);
    s.setup_with(SetupHit::OpenMaps, &r, &m, &f);
    s.type_search("no such map", &m, &f);
    assert!(s.maps_shown(&m, &f).is_empty());
    assert_eq!(chosen(&s), 3, "unchanged");
    assert_eq!(s.setup_with(SetupHit::Start_, &r, &m, &f), None);
    assert_eq!(s.screen, Screen::Setup);
    // The arrows have nothing to step to.
    s.setup_with(SetupHit::Map(1), &r, &m, &f);
    assert_eq!(chosen(&s), 3);
    // The setup screen says so and offers no start.
    let setup = s.setup.as_ref().unwrap();
    let list = Listed {
        any: false,
        shown: Shown::NONE,
    };
    let (d, t) = setup.view_listed((800, 600), &r, &m, None, &list);
    let texts = d.texts();
    assert!(
        texts.contains(&"No map matches the search and filters"),
        "{texts:?}"
    );
    assert!(t.rect_of(&SetupHit::Start_).is_none());
    assert!(t.rect_of(&SetupHit::Map(1)).is_none());
    assert!(t.rect_of(&SetupHit::OpenMaps).is_some());
    // The picker says so too.
    let (d, t) = s
        .picker
        .view((800, 600), &m, &f, 3, &[], &Shown::NONE, None);
    assert!(d.texts().contains(&"No map matches the search and filters"));
    assert!(t.rect_of(&SetupHit::MapsDone).is_some());
    // A match again: the first is chosen and the game starts.
    s.setup_with(SetupHit::ClearSearch, &r, &m, &f);
    assert_eq!(chosen(&s), 3);
    assert!(matches!(
        s.setup_with(SetupHit::Start_, &r, &m, &f),
        Some(crate::shell::Event::StartGame(_))
    ));
}

#[test]
fn without_facts_and_filters_the_arrows_step_in_install_order() {
    let (m, _) = sample();
    let r = rules();
    let mut s = shell_at(0);
    for want in [1, 2, 3, 4, 5, 0] {
        s.setup(SetupHit::Map(1), &r, &m);
        assert_eq!(chosen(&s), want);
    }
    s.setup(SetupHit::Map(-1), &r, &m);
    assert_eq!(chosen(&s), 5);
}

#[test]
fn the_query_is_kept_as_text_pairs() {
    let mut q = MapQuery {
        search: "ridge\nx".into(),
        players: 4,
        size: SizeClass::Large,
        theater: "Snow".into(),
        preview: PreviewFilter::Without,
        sort: Sort::Players,
    };
    let pairs = q.kept();
    assert!(pairs.iter().all(|(_, v)| !v.contains('\n')));
    q.search = "ridgex".into();
    let mut back = MapQuery::default();
    for (k, v) in &pairs {
        assert!(back.keep(k, v), "{k}");
    }
    assert_eq!(back, q);
    // Wrong values leave the query as it is; other keys are not its own.
    let mut w = MapQuery::default();
    assert!(w.keep("players", "many"));
    assert!(w.keep("size", "huge"));
    assert!(w.keep("sort", ""));
    assert_eq!(w, MapQuery::default());
    assert!(!w.keep("volume", "3"));
    assert!(MapQuery::default().is_default());
}

#[test]
fn the_list_scrolls_and_keeps_the_choice_in_view() {
    let m: Vec<MapEntry> = (0..50).map(|i| entry(&format!("Map {i:02}"), 2)).collect();
    let mut p = MapPicker::default();
    p.reveal(30, 50);
    assert_eq!(p.scroll, 30 + 1 - ROWS);
    p.reveal(5, 50);
    assert_eq!(p.scroll, 5);
    p.scroll_by(100, 50);
    assert_eq!(p.scroll, 50 - ROWS);
    p.scroll_by(-100, 50);
    assert_eq!(p.scroll, 0);
    let shown: Vec<usize> = (0..50).collect();
    p.scroll = 10;
    let (d, t) = p.view((640, 480), &m, &[], 12, &shown, &Shown::NONE, None);
    // Rows 10 … 29 have targets; the others do not.
    assert!(t.rect_of(&SetupHit::PickMap(9)).is_none());
    assert!(t.rect_of(&SetupHit::PickMap(10)).is_some());
    assert!(t.rect_of(&SetupHit::PickMap(29)).is_some());
    assert!(t.rect_of(&SetupHit::PickMap(30)).is_none());
    let row = t.rect_of(&SetupHit::PickMap(12)).unwrap();
    assert_eq!(t.hit(row.centre()), Some(SetupHit::PickMap(12)));
    // The scroll bar pages either way.
    assert!(t.rect_of(&SetupHit::ScrollMaps(-(ROWS as i32))).is_some());
    assert!(t.rect_of(&SetupHit::ScrollMaps(ROWS as i32)).is_some());
    assert!(d.texts().contains(&"50 of 50 maps"));
}

/// Every picker target inside the window and the panel, none on top of
/// another, at the smallest interface and larger ones.
#[test]
fn the_picker_fits_the_smallest_interface() {
    let (m, f) = sample();
    let mut p = MapPicker {
        open: true,
        typing: true,
        ..MapPicker::default()
    };
    p.query.search = "a long search that runs past the field's end".into();
    let shown: Vec<usize> = (0..m.len()).collect();
    for size in [MIN_UI, (800, 600), (1280, 960)] {
        let (_, t) = p.view(size, &m, &f, 2, &shown, &Shown::NONE, None);
        let panel = Setup::panel_rect(size);
        assert!(panel.x >= 0 && panel.y >= 0);
        assert!(panel.right() <= size.0 && panel.bottom() <= size.1);
        for (i, (r, hit)) in t.list.iter().enumerate() {
            assert!(
                r.x >= panel.x && r.y >= panel.y,
                "{hit:?} {r:?} outside {panel:?}"
            );
            assert!(r.right() <= panel.right() && r.bottom() <= panel.bottom());
            for (o, other) in &t.list[i + 1..] {
                assert!(
                    r.intersect(o).is_empty(),
                    "{hit:?} overlaps {other:?} at {size:?}"
                );
            }
        }
    }
}

/// Every filter label fits beside its value box, and every value offered
/// fits in it.
#[test]
fn every_filter_label_and_value_fits() {
    let (m, mut f) = sample();
    f[0].theater = "New urban".into();
    let (label, _, sw) = SORT_AT;
    let mut all = vec![(label, sw, FilterKind::Sort)];
    all.extend(FILTERS.iter().map(|&(k, l, _, w)| (l, w, k)));
    for (label, w, k) in all {
        let lw = font::text_width(label);
        assert!(lw <= w * 2 / 5, "{label} is {lw} pixels wide");
        let mut q = MapQuery::default();
        // Every value of the filter, round once.
        for _ in 0..8 {
            let v = q.value_label(k);
            let vw = font::text_width(&v);
            assert!(vw <= w - w * 2 / 5 - 16, "{label}: {v} is {vw} pixels wide");
            q.step(k, 1, &m, &f);
        }
    }
}

// spec: S-0009 Formulas (preview marker position: px = trunc(trunc((xi − X0)·1000 / Wh) · pw / 1000) + left − 9, py likewise with − 6)
#[test]
fn markers_follow_the_formula() {
    let m = Marks {
        count: 2,
        origin: (10, 20),
        extent: (100, 50),
        pairs: vec![(60, 45), (5, 20)],
    };
    let drawn = Rect::new(300, 100, 200, 120);
    // Pair 1: (60 − 10)·1000/100 = 500, 500·200/1000 = 100 → 100 + 300 − 9;
    // (45 − 20)·1000/50 = 500, 500·120/1000 = 60 → 60 + 100 − 6.
    // Pair 2: (5 − 10)·1000/100 = −50, −50·200/1000 = −10 (toward zero) →
    // −10 + 300 − 9; (20 − 20) → 0 + 100 − 6.
    assert_eq!(marker_corners(&m, drawn), [(391, 154), (281, 94)]);
    // Truncation toward zero, not floor: −7·1000/100 = −70, −70·15/1000 = −1.05 → −1.
    let n = Marks {
        count: 1,
        origin: (7, 0),
        extent: (100, 100),
        pairs: vec![(0, 0)],
    };
    assert_eq!(marker_corners(&n, Rect::new(0, 0, 15, 15)), [(-10, -6)]);
}

// spec: S-0009 Formulas (the wrap example: X0 = −2147484, Wh = 1000, xi = 2147984 give the inner quotient 500)
#[test]
fn marker_products_wrap() {
    let m = Marks {
        count: 1,
        origin: (-2_147_484, 0),
        extent: (1000, 10),
        pairs: vec![(2_147_984, 5)],
    };
    // 500·200/1000 = 100 → 100 + 10 − 9; 5·1000/10 = 500, 500·80/1000 = 40 → 40 + 20 − 6.
    assert_eq!(marker_corners(&m, Rect::new(10, 20, 200, 80)), [(101, 54)]);
}

// spec: S-0009 claim 25 (markers only for 1 ≤ N_h ≤ 8; an extent of 0 draws none in our engine; no header reads as −1)
#[test]
fn markers_need_a_count_of_one_to_eight_and_an_extent() {
    let base = Marks {
        count: 1,
        origin: (0, 0),
        extent: (10, 10),
        pairs: vec![(5, 5)],
    };
    let r = Rect::new(0, 0, 100, 100);
    assert_eq!(marker_corners(&base, r).len(), 1);
    for (count, extent) in [
        (0, (10, 10)),
        (-1, (-1, -1)),
        (9, (10, 10)),
        (1, (0, 10)),
        (1, (10, 0)),
    ] {
        let m = Marks {
            count,
            extent,
            ..base.clone()
        };
        assert!(marker_corners(&m, r).is_empty(), "{count} {extent:?}");
    }
    // Only pairs 1 … N_h.
    let m = Marks {
        count: 1,
        pairs: vec![(1, 1), (2, 2)],
        ..base.clone()
    };
    assert_eq!(marker_corners(&m, r).len(), 1);
    // An extent of −1 divides: 5·1000/−1 wraps nothing, it is −5000.
    let m = Marks {
        extent: (-1, 10),
        ..base
    };
    assert_eq!(marker_corners(&m, r)[0].0, -5000 * 100 / 1000 - 9);
}

#[test]
fn a_preview_box_shows_the_image_fitted_with_markers_or_a_placeholder() {
    let r = Rect::new(0, 0, 104, 104);
    let mut d = DrawList::new();
    draw_preview(&mut d, r, &Shown::NONE, "No preview");
    assert!(d.texts().contains(&"No preview"));
    assert!(!d.prims.iter().any(|p| matches!(p, Prim::Image { .. })));
    let marks = Marks {
        count: 2,
        origin: (0, 0),
        extent: (10, 10),
        pairs: vec![(5, 5), (0, 0)],
    };
    let shown = Shown {
        preview: PreviewShow::Image {
            id: 7,
            size: (50, 25),
        },
        marks: Some(&marks),
    };
    let mut d = DrawList::new();
    draw_preview(&mut d, r, &shown, "No preview");
    // 50 × 25 fitted in 100 × 100: 100 × 50, centred.
    let image = d.prims.iter().find_map(|p| match p {
        Prim::Image { id: 7, rect } => Some(*rect),
        _ => None,
    });
    assert_eq!(image, Some(Rect::new(2, 27, 100, 50)));
    let texts = d.texts();
    assert!(texts.contains(&"1") && texts.contains(&"2"), "{texts:?}");
    // A pending preview shows neither.
    let mut d = DrawList::new();
    let pending = Shown {
        preview: PreviewShow::Pending,
        marks: Some(&marks),
    };
    draw_preview(&mut d, r, &pending, "No preview");
    assert!(!d.texts().contains(&"No preview"));
    assert!(!d.texts().contains(&"1"));
}

#[test]
fn fitting_keeps_proportions() {
    let inner = Rect::new(10, 10, 200, 100);
    assert_eq!(fit_image((400, 200), inner), inner);
    assert_eq!(fit_image((100, 100), inner), Rect::new(60, 10, 100, 100));
    assert_eq!(fit_image((300, 50), inner), Rect::new(10, 43, 200, 33));
    assert_eq!(fit_image((0, 0), inner), Rect::new(60, 10, 100, 100));
}

/// The skirmish screen with the picker's parts at the smallest interface:
/// the preview below the settings, inside the panel, on no other target.
#[test]
fn the_setup_screen_fits_its_preview() {
    let (m, f) = sample();
    let r = rules();
    let mut s = shell_at(2);
    // Every row shown (the map has eight starts).
    for i in 2..8 {
        s.setup.as_mut().unwrap().slots[i].kind = crate::setup::SlotKind::Ai;
    }
    let _ = f;
    let list = Listed {
        any: true,
        shown: Shown::NONE,
    };
    for size in [MIN_UI, (800, 600)] {
        let (d, t) = s
            .setup
            .as_ref()
            .unwrap()
            .view_listed(size, &r, &m, None, &list);
        let panel = Setup::panel_rect(size);
        let thumb = t.rect_of(&SetupHit::OpenMaps).unwrap();
        assert_eq!(thumb.w, 100, "the button first");
        let all: Vec<_> = t
            .list
            .iter()
            .filter(|(_, h)| *h == SetupHit::OpenMaps)
            .collect();
        assert_eq!(all.len(), 2, "the button and the preview");
        let preview = all[1].0;
        assert!(preview.right() <= panel.right() && preview.bottom() <= panel.bottom());
        for (rect, hit) in &t.list {
            if *hit != SetupHit::OpenMaps {
                assert!(
                    rect.intersect(&preview).is_empty(),
                    "{hit:?} under the preview"
                );
            }
        }
        assert!(d.texts().contains(&"Choose map"));
        assert!(d.texts().contains(&"No preview"));
        for want in [SetupHit::Map(1), SetupHit::Start_, SetupHit::Kind(7, 1)] {
            assert!(t.rect_of(&want).is_some(), "{want:?}");
        }
    }
}
