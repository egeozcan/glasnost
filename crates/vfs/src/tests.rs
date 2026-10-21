use std::cell::Cell;

use glasnost_fmt::mix::{HeaderCipher, MixWriter, WriteLayout};
use glasnost_fmt::mix_key::RsaKeyPair;

use super::*;
use crate::theater::Theater;
use glasnost_fmt::mix::may_be_mix;

fn synthetic_keys() -> MixKeys {
    MixKeys {
        rsa: RsaKeyPair::synthetic().public().clone(),
        cipher: HeaderCipher::BigEndianWords,
    }
}

fn mix(layout: WriteLayout, files: &[(&str, Vec<u8>)]) -> Vec<u8> {
    MixWriter {
        layout,
        files: files.iter().map(|(n, d)| (id_hash(n), d.clone())).collect(),
    }
    .write()
}

fn encrypted() -> WriteLayout {
    WriteLayout::Encrypted {
        key: [9u8; 56],
        rsa: RsaKeyPair::synthetic(),
        cipher: HeaderCipher::BigEndianWords,
    }
}

/// Counts reads, to check that mounting stays lazy.
struct Counting {
    inner: MemSource,
    bytes_read: Cell<u64>,
}

impl ByteSource for Counting {
    fn len(&self) -> u64 {
        self.inner.len()
    }
    fn read_at(&self, offset: u64, len: usize) -> ReadFuture<'_> {
        self.bytes_read.set(self.bytes_read.get() + len as u64);
        self.inner.read_at(offset, len)
    }
}

fn picked(path: &str, bytes: Vec<u8>) -> PickedFile {
    PickedFile {
        path: path.into(),
        source: Rc::new(MemSource(bytes)),
    }
}

fn install() -> Vec<PickedFile> {
    let inner = mix(
        encrypted(),
        &[
            ("deep.dat", b"deep".to_vec()),
            ("shared.dat", b"inner".to_vec()),
        ],
    );
    let a = mix(
        WriteLayout::Plain,
        &[
            ("inner.mix", inner),
            ("shared.dat", b"from-a".to_vec()),
            ("only-a.dat", vec![1, 2, 3]),
        ],
    );
    let b = mix(WriteLayout::Classic, &[("shared.dat", b"from-b".to_vec())]);
    vec![
        picked("readme.txt", b"not an archive".to_vec()),
        picked("b.MIX", b),
        picked("a.mix", a),
        picked("sub/c.mix", vec![0; 8]),
    ]
}

fn mounted() -> (Vfs, MountReport) {
    let mut vfs = Vfs::new(synthetic_keys(), MountOptions::default());
    let report = pollster::block_on(vfs.mount_install(install()));
    (vfs, report)
}

#[test]
fn mounts_top_level_and_nested_archives() {
    let (vfs, report) = mounted();
    assert!(report.failed.is_empty(), "{:?}", report.failed);
    assert_eq!(report.top_level.len(), 2);
    assert_eq!(report.nested.len(), 1);
    let names: Vec<_> = vfs.archives().iter().map(|a| a.path.as_str()).collect();
    let inner_hex = format!("{:08X}", id_hash("inner.mix"));
    assert_eq!(
        names,
        [
            "a.mix".to_owned(),
            format!("a.mix/{inner_hex}"),
            "b.MIX".into()
        ]
    );
    assert!(vfs.archives()[1].encrypted());
    let loose: Vec<_> = vfs.loose_files().iter().map(|f| f.name.as_str()).collect();
    assert_eq!(loose, ["readme.txt", "sub/c.mix"]);
}

#[test]
fn nothing_resolves_before_the_search_order_is_built() {
    let (vfs, _) = mounted();
    assert!(vfs.search_order().is_empty());
    assert!(vfs.find("shared.dat").is_none());
    // Browsing still lists every copy.
    assert_eq!(vfs.find_all(id_hash("shared.dat")).len(), 3);
}

fn plain(files: &[(&str, Vec<u8>)]) -> Vec<u8> {
    mix(WriteLayout::Plain, files)
}

