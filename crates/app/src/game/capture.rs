//! Pathing reports (M8 A4): the client keeps, while a game runs, a short
//! ring of snapshots of the game (its own save bytes, `Game::save`), and on
//! the person's key it writes a *pathing report*: the oldest snapshot in
//! the window, every order of a person since then, the state hash at the
//! capture, the build and the game's hashes, and what the person was
//! looking at. The simulation is deterministic, so the replay tool
//! (`glasnost-headless path-report`) replays the window from the snapshot
//! and recomputes everything that happened in it, with the movement trace
//! on (`glasnost_sim::trace`).
//!
//! Orders need no ring of their own: every person's order of a skirmish or
//! of a network game goes through `Game::issue` and so into the game's
//! replay, and the snapshot is taken right after a tick, before any order
//! of the next. The ring and the report live in the client: nothing here
//! reaches the simulation, and no hash changes.
//!
//! Our own diagnostics; no spec applies.

use std::collections::VecDeque;

use glasnost_ai::game::Game;
use glasnost_map::CellXY;
use glasnost_rules::layers::Layers;
use glasnost_sim::orders::Order;
use glasnost_sim::world::HouseId;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// The game time the ring covers at least, in milliseconds.
// nocite: our own diagnostics (the brief's 20 seconds)
pub const WINDOW_MS: i64 = 20_000;

/// Snapshots per window: one is taken every window / (this − 1) ticks, so
/// the oldest kept is between one window and one and a half windows old.
// nocite: our own diagnostics (ring spacing)
pub const PER_WINDOW: i32 = 3;

/// The most snapshots ever held (a slowed game keeps no more).
// nocite: our own diagnostics (a bound on the memory held)
pub const MAX_SNAPSHOTS: usize = 6;

/// The clicks remembered.
// nocite: our own diagnostics (the last few clicks)
pub const CLICKS: usize = 8;

/// The marks remembered.
// nocite: our own diagnostics (a bound on the marks held)
pub const MARKS: usize = 32;

/// The file's first bytes.
// nocite: our own file format
pub const MAGIC: &[u8; 4] = b"GPTH";

/// The format version of a pathing report. Bump it whenever the layout of
/// [`Bundle`] changes; the snapshot inside carries the game's own format
/// versions. Version 2 (M8 step C7) added [`Bundle::layers`]; a version 1
/// report reads with no layer, which is what every version 1 game had.
// nocite: our own file format
pub const VERSION: u32 = 2;

/// The oldest format version still read.
// nocite: our own file format
pub const OLDEST_VERSION: u32 = 1;

/// The folder, under the working directory, of the native client's
/// reports.
// nocite: our own diagnostics
pub const DIR: &str = "path-reports";

/// The file name extension.
// nocite: our own file format
pub const EXTENSION: &str = "gpath";

/// The window in ticks at game-speed index `speed`: at least
/// [`WINDOW_MS`] of paced game time.
pub fn window_ticks(speed: u8) -> i32 {
    let p = glasnost_ui::setup::period_ms(speed).max(1);
    ((WINDOW_MS + p - 1) / p) as i32
}

/// The ticks between snapshots at game-speed index `speed`.
pub fn spacing(speed: u8) -> i32 {
    let w = window_ticks(speed);
    let k = PER_WINDOW - 1;
    ((w + k - 1) / k).max(1)
}

/// One snapshot: the game's save bytes right after a tick.
#[derive(Clone, Debug)]
pub struct Snapshot {
    /// Ticks run when it was taken.
    pub ticks_run: i32,
    /// The world's tick number then (the tick the next orders belong to).
    pub tick: i32,
    /// `Game::save`.
    pub bytes: Vec<u8>,
}

/// A battlefield click the person made.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Click {
    /// The world's tick number.
    pub tick: i32,
    /// The right button (else the left).
    pub right: bool,
    /// The cell under the pointer.
    pub cell: CellXY,
    /// The object under the pointer.
    pub object: Option<u32>,
}

/// The view: its origin and size in battlefield pixels and the cells under
/// its four corners (top left, top right, bottom left, bottom right).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewRect {
    /// The top left in battlefield pixels.
    pub origin: (i32, i32),
    /// The size in pixels.
    pub size: (u32, u32),
    /// The cells under the corners.
    pub corners: [Option<CellXY>; 4],
}

/// What the person was looking at when the report was saved.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Look {
    /// The person's house; `None` in a watch game, where no house is
    /// played (an older report's number reads as `Some`).
    pub local: Option<u8>,
    /// The selection, in selection order (object ids).
    pub selection: Vec<u32>,
    /// The view.
    pub view: ViewRect,
    /// The cell under the pointer, when it is over the battlefield.
    pub cursor: Option<CellXY>,
    /// The last clicks, oldest first.
    pub clicks: Vec<Click>,
    /// The game-speed index.
    pub speed: u8,
}

