use std::cell::Cell;

use glasnost_fmt::mix::{HeaderCipher, MixWriter, WriteLayout};
use glasnost_fmt::mix_key::RsaKeyPair;

use super::*;
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
fn placeholder_precedence_is_mount_order() {
    let (vfs, _) = mounted();
    let read = |name: &str| pollster::block_on(vfs.read(&vfs.find(name).unwrap())).unwrap();
    // Q-0002 placeholder: a.mix, then its nested archive, then b.MIX.
    assert_eq!(read("shared.dat"), b"from-a");
    assert_eq!(read("deep.dat"), b"deep");
    assert_eq!(read("ONLY-A.DAT"), [1, 2, 3]);
    assert_eq!(vfs.find_all(id_hash("shared.dat")).len(), 3);
    assert!(vfs.find("missing.dat").is_none());
}

#[test]
fn named_precedence_reorders_lookups() {
    let (mut vfs, _) = mounted();
    vfs.set_precedence(Box::new(NamedPrecedence {
        names: vec!["b.mix".into()],
    }));
    let got = pollster::block_on(vfs.read(&vfs.find("shared.dat").unwrap())).unwrap();
    assert_eq!(got, b"from-b");
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