/// An invented install laid out like S-0005 Table 1 (names only; contents
/// are ours).
fn game_install(extra: Vec<PickedFile>) -> Vec<PickedFile> {
    let cachemd = plain(&[("shared.dat", b"cachemd".to_vec())]);
    let localmd = plain(&[
        ("rules.txt", b"localmd".to_vec()),
        ("only-local.dat", b"L".to_vec()),
    ]);
    let ra2md = plain(&[
        ("cachemd.mix", cachemd),
        ("localmd.mix", localmd),
        ("shared.dat", b"ra2md".to_vec()),
        ("conqmd.mix", plain(&[("unit.shp", b"conq".to_vec())])),
    ]);
    let language = plain(&[("lang.dat", b"language".to_vec())]);
    let langmd = plain(&[
        ("language.mix", language),
        ("audiomd.mix", plain(&[("sfx.dat", b"audio".to_vec())])),
        ("shared.dat", b"langmd".to_vec()),
    ]);
    let tem = plain(&[("tile.tem", b"temperate".to_vec())]);
    let sno = plain(&[("tile.tem", b"snow-group".to_vec())]);
    let mut files = vec![
        picked("langmd.mix", langmd),
        picked("RA2MD.MIX", ra2md),
        picked(
            "expandmd01.mix",
            plain(&[("rules.txt", b"expand01".to_vec())]),
        ),
        picked(
            "expandmd02.mix",
            plain(&[("rules.txt", b"expand02".to_vec())]),
        ),
        picked("ecacheB.mix", plain(&[("e.dat", b"B".to_vec())])),
        picked("ecacheA.mix", plain(&[("e.dat", b"A".to_vec())])),
        picked("elocal1.mix", plain(&[("e.dat", b"L".to_vec())])),
        picked("conquer.mix", plain(&[("c.dat", vec![1])])),
        picked("cameomd.mix", plain(&[("c.dat", vec![2])])),
        picked("cameo.mix", plain(&[("c.dat", vec![3])])),
        picked("mapsmd03.mix", plain(&[("m.dat", vec![4])])),
        picked("multimd.mix", plain(&[("m.dat", vec![5])])),
        picked("movies03.mix", plain(&[("v.dat", vec![6])])),
        picked("missions.yro", plain(&[("missions.pkt", vec![7])])),
        picked("zeta.yro", plain(&[("zeta.pkt", vec![8])])),
        picked("alpha.yro", plain(&[("alpha.pkt", vec![9])])),
        picked("loadmd.mix", plain(&[("tile.tem", b"loading".to_vec())])),
        picked("temperat.mix", tem),
        picked("snow.mix", sno),
        picked("sidec01.mix", plain(&[("side.dat", b"side1".to_vec())])),
        picked("rulesmd.ini", b"loose".to_vec()),
        picked("MAPS01.MIX", plain(&[("rules.txt", b"never".to_vec())])),
    ];
    files.extend(extra);
    files
}

fn startup(extra: Vec<PickedFile>, options: GameOptions) -> (Vfs, StartupReport) {
    let mut vfs = Vfs::new(synthetic_keys(), MountOptions::default());
    pollster::block_on(vfs.mount_install(game_install(extra)));
    let report = pollster::block_on(vfs.mount_startup(options));
    (vfs, report)
}

fn read(vfs: &Vfs, name: &str) -> Vec<u8> {
    pollster::block_on(vfs.read_resolved(&vfs.resolve(name).unwrap())).unwrap()
}

/// spec: S-0005 claims 9 to 12 and 14a, Table 1
#[test]
fn startup_order_follows_table_one() {
    let (vfs, report) = startup(vec![], GameOptions::default());
    let names: Vec<&str> = vfs.search_order().iter().map(|r| r.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "langmd.mix",
            "language.mix",
            "expandmd02.mix",
            "expandmd01.mix",
            "ra2md.mix",
            "cachemd.mix",
            "localmd.mix",
            "audiomd.mix",
            "ecacheA.mix",
            "ecacheB.mix",
            "elocal1.mix",
            "conqmd.mix",
            "conquer.mix",
            "cameomd.mix",
            "cameo.mix",
            "mapsmd03.mix",
            "multimd.mix",
            "movies03.mix",
            "alpha.yro",
            "zeta.yro",
        ]
    );
    // language.mix came from inside langmd.mix (claim 5), langmd.mix is loose.
    let rank = |n: &str| vfs.search_order().iter().find(|r| r.name == n).unwrap();
    assert!(matches!(
        rank("language.mix").origin,
        game::Origin::Member(_)
    ));
    assert!(matches!(rank("langmd.mix").origin, game::Origin::Loose));
    assert!(report.starts(), "{:?}", report.missing_required);
    // MAPS01.MIX is browsable but never searched (claim 15).
    assert_eq!(read(&vfs, "rules.txt"), b"expand02");
}