/// A pathing report.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bundle {
    /// The build that wrote it ([`build_id`]).
    pub build: String,
    /// The rules' hash.
    pub rules_hash: String,
    /// The map's content hash.
    pub map_hash: String,
    /// The setup's hash (`glasnost_net::start::setup_hash`).
    pub setup_hash: String,
    /// Our own rules patch layers the game was played with (M8 step C7):
    /// whether *Pathing and other fixes* was on. They are in `rules_hash`
    /// too; this says it in words.
    pub layers: Layers,
    /// Whether it is a network game.
    pub network: bool,
    /// Ticks run at the snapshot.
    pub snapshot_ticks_run: i32,
    /// The world's tick number at the snapshot.
    pub snapshot_tick: i32,
    /// The snapshot (`Game::save`).
    pub snapshot: Vec<u8>,
    /// Every person's order taken since the snapshot: (tick, house, order),
    /// in the order the game took them.
    pub orders: Vec<(i32, HouseId, Order)>,
    /// Ticks run at the capture.
    pub capture_ticks_run: i32,
    /// The world's tick number at the capture (orders of this tick taken
    /// before the capture are in `orders`).
    pub capture_tick: i32,
    /// The game's state hash at the capture.
    pub capture_hash: String,
    /// The marked ticks inside the window, oldest first.
    pub marks: Vec<i32>,
    /// What the person was looking at.
    pub look: Look,
}

/// The build's id: the client's build string and the game's format
/// versions.
pub fn build_id() -> String {
    format!(
        "{} (save format {}, game format {})",
        super::client::BUILD,
        glasnost_sim::save::FORMAT_VERSION,
        glasnost_ai::game::GAME_FORMAT_VERSION
    )
}

impl Bundle {
    /// The file's bytes: [`MAGIC`], [`VERSION`] (4 bytes, little-endian),
    /// the SHA-256 of the body, then the body (postcard).
    pub fn to_bytes(&self) -> Vec<u8> {
        let body = postcard::to_allocvec(self).expect("a pathing report serializes");
        let mut out = Vec::with_capacity(body.len() + 40); // nocite: our own file format (the header's size)
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&VERSION.to_le_bytes());
        out.extend_from_slice(&Sha256::digest(&body));
        out.extend_from_slice(&body);
        out
    }

    /// Reads a file made by [`Bundle::to_bytes`]; says why not, in words
    /// for a person.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        // nocite: our own file format (magic 4, version 4, digest 32)
        let (head, rest) = bytes.split_at_checked(8).ok_or(
            "this is not a pathing report: the file is too short for its header".to_owned(),
        )?;
        if &head[..4] != MAGIC {
            return Err("this is not a pathing report: it does not start with GPTH".into());
        }
        let version = u32::from_le_bytes([head[4], head[5], head[6], head[7]]);
        if !(OLDEST_VERSION..=VERSION).contains(&version) {
            return Err(format!(
                "this pathing report has format version {version}; this build reads versions {OLDEST_VERSION} to {VERSION}"
            ));
        }
        let (digest, body) = rest
            .split_at_checked(32)
            .ok_or("the pathing report is damaged: it ends inside its header".to_owned())?;
        if Sha256::digest(body).as_slice() != digest {
            return Err(
                "the pathing report is damaged: its checksum does not match its contents".into(),
            );
        }
        let damaged = |e: postcard::Error| {
            format!("the pathing report is damaged: its contents do not read ({e})")
        };
        let (b, tail) = if version == OLDEST_VERSION {
            let (v1, tail) = postcard::take_from_bytes::<BundleV1>(body).map_err(damaged)?;
            (v1.into(), tail)
        } else {
            postcard::take_from_bytes::<Bundle>(body).map_err(damaged)?
        };
        if !tail.is_empty() {
            return Err("the pathing report is damaged: it has bytes past its end".into());
        }
        if b.capture_ticks_run < b.snapshot_ticks_run {
            return Err("the pathing report is damaged: its capture is before its snapshot".into());
        }
        Ok(b)
    }

    /// The file name: `path-<capture tick>-<n>.gpath`.
    pub fn file_name(&self, n: u32) -> String {
        format!("path-{}-{n}.{EXTENSION}", self.capture_tick)
    }
}

