//! A particle type's colour list from the imported text to the simulation
//! (S-0202 claim 17 and Edge cases, `quirks.colour_list_item_triples`):
//! claim 15's colour-draw gate and S-0616 claim 29 and F16's colour steps
//! read the entries the importer's reading gives. Synthetic rules only:
//! invented type names, every number our own.

use glasnost_convert::inputs::{InputFile, Inputs};
use glasnost_convert::sim::SimQuirks;
use glasnost_convert::{ImportOptions, convert};
use glasnost_num::Rounding;
use glasnost_sim::gas::{ColourState, GasRules, ParticleType, Rgb};

const M: Rounding = Rounding::Nearest;

/// The particle types of `rules` after import with the switch set to
/// `item_triples` and the simulation's rules build.
fn particles(rules: &str, item_triples: bool) -> Vec<ParticleType> {
    let inputs = Inputs {
        base: Some(InputFile {
            name: "rules.ini".into(),
            origin: "test".into(),
            bytes: rules.as_bytes().to_vec(),
        }),
        ..Inputs::default()
    };
    let options = ImportOptions {
        sim: SimQuirks {
            colour_list_item_triples: item_triples,
            ..SimQuirks::default()
        },
        ..ImportOptions::default()
    };
    let import = convert(&inputs, &options);
    let r = glasnost_rules::resolve_ruleset(Some((import.tree, import.provenance)), &[])
        .unwrap_or_else(|e| panic!("{e}"));
    GasRules::new(&r.ruleset, &[], &glasnost_rules::quirks::Quirks::default()).particles
}

fn named<'a>(ps: &'a [ParticleType], name: &str) -> &'a ParticleType {
    ps.iter().find(|p| p.name == name).expect("particle type")
}

/// The colour a particle of type `p` shows after each update, from its
/// creation until it shows `last` (each update's colour draw 0).
fn shown(p: &ParticleType, last: Rgb) -> Vec<Rgb> {
    let mut c = ColourState::new(&p.look, None, M);
    let mut out = Vec::new();
    for _ in 0..64 {
        c.advance(&p.look, 0, M);
        let now = c.drawn(&p.look, M);
        out.push(now);
        if now == last {
            return out;
        }
    }
    panic!("{} never shows {last:?}: {out:?}", p.name);
}

/// spec: S-0202 claim 15 (the colour draw needs a list with at least one
/// entry and a nonblack start colour) and claim 17 (a reading with no
/// entry, such as `(200,100)` alone, is an empty list; R16 part b's three
/// lists), Edge cases (quirks.colour_list_item_triples off: the group gives
/// one entry, so the draw is taken)
#[test]
fn colour_draw_gate_reads_the_entries() {
    let rules = "[Particles]\n0=Pair\n1=Empty\n2=Three\n\
                 [Pair]\nBehavesLike=Smoke\nStartColor1=1,2,3\nColorList=(200,100)\n\
                 [Empty]\nBehavesLike=Smoke\nStartColor1=1,2,3\n\
                 [Three]\nBehavesLike=Smoke\nStartColor1=1,2,3\nColorList=(200,100,50)\n";
    let on = particles(rules, true);
    assert!(!named(&on, "Pair").draw.colour_draw);
    assert!(!named(&on, "Empty").draw.colour_draw);
    assert!(named(&on, "Three").draw.colour_draw);
    let off = particles(rules, false);
    assert!(named(&off, "Pair").draw.colour_draw);
    assert!(!named(&off, "Empty").draw.colour_draw);
    assert!(named(&off, "Three").draw.colour_draw);
}

/// spec: S-0202 claim 17 and R16 part a (a spark type with black start
/// colours and colour speed 2.0 shows its entries one update each, so a
/// test text between the clean prefix and the filler and sentinel shows
/// the same per-update colours as its control written cleanly), S-0616
/// claim 29 and F16 (the colour steps over the list's entries), Edge cases
/// (off: the parenthesis-aware reading's own control)
#[test]
fn colour_steps_follow_the_reading() {
    const PREFIX: &str = "(250,0,0),(0,250,0),";
    const SENTINEL: Rgb = [0, 0, 250];
    let ty = |name: &str, list: &str| {
        format!(
            "[{name}]\nBehavesLike=Spark\nColorSpeed=2.0\nMaxEC=100\nColorList={PREFIX}{list}\n"
        )
    };
    let rules = [
        "[Particles]\n0=Test\n1=Control\n2=Hex\n3=HexControl\n4=Wrap\n5=WrapControl\n6=GroupControl\n",
        // R16's example: one filler item makes the test text's items a whole number of threes.
        &ty("Test", "(200,100),(50,150,250),9),(0,0,250)"),
        &ty("Control", "(200,100,0),(50,250,9),(0,0,250)"),
        &ty("Hex", "(1Fh,$20,-1),(0,0,250)"),
        &ty("HexControl", "(1,0,255),(0,0,250)"),
        &ty("Wrap", "(300,-56,4294967340),(0,0,250)"),
        &ty("WrapControl", "(44,200,44),(0,0,250)"),
        &ty("GroupControl", "(200,100,0),(50,150,250),(0,0,250)"),
    ]
    .concat();
    let on = particles(&rules, true);
    for (test, control) in [
        ("Test", "Control"),
        ("Hex", "HexControl"),
        ("Wrap", "WrapControl"),
    ] {
        let a = shown(named(&on, test), SENTINEL);
        assert_eq!(a, shown(named(&on, control), SENTINEL), "{test}");
    }
    assert_eq!(
        shown(named(&on, "Test"), SENTINEL),
        [[0, 250, 0], [200, 100, 0], [50, 250, 9], [0, 0, 250]]
    );
    // Off: the groups `(200,100)` and `(50,150,250)` are entries and the filler is outside every group.
    let off = particles(&rules, false);
    assert_eq!(
        shown(named(&off, "Test"), SENTINEL),
        shown(named(&off, "GroupControl"), SENTINEL)
    );
    assert_ne!(
        shown(named(&off, "Test"), SENTINEL),
        shown(named(&on, "Test"), SENTINEL)
    );
}