/// spec: S-0005 claims 7 and 9 (language archives first, higher expansion first)
#[test]
fn first_ranked_archive_wins() {
    let (vfs, _) = startup(vec![], GameOptions::default());
    assert_eq!(read(&vfs, "shared.dat"), b"langmd");
    assert_eq!(read(&vfs, "only-local.dat"), b"L");
    assert_eq!(read(&vfs, "e.dat"), b"A");
    assert_eq!(read(&vfs, "SFX.DAT"), b"audio");
}

/// spec: S-0005 claim 3 (a loose nested-archive name replaces the whole archive)
#[test]
fn loose_copy_replaces_a_nested_archive_whole() {
    let loose_cache = plain(&[("other.dat", b"x".to_vec())]);
    let (vfs, _) = startup(
        vec![picked("CacheMD.mix", loose_cache)],
        GameOptions::default(),
    );
    let rank = vfs
        .search_order()
        .iter()
        .find(|r| r.name == "cachemd.mix")
        .unwrap();
    assert!(matches!(rank.origin, game::Origin::Loose));
    assert_eq!(read(&vfs, "other.dat"), b"x");
}

/// spec: S-0005 claims 1 and 2; PLAN.md 4.3 (loose data files are an opt-in mod layer)
#[test]
fn loose_data_files_only_with_the_mod_layer() {
    let extra = || vec![picked("rules.TXT", b"loose-rules".to_vec())];
    let (vfs, _) = startup(extra(), GameOptions::default());
    assert_eq!(read(&vfs, "rules.txt"), b"expand02");
    assert!(vfs.shadowing_loose("RULES.txt").is_some());
    let on = GameOptions {
        mod_layer: true,
        ..GameOptions::default()
    };
    let (vfs, _) = startup(extra(), on);
    assert_eq!(read(&vfs, "rules.txt"), b"loose-rules");
    assert!(vfs.shadowing_loose("rules.txt").is_none());
    assert_eq!(read(&vfs, "RulesMD.ini"), b"loose");
}

/// spec: S-0005 claim 16 (required archives; either movie archive)
#[test]
fn missing_required_archives_are_reported() {
    let mut vfs = Vfs::new(synthetic_keys(), MountOptions::default());
    pollster::block_on(vfs.mount_install(vec![picked("langmd.mix", plain(&[("a", vec![1])]))]));
    let report = pollster::block_on(vfs.mount_startup(GameOptions::default()));
    assert!(!report.starts());
    assert_eq!(report.missing_required.len(), 7);
}

/// spec: S-0005 claims 13, 13a and 14
#[test]
fn per_game_groups() {
    let (mut vfs, _) = startup(vec![], GameOptions::default());
    let report = pollster::block_on(vfs.mount_game(Theater::Temperate, Some(0)));
    assert_eq!(report.side_number, Some(1));
    assert!(!report.never_reaches_play);
    // While loading, loadmd.mix ranks above the theater group mounted for this game.
    assert_eq!(read(&vfs, "tile.tem"), b"loading");
    vfs.finish_loading();
    assert_eq!(read(&vfs, "tile.tem"), b"temperate");
    assert_eq!(read(&vfs, "side.dat"), b"side1");
    // Same theater: the group stays, so it now ranks above the loading archives.
    pollster::block_on(vfs.mount_game(Theater::Temperate, Some(0)));
    assert_eq!(read(&vfs, "tile.tem"), b"temperate");
    // Side index 1 asks for sidec02.mix, which is absent: never reaches play.
    let report = pollster::block_on(vfs.mount_game(Theater::Snow, Some(1)));
    assert!(report.never_reaches_play);
    vfs.finish_loading();
    assert_eq!(read(&vfs, "tile.tem"), b"snow-group");
    assert!(vfs.resolve("side.dat").is_none());
    let report = pollster::block_on(vfs.mount_game(Theater::Snow, None));
    assert!(report.never_reaches_play);
}

/// spec: S-0005 claim 6 (names longer than 259 bytes are not found) and Edge cases (case clash)
#[test]
fn long_names_and_case_clashes() {
    let (vfs, _) = startup(vec![], GameOptions::default());
    assert!(vfs.resolve(&"a".repeat(260)).is_none());
    let mut vfs = Vfs::new(synthetic_keys(), MountOptions::default());
    let report = pollster::block_on(vfs.mount_install(vec![
        picked("rulesmd.ini", b"1".to_vec()),
        picked("RULESMD.INI", b"2".to_vec()),
    ]));
    assert_eq!(report.case_clashes.len(), 1);
    assert_eq!(vfs.case_clashes().len(), 1);
}

