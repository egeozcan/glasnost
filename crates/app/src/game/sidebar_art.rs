//! The install's sidebar button images (S-0411 claim 19): the sell,
//! repair, four tab and two scroll buttons, each from its own image file,
//! drawn with the dedicated sidebar palette.
//!
//! Drawing them is optional (S-0411 Summary); our engine draws them when
//! the palette and a file resolve, and our own symbol for a button whose
//! file or needed frame does not (claim 19, *Layout*). Each game reads them
//! afresh at its start, after the local player's side group is mounted
//! (S-0005 claims 13 and 13a; claim 22 discards the sidebar palette at
//! every side-group mount). Whether the original keeps the pictures for
//! the session is not claimed (S-0411 claim 19), so reading them per game
//! is our design.

use glasnost_fmt::image::IndexedImage;
use glasnost_fmt::pal::Palette6;
use glasnost_fmt::shp::Shp;
use glasnost_ui::hud::{ButtonArt, ButtonImage, TABS as TAB_COUNT};
use glasnost_vfs::Vfs;
use glasnost_vfs::theater::Theater;

/// The sell button's image (S-0411 Import mapping).
pub const SELL: &str = "SELL.SHP"; // spec: S-0411 Import mapping (SELL.SHP, the sell button image)
/// The repair button's image (S-0411 Import mapping).
pub const REPAIR: &str = "REPAIR.SHP"; // spec: S-0411 Import mapping (REPAIR.SHP, the repair button image)
/// The tab images, by the tab's zero-based position in claim 1's order
/// (S-0411 Import mapping).
pub const TABS: [&str; 4] = ["TAB00.SHP", "TAB01.SHP", "TAB02.SHP", "TAB03.SHP"]; // spec: S-0411 Import mapping (TAB00.SHP to TAB03.SHP, the two digits the tab's zero-based position in claim 1's order)
/// The scroll-up and scroll-down images (S-0411 Import mapping).
pub const SCROLL: [&str; 2] = ["R-UP.SHP", "R-DN.SHP"]; // spec: S-0411 Import mapping (R-UP.SHP and R-DN.SHP, the scroll-up and scroll-down images)
/// The palette of all of them, the same name for every side (S-0411
/// Import mapping).
pub const PALETTE: &str = "SIDEBAR.PAL"; // spec: S-0411 Import mapping (SIDEBAR.PAL, fixed file name for every side, no rules selector)

/// Every image file of claim 19, sell and repair first, then the tabs in
/// order, then scroll up and down.
pub const FILES: [&str; 8] = [
    SELL, REPAIR, TABS[0], TABS[1], TABS[2], TABS[3], SCROLL[0], SCROLL[1],
];

/// The frames claim 19 uses of each file of [`FILES`]: 0 and 1 of the
/// sell and repair images, 0 to 4 of a tab's, 0 to 2 of a scroll button's.
const FRAMES_USED: [usize; 8] = [2, 2, 5, 5, 5, 5, 3, 3]; // spec: S-0411 claim 19 and Import mapping (sell, repair: frames 0 and 1; a tab: frames 0 to 4; scroll: frames 0 to 2)

/// The palette file's size: 256 entries of a red, a green and a blue
/// byte (S-0411 Import mapping).
const PALETTE_BYTES: usize = 768; // spec: S-0411 Import mapping (SIDEBAR.PAL: 768-byte file of 256 entries, each a red, a green and a blue byte)

/// One image file as read: its canvas size and the frames claim 19 uses,
/// each on its canvas (`None`: the frame is missing or does not decode).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ButtonFile {
    /// The canvas size.
    pub size: (i32, i32),
    /// Frames 0, 1, … of the file, as many as claim 19 uses.
    pub frames: Vec<Option<IndexedImage>>,
}

/// The sidebar button images of a game.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SidebarArt {
    /// The sidebar palette as the painter's RGBA rows.
    pub palette: [u8; 1024],
    /// The files of [`FILES`], in that order; `None` for one that does not
    /// resolve or parse.
    pub files: [Option<ButtonFile>; 8],
}

/// The sidebar palette's colours from its file (S-0411 Import mapping:
/// file byte *c* gives the component (4·*c*) mod 256), as RGBA. `None`
/// for a file that is not 768 bytes.
pub fn palette_rgba(file: &[u8]) -> Option<[u8; 1024]> {
    if file.len() != PALETTE_BYTES {
        return None;
    }
    let pal = Palette6::parse(file).ok()?;
    let mut out = [0u8; 1024];
    for (dst, src) in out.as_chunks_mut::<4>().0.iter_mut().zip(pal.colors.iter()) {
        // spec: S-0411 Import mapping (SIDEBAR.PAL: a file byte c gives the palette component (4·c) mod 256)
        let [r, g, b] = src.map(|c| c.wrapping_mul(4));
        *dst = [r, g, b, 255];
    }
    Some(out)
}

/// A file resolved by the ordinary precedence (S-0005 claims 1 and 7) and
/// read.
async fn read_file(vfs: &Vfs, name: &str) -> Option<Vec<u8>> {
    let r = vfs.resolve(name)?;
    vfs.read_resolved(&r).await.ok()
}

