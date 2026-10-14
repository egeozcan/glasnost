//! The client's screens without a window or an install: the menu, the
//! setup and the start job, on a synthetic install.

use super::*;
use glasnost_convert::find::Listing;
use glasnost_fmt::mix::MixEntry;
use glasnost_ui::setup::{MapEntry, SetupRules};
use glasnost_vfs::ArchiveId;

fn click(c: &mut Client, r: Rect) {
    let p = Point::new(r.x + r.w / 2, r.y + r.h / 2);
    c.pointer_down(Button::Left, p);
    c.pointer_up(Button::Left, p, Modifiers::default());
}

fn synthetic_install() -> Install {
    let file = FileRef {
        archive: ArchiveId(0),
        entry: MixEntry {
            id: 1,
            offset: 0,
            size: 0,
        },
    };
    Install {
        setup: SetupRules {
            money: 5000,
            money_range: (1000, 9000, 500),
            units: 3,
            units_range: (1, 9),
            bases: true,
            flags: [false, true, true, true, true],
            game_speed: 3,
            countries: vec!["Alphaland".into(), "Betaland".into()],
            colours: vec![[200, 0, 0, 255], [0, 0, 200, 255]],
        },
        maps: vec![(
            Listing {
                file,
                archive: "synthetic".into(),
                name: "Two".into(),
                theater: "x".into(),
                multiplayer: true,
            },
            MapEntry {
                label: "Two (2)".into(),
                starts: 2,
            },
        )],
        names: Default::default(),
        sounds: None,
        speech: None,
        bank: Default::default(),
        summary: "synthetic".into(),
    }
}

/// The menu offers the install picker before an install is read, and the
/// setup's Start hands out a start job with the lobby's choices.
/// spec: S-0412 claim 1 (the setup screen's values from the rules)
#[test]
fn menu_setup_and_start_hand_out_jobs() {
    let mut c = Client::new();
    let _ = c.frame_rgba();
    let install = c.menu_targets.rect_of(&MenuHit::Install).unwrap();
    assert!(
        c.menu_targets.rect_of(&MenuHit::Skirmish).is_none(),
        "no skirmish without an install"
    );
    click(&mut c, install);
    assert!(c.pick_install);

    c.done(Done::Prepared(Vfs::default(), Ok(Box::new(synthetic_install()))));
    assert!(c.status.contains("synthetic"));
    let _ = c.frame_rgba();
    let skirmish = c.menu_targets.rect_of(&MenuHit::Skirmish).unwrap();
    click(&mut c, skirmish);
    assert_eq!(c.shell.screen, Screen::Setup);
    let _ = c.frame_rgba();
    let go = c.setup_targets.rect_of(&SetupHit::Start_).unwrap();
    c.next_seed = 42;
    click(&mut c, go);
    match c.job.take() {
        Some(Job::Start {
            lobby,
            seed,
            offered,
            ..
        }) => {
            assert_eq!(seed, 42);
            assert_eq!(lobby.map, 0);
            assert_eq!(offered.len(), 2);
        }
        _ => panic!("Start hands out a start job"),
    }
    assert!(c.vfs.is_none(), "the job holds the file system");
    let px = c.frame_rgba();
    assert_eq!(px.len(), (c.size.0 * c.size.1 * 4) as usize);
    // The map does not load: back to the setup, the file system kept.
    c.done(Done::Started(
        Vfs::default(),
        Box::new(Err("no map".into())),
        glasnost_audio::bank::SoundBank::new(),
    ));
    assert_eq!(c.shell.screen, Screen::Setup);
    assert!(c.vfs.is_some());
    assert_eq!(c.status, "no map");
}