/// spec: S-0005 claim 8 (a member the bisection misses counts as absent; the search goes on)
#[test]
fn unsigned_index_hides_members() {
    use glasnost_fmt::mix::IndexOrder;
    // Two names whose ids straddle 2^31, in an index sorted as unsigned.
    let names: Vec<String> = (0..200).map(|i| format!("probe{i}.dat")).collect();
    let lo = names.iter().find(|n| id_hash(n) < 0x8000_0000).unwrap();
    let hi = names.iter().find(|n| id_hash(n) >= 0x8000_0000).unwrap();
    let bad = MixWriter {
        layout: WriteLayout::Plain,
        files: vec![(id_hash(lo), b"lo".to_vec()), (id_hash(hi), b"hi".to_vec())],
    }
    .write_ordered(IndexOrder::UnsignedAscending);
    let backup = plain(&[(lo, b"backup-lo".to_vec()), (hi, b"backup-hi".to_vec())]);
    let files = || {
        vec![
            picked("langmd.mix", bad.clone()),
            picked("ra2md.mix", backup.clone()),
        ]
    };
    let mut vfs = Vfs::new(synthetic_keys(), MountOptions::default());
    pollster::block_on(vfs.mount_install(files()));
    pollster::block_on(vfs.mount_startup(GameOptions::default()));
    let got = [read(&vfs, lo), read(&vfs, hi)];
    assert!(got.contains(&b"backup-lo".to_vec()) || got.contains(&b"backup-hi".to_vec()));
    let fixed = GameOptions {
        quirks: GameQuirks {
            mix_signed_binary_search: false,
            ..GameQuirks::default()
        },
        ..GameOptions::default()
    };
    pollster::block_on(vfs.mount_startup(fixed));
    assert_eq!(
        [read(&vfs, lo), read(&vfs, hi)],
        [b"lo".to_vec(), b"hi".to_vec()]
    );
}

#[test]
fn learning_a_name_renames_nested_archives() {
    let (mut vfs, _) = mounted();
    vfs.learn_name("Inner.mix");
    assert_eq!(vfs.archives()[1].path, "a.mix/inner.mix");
}

#[test]
fn mounting_reads_only_headers() {
    let big = vec![7u8; 1 << 20];
    let bytes = mix(WriteLayout::Plain, &[("big.dat", big)]);
    let source = Rc::new(Counting {
        inner: MemSource(bytes),
        bytes_read: Cell::new(0),
    });
    let mut vfs = Vfs::new(synthetic_keys(), MountOptions::default());
    pollster::block_on(vfs.mount_top("big.mix", source.clone())).unwrap();
    assert!(
        source.bytes_read.get() < 8192,
        "read {}",
        source.bytes_read.get()
    );
}

#[test]
fn inconsistent_archives_fail_to_mount() {
    let mut bytes = mix(WriteLayout::Plain, &[("x.dat", vec![1; 10])]);
    bytes.push(0);
    let mut vfs = Vfs::new(synthetic_keys(), MountOptions::default());
    let err = pollster::block_on(vfs.mount_top("x.mix", Rc::new(MemSource(bytes)))).unwrap_err();
    assert!(matches!(err, MountError::Format(_)), "{err}");
}

#[test]
fn depth_and_count_limits_hold() {
    // Ten levels of nesting; only max_depth of them are followed.
    let mut bytes = mix(WriteLayout::Plain, &[("leaf.dat", vec![1])]);
    for level in 0..10 {
        bytes = mix(WriteLayout::Plain, &[(&format!("level{level}.mix"), bytes)]);
    }
    let options = MountOptions {
        max_depth: 3,
        ..MountOptions::default()
    };
    let mut vfs = Vfs::new(synthetic_keys(), options);
    pollster::block_on(vfs.mount_top("top.mix", Rc::new(MemSource(bytes)))).unwrap();
    assert_eq!(vfs.archives().len(), 4);
}

#[test]
fn probe_matches_writer_output() {
    for layout in [WriteLayout::Classic, WriteLayout::Plain, encrypted()] {
        let plain = !matches!(layout, WriteLayout::Encrypted { .. });
        let bytes = mix(layout, &[("f.dat", vec![5; 33])]);
        assert!(may_be_mix(&bytes[..16], bytes.len() as u64));
        // Plain headers must imply the exact length; encrypted ones are
        // only checked by the full parse.
        assert_eq!(may_be_mix(&bytes[..16], bytes.len() as u64 - 1), !plain);
    }
    assert!(!may_be_mix(&[0, 0, 0, 0x40], 100));
    assert!(!may_be_mix(&[], 0));
}