/// Reads one image file and the first `used` frames on its canvas.
async fn read_button(vfs: &Vfs, name: &str, used: usize) -> Option<ButtonFile> {
    let bytes = read_file(vfs, name).await?;
    let shp = Shp::parse(&bytes).ok()?;
    let size = (i32::from(shp.width), i32::from(shp.height));
    let frames = (0..used)
        .map(|f| {
            shp.decode_on_canvas(f)
                .ok()
                .filter(|i| i.width() > 0 && i.height() > 0)
        })
        .collect();
    Some(ButtonFile { size, frames })
}

/// Reads the button images and their palette through the search order of
/// `vfs` as it stands. `None` when the palette does not resolve or is not
/// a palette file: every button then draws our own symbol (S-0411 Import
/// mapping, SIDEBAR.PAL's unresolved row).
pub async fn read(vfs: &Vfs) -> Option<SidebarArt> {
    // spec: S-0411 Import mapping (SIDEBAR.PAL: unresolved file, our engine's own symbols, claim 19)
    let palette = palette_rgba(&read_file(vfs, PALETTE).await?)?;
    let mut files: [Option<ButtonFile>; 8] = Default::default();
    for (slot, (name, used)) in files.iter_mut().zip(FILES.iter().zip(FRAMES_USED)) {
        // spec: S-0411 claim 19 (each file resolved by the ordinary precedence of S-0005 claims 1 and 7)
        *slot = read_button(vfs, name, used).await;
    }
    Some(SidebarArt { palette, files })
}

/// Mounts the side group of side index `side` for the rest of the game
/// (S-0005 claims 13 and 13a): the local player's, which S-0411 claim 19
/// resolves the button images with. The game's rules and art were read
/// before this with the importer's side group, so nothing the simulation
/// reads changes.
pub async fn mount_side(vfs: &mut Vfs, theater: Theater, side: u32) {
    // spec: S-0005 claim 13 (the side group of the local player's side, released and mounted again at the start of every game)
    let report = vfs.mount_game(theater, Some(side)).await;
    if report.never_reaches_play {
        log::warn!(
            "the local player's side archive {:?} cannot be resolved (S-0005 claims 13a and 16)",
            report.side_number
        );
    }
    // spec: S-0005 claim 13 (the loading archives are not in the search order during play)
    vfs.finish_loading();
}

/// The first sprite id of the button images; file *k* of [`FILES`] takes
/// id `BUTTON_SPRITES + k`, its frame *f* the sprite's frame *f*.
// nocite: our own sprite id, between the cursor's and the icons'
pub const BUTTON_SPRITES: u32 = 910;