/// A version 1 report's body: a [`Bundle`] without its layers.
#[derive(Deserialize)]
#[cfg_attr(test, derive(Serialize))]
struct BundleV1 {
    build: String,
    rules_hash: String,
    map_hash: String,
    setup_hash: String,
    network: bool,
    snapshot_ticks_run: i32,
    snapshot_tick: i32,
    snapshot: Vec<u8>,
    orders: Vec<(i32, HouseId, Order)>,
    capture_ticks_run: i32,
    capture_tick: i32,
    capture_hash: String,
    marks: Vec<i32>,
    look: Look,
}

impl From<BundleV1> for Bundle {
    fn from(b: BundleV1) -> Self {
        Self {
            build: b.build,
            rules_hash: b.rules_hash,
            map_hash: b.map_hash,
            setup_hash: b.setup_hash,
            layers: Layers::REFERENCE,
            network: b.network,
            snapshot_ticks_run: b.snapshot_ticks_run,
            snapshot_tick: b.snapshot_tick,
            snapshot: b.snapshot,
            orders: b.orders,
            capture_ticks_run: b.capture_ticks_run,
            capture_tick: b.capture_tick,
            capture_hash: b.capture_hash,
            marks: b.marks,
            look: b.look,
        }
    }
}

/// The ring.
#[derive(Clone, Debug, Default)]
pub struct Ring {
    snapshots: VecDeque<Snapshot>,
    marks: VecDeque<i32>,
    clicks: VecDeque<Click>,
    /// Reports made in this game.
    pub saved: u32,
    /// The game-speed index the game is paced at (the session sets it
    /// before each tick; a network game's is the room's).
    pub speed: u8,
    /// Our own rules patch layers of the game (the session sets them when
    /// the game starts and keeps them when the ring starts again).
    pub layers: Layers,
}

impl Ring {
    /// A ring for a game that starts now: its first snapshot.
    pub fn new(g: &Game) -> Self {
        let mut r = Self {
            speed: g.world.game_speed,
            ..Self::default()
        };
        r.take(g);
        r
    }

    fn take(&mut self, g: &Game) {
        self.snapshots.push_back(Snapshot {
            ticks_run: glasnost_sim::tick::ticks_run(&g.world),
            tick: g.world.tick,
            bytes: g.save(),
        });
    }

    /// After a tick ran: a snapshot when one is due at the ring's speed,
    /// and the snapshots no longer needed are dropped. Returns whether it
    /// took one.
    pub fn after_tick(&mut self, g: &Game) -> bool {
        let speed = self.speed;
        let now = glasnost_sim::tick::ticks_run(&g.world);
        let due = self
            .snapshots
            .back()
            .is_none_or(|s| now - s.ticks_run >= spacing(speed));
        if due {
            self.take(g);
        }
        let window = window_ticks(speed);
        // The oldest goes once the next one already covers the window.
        while self.snapshots.len() > 1
            && (now - self.snapshots[1].ticks_run >= window || self.snapshots.len() > MAX_SNAPSHOTS)
        {
            self.snapshots.pop_front();
        }
        let oldest = self.snapshots.front().map_or(0, |s| s.tick);
        while self.marks.front().is_some_and(|&m| m < oldest) {
            self.marks.pop_front();
        }
        due
    }

    /// Notes a mark at tick `tick`.
    pub fn mark(&mut self, tick: i32) {
        self.marks.push_back(tick);
        if self.marks.len() > MARKS {
            self.marks.pop_front();
        }
    }

    /// Notes a battlefield click.
    pub fn click(&mut self, c: Click) {
        self.clicks.push_back(c);
        if self.clicks.len() > CLICKS {
            self.clicks.pop_front();
        }
    }

    /// The snapshots held.
    pub fn snapshots(&self) -> impl Iterator<Item = &Snapshot> {
        self.snapshots.iter()
    }

    /// The bytes the ring holds.
    pub fn bytes_held(&self) -> usize {
        self.snapshots.iter().map(|s| s.bytes.len()).sum()
    }

    /// The game time the ring covers now, in ticks.
    pub fn covered(&self, g: &Game) -> i32 {
        self.snapshots
            .front()
            .map_or(0, |s| glasnost_sim::tick::ticks_run(&g.world) - s.ticks_run)
    }

    /// The report of game `g` now, with what the person looks at. `None`
    /// before the first snapshot.
    pub fn bundle(&self, g: &Game, network: bool, mut look: Look) -> Option<Bundle> {
        let s = self.snapshots.front()?;
        let replay = g.replay();
        let from = replay.orders.partition_point(|o| o.0 < s.tick);
        look.clicks = self.clicks.iter().copied().collect();
        Some(Bundle {
            build: build_id(),
            rules_hash: g.world.rules().hash.clone(),
            map_hash: replay.map.clone(),
            setup_hash: glasnost_net::start::setup_hash(&replay.setup),
            layers: self.layers,
            network,
            snapshot_ticks_run: s.ticks_run,
            snapshot_tick: s.tick,
            snapshot: s.bytes.clone(),
            orders: replay.orders[from..].to_vec(),
            capture_ticks_run: glasnost_sim::tick::ticks_run(&g.world),
            capture_tick: g.world.tick,
            capture_hash: g.state_hash(),
            marks: self.marks.iter().copied().collect(),
            look,
        })
    }
}