/// The button images as the sidebar model takes them, for the frames that
/// `registered` accepts (file index, frame).
pub fn button_art(art: &SidebarArt, registered: impl Fn(usize, usize) -> bool) -> ButtonArt {
    let image = |k: usize| {
        let f = art.files[k].as_ref()?;
        let mut frames = 0u32;
        for (i, img) in f.frames.iter().enumerate() {
            if img.is_some() && registered(k, i) {
                frames |= 1 << i;
            }
        }
        Some(ButtonImage {
            id: BUTTON_SPRITES + k as u32,
            size: f.size,
            frames,
        })
    };
    let mut out = ButtonArt {
        sell: image(0),
        repair: image(1),
        ..ButtonArt::default()
    };
    for t in 0..TAB_COUNT {
        out.tabs[t] = image(2 + t);
    }
    out.scroll = [image(6), image(7)];
    out
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use glasnost_fmt::mix::{MixWriter, WriteLayout, id_hash};
    use glasnost_fmt::shp::{ShpWriter, WriteFrame};
    use glasnost_vfs::{GameOptions, MemSource, PickedFile};

    use super::*;

    /// A synthetic image file: a `canvas`, `frames` frames each filled with
    /// index frame + 1. Sizes and contents are invented.
    fn shp(canvas: (u16, u16), frames: usize) -> Vec<u8> {
        ShpWriter {
            width: canvas.0,
            height: canvas.1,
            frames: (0..frames)
                .map(|i| WriteFrame {
                    x: 0,
                    y: 0,
                    image: IndexedImage::new(
                        u32::from(canvas.0),
                        u32::from(canvas.1),
                        vec![i as u8 + 1; usize::from(canvas.0) * usize::from(canvas.1)],
                    )
                    .unwrap(),
                    compression: 1,
                })
                .collect(),
        }
        .write()
    }

    fn mix(files: &[(&str, Vec<u8>)]) -> Vec<u8> {
        let mut files: Vec<_> = files.iter().map(|(n, b)| (id_hash(n), b.clone())).collect();
        files.push((id_hash("filler.ini"), b"[Filler]\n".to_vec()));
        MixWriter {
            layout: WriteLayout::Plain,
            files,
        }
        .write()
    }

    /// A mounted install: `main` in the main archive, and the side archives
    /// of side numbers 1 and 2 holding `side1` and `side2`.
    fn install(
        main: &[(&str, Vec<u8>)],
        side1: &[(&str, Vec<u8>)],
        side2: &[(&str, Vec<u8>)],
    ) -> Vfs {
        let picked = |path: &str, bytes: Vec<u8>| PickedFile {
            path: path.into(),
            source: Rc::new(MemSource(bytes)),
        };
        let mut vfs = Vfs::default();
        pollster::block_on(vfs.mount_install(vec![
            picked("ra2md.mix", mix(main)),
            picked("sidec01.mix", mix(side1)),
            picked("sidec02.mix", mix(side2)),
        ]));
        pollster::block_on(vfs.mount_startup(GameOptions::default()));
        vfs
    }

    fn palette() -> Vec<u8> {
        (0..768u32).map(|i| (i % 64) as u8).collect()
    }

    /// spec: S-0411 Import mapping (SIDEBAR.PAL: 768 bytes, a file byte c gives the component (4·c) mod 256)
    #[test]
    fn the_sidebar_palette_converts_each_byte_by_four() {
        let mut file = vec![0u8; 768];
        file[..3].copy_from_slice(&[1, 63, 64]);
        file[765..].copy_from_slice(&[255, 70, 0]);
        let p = palette_rgba(&file).unwrap();
        assert_eq!(p[..4], [4, 252, 0, 255], "64 · 4 wraps to 0");
        assert_eq!(
            p[1020..],
            [252, 24, 0, 255],
            "255 · 4 mod 256 = 252, 70 · 4 mod 256 = 24"
        );
        assert_eq!(palette_rgba(&[0u8; 767]), None);
        assert_eq!(palette_rgba(&[0u8; 769]), None);
    }

    /// spec: S-0411 claim 19 and Import mapping (the eight files and their palette, each resolved
    /// by the ordinary precedence; the frames each state uses; an unresolved file or frame: our
    /// symbol, and with no palette every button is ours)
    #[test]
    fn the_button_images_are_read_with_the_frames_claim_19_uses() {
        let vfs = install(
            &[
                (PALETTE, palette()),
                (SELL, shp((11, 7), 1)),
                (TABS[2], shp((9, 5), 9)),
                (SCROLL[1], shp((13, 6), 3)),
            ],
            &[],
            &[],
        );
        let art = pollster::block_on(read(&vfs)).unwrap();
        assert_eq!(
            art.palette[4..8],
            [12, 16, 20, 255],
            "entry 1: bytes 3, 4, 5"
        );
        let sell = art.files[0].as_ref().unwrap();
        assert_eq!(sell.size, (11, 7));
        assert_eq!(sell.frames.len(), 2);
        assert!(sell.frames[0].is_some());
        assert!(sell.frames[1].is_none(), "frame 1 is missing");
        assert!(art.files[1].is_none(), "no repair image");
        let tab = art.files[4].as_ref().unwrap();
        assert_eq!(tab.frames.len(), 5, "frames 0 to 4 only");
        assert_eq!(tab.frames[4].as_ref().unwrap().get(0, 0), Some(5));
        assert_eq!(art.files[7].as_ref().unwrap().frames.len(), 3);
        // The model's view: only the frames read and registered.
        let b = button_art(&art, |_, _| true);
        assert_eq!(b.sell.unwrap().frames, 0b1);
        assert_eq!(b.sell.unwrap().size, (11, 7));
        assert_eq!(b.repair, None);
        assert_eq!(b.tabs[2].unwrap().frames, 0b11111);
        assert_eq!(b.tabs[2].unwrap().id, BUTTON_SPRITES + 4);
        assert_eq!(b.scroll[1].unwrap().frames, 0b111);
        let b = button_art(&art, |k, f| !(k == 4 && f == 3));
        assert_eq!(b.tabs[2].unwrap().frames, 0b10111, "a frame not registered");
        // No palette: no images at all.
        let vfs = install(&[(SELL, shp((11, 7), 2))], &[], &[]);
        assert_eq!(pollster::block_on(read(&vfs)), None);
        // A palette file of the wrong size is no palette.
        let vfs = install(
            &[(PALETTE, vec![1; 700]), (SELL, shp((11, 7), 2))],
            &[],
            &[],
        );
        assert_eq!(pollster::block_on(read(&vfs)), None);
    }

    /// spec: S-0411 claim 19 (each file resolved with the local player's side group mounted);
    /// S-0005 claims 13 and 13a (side index s gives archive number s + 1, except that 2 gives 2)
    #[test]
    fn the_images_come_from_the_local_players_side_group() {
        let mut vfs = install(
            &[(PALETTE, palette()), (REPAIR, shp((5, 5), 2))],
            &[(SELL, shp((11, 7), 2))],
            &[(SELL, shp((15, 9), 2))],
        );
        for (side, size) in [(0, (11, 7)), (1, (15, 9)), (2, (15, 9)), (0, (11, 7))] {
            pollster::block_on(mount_side(&mut vfs, Theater::Temperate, side));
            let art = pollster::block_on(read(&vfs)).unwrap();
            assert_eq!(art.files[0].as_ref().unwrap().size, size, "side {side}");
            assert_eq!(
                art.files[1].as_ref().unwrap().size,
                (5, 5),
                "from the main archive"
            );
        }
    }
}