/// Makes the pathing report of `play` now and stores it with `save`: the
/// status text to show, and the report when it was stored.
pub fn save_report(
    play: &mut super::play::Play,
    network: bool,
    look: Look,
    save: Option<super::report::SaveReport>,
) -> (String, Option<Bundle>) {
    let Some(save) = save else {
        return ("This client cannot save a pathing report.".into(), None);
    };
    let Some(b) = play.capture.bundle(&play.game, network, look) else {
        return ("There is no pathing report to save yet.".into(), None);
    };
    play.capture.saved += 1;
    let name = b.file_name(play.capture.saved);
    match save(&name, &b.to_bytes()) {
        Ok(place) => (
            format!(
                "Pathing report of ticks {} to {} saved to {place}.",
                b.snapshot_tick, b.capture_tick
            ),
            Some(b),
        ),
        Err(why) => (format!("The pathing report was not saved: {why}."), None),
    }
}

/// Writes a report natively to `path-reports/<name>` under the working
/// directory, making the folder if need be; a name already taken gets a
/// number before its extension. Returns the path written.
#[cfg(not(target_arch = "wasm32"))]
pub fn save_file(name: &str, bytes: &[u8]) -> Result<String, String> {
    let dir = std::path::Path::new(DIR);
    std::fs::create_dir_all(dir).map_err(|e| format!("cannot make {}: {e}", dir.display()))?;
    let stem = name.strip_suffix(&format!(".{EXTENSION}")).unwrap_or(name);
    let mut path = dir.join(name);
    let mut k = 1;
    while path.exists() {
        k += 1;
        path = dir.join(format!("{stem}-{k}.{EXTENSION}"));
    }
    std::fs::write(&path, bytes).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    Ok(path.display().to_string())
}

/// The platform's own way to store a report, where it has one built in: a
/// file natively. The web shell passes its own (a download).
pub fn platform_save() -> Option<super::report::SaveReport> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        Some(save_file)
    }
    #[cfg(target_arch = "wasm32")]
    {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bundle(layers: Layers) -> Bundle {
        Bundle {
            build: "test".into(),
            rules_hash: "rules".into(),
            map_hash: "map".into(),
            setup_hash: "setup".into(),
            layers,
            network: false,
            snapshot_ticks_run: 1,
            snapshot_tick: 1,
            snapshot: vec![1, 2, 3],
            orders: Vec::new(),
            capture_ticks_run: 2,
            capture_tick: 2,
            capture_hash: "state".into(),
            marks: vec![2],
            look: Look::default(),
        }
    }

    /// A report keeps the game's layers (M8 step C7, version 2).
    #[test]
    fn a_report_keeps_its_layers() {
        for layers in [Layers::REFERENCE, Layers::PLAY_FIXES] {
            let b = bundle(layers);
            assert_eq!(Bundle::from_bytes(&b.to_bytes()), Ok(b));
        }
    }

    /// A version 1 report (before the layers) still reads, with no layer:
    /// no version 1 game had one.
    #[test]
    fn a_version_1_report_reads_with_no_layer() {
        let b = bundle(Layers::REFERENCE);
        let v1 = BundleV1 {
            build: b.build.clone(),
            rules_hash: b.rules_hash.clone(),
            map_hash: b.map_hash.clone(),
            setup_hash: b.setup_hash.clone(),
            network: b.network,
            snapshot_ticks_run: b.snapshot_ticks_run,
            snapshot_tick: b.snapshot_tick,
            snapshot: b.snapshot.clone(),
            orders: b.orders.clone(),
            capture_ticks_run: b.capture_ticks_run,
            capture_tick: b.capture_tick,
            capture_hash: b.capture_hash.clone(),
            marks: b.marks.clone(),
            look: b.look.clone(),
        };
        let body = postcard::to_allocvec(&v1).unwrap();
        let mut bytes = MAGIC.to_vec();
        bytes.extend_from_slice(&1u32.to_le_bytes());
        bytes.extend_from_slice(&Sha256::digest(&body));
        bytes.extend_from_slice(&body);
        assert_eq!(Bundle::from_bytes(&bytes), Ok(b));
        bytes[4] = 3;
        assert!(
            Bundle::from_bytes(&bytes)
                .unwrap_err()
                .contains("version 3")
        );
    }
}
