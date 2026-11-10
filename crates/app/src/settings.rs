//! The client's settings (M8 step C2) and where they are kept.
//!
//! Settings are presentation and input choices of the person at this
//! machine: they never reach the simulation crates and never change a state
//! hash. They are kept as a small versioned text file (`key=value` lines) in
//! the platform's configuration folder natively and in the browser's
//! `localStorage` on the web. A missing or unreadable store gives defaults;
//! a field a file lacks (an older file) or holds wrongly takes its default.

use std::path::PathBuf;

/// The format version this build writes. Version 1 holds every field below;
/// a later version may add fields, which older builds ignore.
// nocite: our own client setting (storage format)
pub const VERSION: u32 = 1;

/// The key `localStorage` keeps the text under on the web.
// nocite: our own client setting (storage name)
pub const WEB_KEY: &str = "glasnost.settings";

/// A bindable command key (S-0213 table T2 without route mode), or the
/// key that plays S-0214's type-select role.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// Stop.
    Stop,
    /// Scatter.
    Scatter,
    /// Guard.
    Guard,
    /// Deploy.
    Deploy,
    /// Cheer.
    Cheer,
    /// Route mode, held (S-0213 claims 16 and 22).
    RouteMode,
    /// The type-select role (S-0214 Terms and claim 12): held during a
    /// selecting click, or pressed.
    TypeSelect,
    /// Save a pathing report (M8 A4, our own diagnostics).
    PathReport,
    /// Mark the moment in the pathing report's window (M8 A4).
    Mark,
    /// Held with a digit, makes that control group: the web build's
    /// stand-in for Ctrl+digit, which browsers may keep for themselves
    /// (M8 A4 round 5 seams, design).
    MakeGroup,
}

/// Every bindable action, in the order the screen lists them.
pub const ACTIONS: [Action; 10] = [
    Action::Stop,
    Action::Scatter,
    Action::Guard,
    Action::Deploy,
    Action::Cheer,
    Action::RouteMode,
    Action::TypeSelect,
    Action::PathReport,
    Action::Mark,
    Action::MakeGroup,
];

impl Action {
    /// The name shown and stored.
    pub fn name(self) -> &'static str {
        match self {
            Action::Stop => "stop",
            Action::Scatter => "scatter",
            Action::Guard => "guard",
            Action::Deploy => "deploy",
            Action::Cheer => "cheer",
            Action::RouteMode => "route",
            Action::TypeSelect => "typeselect",
            Action::PathReport => "report",
            Action::Mark => "mark",
            Action::MakeGroup => "group",
        }
    }

    fn index(self) -> usize {
        ACTIONS.iter().position(|a| *a == self).unwrap_or(0)
    }
}

/// The keys of the fixed bindings, for the screen to list: the options menu
/// key and the view-scroll arrows.
// spec: S-0213 claim 23 (Escape is reserved for the options menu; the arrow keys scroll the view)
pub const FIXED_KEYS: [(&str, &str); 2] = [
    ("Options menu", "Escape"),
    ("Scroll the view", "Arrow keys"),
];

/// Our own keyboard-only keys (M8 step C3) and control-group keys (M8 A4
/// round 5), for the screen to list. None but the group digits is a
/// letter or a digit, and a digit bound to a command key keeps the
/// command (the group keys give way); none is Escape or a plain arrow in
/// running play (S-0213 claims 22 and 23 keep their meaning there).
// nocite: our own client setting (keyboard-only play of the menus and the sidebar; control groups)
pub const KEYBOARD_KEYS: [(&str, &str); 14] = [
    ("Next, previous control", "Tab, Shift+Tab"),
    ("Move in menus", "Up, Down"),
    ("Change a value", "Left, Right"),
    ("Press (menus)", "Enter, Space"),
    ("Back (menus)", "Escape"),
    ("Press (sidebar)", "Space"),
    ("Hold, cancel (sidebar)", "Backspace"),
    ("Sidebar tab", "Page Up, Page Down"),
    ("Placement cell; place", "Ctrl+arrows; Enter"),
    ("Make a control group", "Ctrl+0 to 9; web: group key+digit"),
    ("Add a group to the selection", "Shift+0 to Shift+9"),
    ("Select a group; again soon: centre", "0 to 9"),
    ("Select a group and centre", "Alt+0 to Alt+9"),
    ("Delete the route point", "Delete"),
];

/// The key bindings of the command keys. Keys are lower-case letters or
/// digits, and no two actions share one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bindings([char; 10]);

/// Why a key could not be bound.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BindError {
    /// The key is not a letter or a digit.
    NotBindable,
    /// Another action has the key.
    Taken(Action),
}

impl Default for Bindings {
    fn default() -> Self {
        // spec: S-0213 claim 22 (fresh-profile bindings: S stop, X scatter, G guard, D deploy, C cheer, Z route mode)
        // nocite: our own client setting (T plays S-0214's type-select role, whose key is the player's binding; P saves a pathing report and M marks, M8 A4: no command key of S-0213 claim 22)
        // design: N held with a digit makes a group on the web (M8 A4 round 5 seams; no command key of S-0213 claim 22)
        Self(['s', 'x', 'g', 'd', 'c', 'z', 't', 'p', 'm', 'n'])
    }
}

impl Bindings {
    /// The key of an action.
    pub fn key(&self, a: Action) -> char {
        self.0[a.index()]
    }

    /// The action a key is bound to.
    pub fn action(&self, c: char) -> Option<Action> {
        ACTIONS.iter().copied().find(|a| self.key(*a) == c)
    }

    /// Binds `c` to `a`. A key another action has is refused, not swapped.
    pub fn rebind(&mut self, a: Action, c: char) -> Result<(), BindError> {
        let c = c.to_ascii_lowercase();
        if !c.is_ascii_alphanumeric() {
            return Err(BindError::NotBindable);
        }
        match self.action(c) {
            Some(other) if other != a => Err(BindError::Taken(other)),
            _ => {
                self.0[a.index()] = c;
                Ok(())
            }
        }
    }
}

/// The scroll-speed index range: 0 fastest to 6 slowest.
// spec: S-0101 Terms (scroll index r, 0 fastest to 6 slowest) and Import mapping (ordinary control range 0 … 6)
pub const SCROLL_RANGE: (i32, i32) = (0, 6);

/// The display scales offered, percent.
// nocite: our own client setting (display scale steps)
pub const SCALES: [u16; 5] = [100, 125, 150, 200, 300];

/// The longest relay address kept, in characters.
// nocite: our own client setting (matches the multiplayer screen's field)
const RELAY_CHARS: usize = 200;

/// The settings.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    /// Sound-effects volume, 0 … 100 percent.
    pub sound: u8,
    /// Announcement volume, 0 … 100 percent.
    pub voices: u8,
    /// Music volume, 0 … 100 percent (scales the music, S-0703; our own setting).
    pub music: u8,
    /// Scroll-speed index, [`SCROLL_RANGE`].
    pub scroll_index: i32,
    /// Interface size, percent of view pixels, one of [`SCALES`] (lowered
    /// where the view is too small for it: `glasnost_ui::geom::Scale::fit`).
    /// It enlarges the sidebar and the menus only; the display scale
    /// enlarges the whole picture.
    pub ui_scale: u16,
    /// Display scale: screen pixels per view pixel for the whole game
    /// picture, one of `game::display::CHOICES` (0 Auto, from the
    /// display's scale factor; 1, 2 or 3 fixed).
    pub display_scale: u8,
    /// Whether announcements are shown as captions.
    pub captions: bool,
    /// Whether house colours use our colour-blind palette (from the next
    /// game on).
    pub colour_blind: bool,
    /// The player's tooltip option: whether name tips show over the
    /// battlefield (S-0111 claims 20 and 26).
    pub tool_tips: bool,
    /// The command keys.
    pub bindings: Bindings,
    /// The relay address last used on the multiplayer screen ("" none).
    pub relay: String,
    /// The network game this client can rejoin, while one runs (M9 N5).
    pub rejoin: Option<Rejoin>,
    /// Whether network games try direct links between the peers (M9 N7);
    /// off: the relay carries everything.
    pub direct: bool,
    /// The direct links' STUN or TURN servers, space-separated URLs (M9
    /// N7); none by default.
    pub ice: String,
    /// Touchpad scrolling (our own addition, `game::touch`): a touchpad's
    /// pixel swipes pan the view or step the icon strip. Off, a wheel event
    /// of any kind steps the strip one row, as before. A touch screen's
    /// drags and taps work either way.
    pub touchpad_scroll: bool,
}

/// A network game a client can rejoin after its link or its process went
/// (M9 N5): the relay, the room, the slot and the slot's ticket. Not a
/// choice of the person's, but kept beside the relay address so that a
/// restarted client can offer **Reconnect**; the ticket only opens that
/// slot of that room's running game.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rejoin {
    /// The relay's address.
    pub relay: String,
    /// The room code.
    pub code: String,
    /// The slot.
    pub slot: u8,
    /// The slot's ticket.
    pub ticket: [u8; 16],
}

impl Rejoin {
    /// The kept text: code, slot, ticket in hex and relay, by spaces.
    fn to_text(&self) -> String {
        let hex: String = self.ticket.iter().map(|b| format!("{b:02x}")).collect();
        let relay: String = self.relay.chars().filter(|c| !c.is_control()).collect();
        format!("{} {} {hex} {relay}", self.code, self.slot)
    }

    /// From kept text; `None` if any part is wrong.
    fn from_text(v: &str) -> Option<Self> {
        let mut it = v.trim().splitn(4, ' ');
        let code = it.next()?.to_owned();
        let slot = it.next()?.parse().ok()?;
        let hex = it.next()?;
        let relay: String = it.next()?.chars().take(RELAY_CHARS).collect();
        if code.is_empty() || !code.chars().all(|c| c.is_ascii_alphanumeric()) || hex.len() != 32 {
            return None;
        }
        let mut ticket = [0u8; 16];
        for (i, b) in ticket.iter_mut().enumerate() {
            *b = u8::from_str_radix(hex.get(2 * i..2 * i + 2)?, 16).ok()?;
        }
        Some(Self {
            relay,
            code,
            slot,
            ticket,
        })
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            // spec: S-0701 F3 (the player's sound volume option starts at full volume)
            sound: 100,
            // spec: S-0701 claim 35 (the voice option is at full)
            voices: 100,
            // nocite: our own client setting (music volume default)
            music: 100,
            // spec: S-0101 Import mapping (a missing saved scroll setting gives 3)
            scroll_index: 3,
            // nocite: our own client setting (no scaling)
            ui_scale: 100,
            // design: readable text and display scale (lead's request 2026-11-10): Auto by default
            display_scale: crate::game::display::AUTO,
            // nocite: our own client setting (captions shown by default)
            captions: true,
            // nocite: our own client setting (the colour-blind palette is off by default)
            colour_blind: false,
            // spec: S-0111 Import mapping (settings file [Options] ToolTips: the tooltip option, yes when the settings give none) and claim 20 (on unless the player's settings turn it off)
            tool_tips: true,
            bindings: Bindings::default(),
            relay: String::new(),
            rejoin: None,
            // nocite: our own netcode (direct links on, the relay keeps every frame)
            direct: true,
            // nocite: our own netcode (no STUN or TURN server: nothing contacted by default)
            ice: String::new(),
            // design: touch and touchpad scrolling (lead's request 2026-11-10): on by default
            touchpad_scroll: true,
        }
    }
}

impl Settings {
    /// The mixer option for a volume percentage, 0 … 16384.
    // nocite: our own client setting (percent to the mixer's range, S-0701 F3's o is 0 … 16384)
    pub fn mixer_level(percent: u8) -> i32 {
        i32::from(percent.min(100)) * 16384 / 100
    }

    /// The text kept for these settings.
    pub fn to_text(&self) -> String {
        let mut s = format!("version={VERSION}\n");
        s += &format!(
            "sound={}\nvoices={}\nmusic={}\n",
            self.sound, self.voices, self.music
        );
        s += &format!("scroll={}\nscale={}\n", self.scroll_index, self.ui_scale);
        // design: readable text and display scale (lead's request 2026-11-10): `display=auto`, `1`, `2` or `3`; a file without it reads as Auto
        s += &format!("display={}\n", self.display_label());
        s += &format!(
            "captions={}\ncolour_blind={}\n",
            u8::from(self.captions),
            u8::from(self.colour_blind)
        );
        s += &format!("tool_tips={}\n", u8::from(self.tool_tips));
        for a in ACTIONS {
            s += &format!("key.{}={}\n", a.name(), self.bindings.key(a));
        }
        let relay: String = self.relay.chars().filter(|c| !c.is_control()).collect();
        s += &format!("relay={relay}\n");
        if let Some(r) = &self.rejoin {
            s += &format!("rejoin={}\n", r.to_text());
        }
        if !self.direct {
            s += "direct=0\n";
        }
        let ice: String = self.ice.chars().filter(|c| !c.is_control()).collect();
        if !ice.is_empty() {
            s += &format!("ice={ice}\n");
        }
        if !self.touchpad_scroll {
            s += "touchpad=0\n";
        }
        s
    }

    /// Settings from kept text: a field that is missing or wrong takes its
    /// default; keys that clash fall back to the default bindings together.
    pub fn from_text(text: &str) -> Self {
        let mut s = Self::default();
        let mut keys = Bindings::default();
        let mut taken: Vec<(Action, char)> = Vec::new();
        for line in text.lines() {
            let Some((k, v)) = line.split_once('=') else {
                continue;
            };
            let num = || v.trim().parse::<i64>().ok();
            let flag = || match v.trim() {
                "1" => Some(true),
                "0" => Some(false),
                _ => None,
            };
            match k.trim() {
                "sound" => s.sound = num().map_or(s.sound, |n| n.clamp(0, 100) as u8),
                "voices" => s.voices = num().map_or(s.voices, |n| n.clamp(0, 100) as u8),
                "music" => s.music = num().map_or(s.music, |n| n.clamp(0, 100) as u8),
                "scroll" => {
                    s.scroll_index = num().map_or(s.scroll_index, |n| {
                        n.clamp(i64::from(SCROLL_RANGE.0), i64::from(SCROLL_RANGE.1)) as i32
                    });
                }
                "scale" => {
                    if let Some(n) = num().filter(|n| SCALES.iter().any(|x| i64::from(*x) == *n)) {
                        s.ui_scale = n as u16;
                    }
                }
                "display" => {
                    let d = match v.trim() {
                        "auto" => Some(crate::game::display::AUTO),
                        n => n
                            .parse::<u8>()
                            .ok()
                            .filter(|n| *n != 0 && crate::game::display::CHOICES.contains(n)),
                    };
                    s.display_scale = d.unwrap_or(s.display_scale);
                }
                "captions" => s.captions = flag().unwrap_or(s.captions),
                "colour_blind" => s.colour_blind = flag().unwrap_or(s.colour_blind),
                "tool_tips" => s.tool_tips = flag().unwrap_or(s.tool_tips),
                "relay" => {
                    s.relay = v
                        .chars()
                        .filter(|c| !c.is_control())
                        .take(RELAY_CHARS)
                        .collect();
                }
                "rejoin" => s.rejoin = Rejoin::from_text(v),
                "direct" => s.direct = flag().unwrap_or(s.direct),
                // design: touch and touchpad scrolling (lead's request 2026-11-10): `touch` is the setting's first name, read as `touchpad`
                "touchpad" | "touch" => s.touchpad_scroll = flag().unwrap_or(s.touchpad_scroll),
                "ice" => {
                    s.ice = v
                        .chars()
                        .filter(|c| !c.is_control())
                        .take(RELAY_CHARS)
                        .collect::<String>()
                        .trim()
                        .to_owned();
                }
                other => {
                    if let Some(name) = other.strip_prefix("key.")
                        && let Some(a) = ACTIONS.iter().copied().find(|a| a.name() == name)
                        && let Some(c) = v.trim().chars().next()
                    {
                        taken.push((a, c.to_ascii_lowercase()));
                    }
                }
            }
        }
        for &(a, c) in &taken {
            keys.0[a.index()] = c;
        }
        // A type-select or pathing-report key the file leaves out (a file
        // written before M8 A4) keeps its default unless the file gives
        // that key to another action; it then takes the first letter or
        // digit free. The command keys of S-0213 table T2 are left as they
        // were. The make-group key (M8 A4 round 5 seams) and the route-mode
        // key (a file written before intake 50, which had none) do the same.
        for a in [
            Action::RouteMode,
            Action::TypeSelect,
            Action::PathReport,
            Action::Mark,
            Action::MakeGroup,
        ] {
            if taken.iter().any(|(t, _)| *t == a) {
                continue;
            }
            let clashes =
                |k: char, keys: &Bindings| ACTIONS.iter().any(|b| *b != a && keys.key(*b) == k);
            if clashes(keys.key(a), &keys)
                && let Some(free) = ('a'..='z').chain('0'..='9').find(|k| !clashes(*k, &keys))
            {
                keys.0[a.index()] = free;
            }
        }
        // A key that is not a letter or digit, or that two actions share,
        // spoils the whole set; the defaults stand then.
        let distinct = ACTIONS.iter().all(|a| keys.key(*a).is_ascii_alphanumeric())
            && ACTIONS.iter().enumerate().all(|(i, a)| {
                ACTIONS[i + 1..]
                    .iter()
                    .all(|b| keys.key(*a) != keys.key(*b))
            });
        if distinct {
            s.bindings = keys;
        }
        s
    }

    /// The scroll rows' text: 1 (slowest) … 7 (fastest).
    pub fn scroll_label(&self) -> String {
        format!("{} of 7", 7 - self.scroll_index)
    }

    /// Steps the scroll speed: `d` > 0 faster.
    pub fn step_scroll(&mut self, d: i32) {
        self.scroll_index = (self.scroll_index - d).clamp(SCROLL_RANGE.0, SCROLL_RANGE.1);
    }

    /// The display scale as kept: `auto`, `1`, `2` or `3`.
    pub fn display_label(&self) -> String {
        if self.display_scale == crate::game::display::AUTO {
            "auto".into()
        } else {
            self.display_scale.to_string()
        }
    }

    /// Steps the display scale through `game::display::CHOICES` (Auto,
    /// 1x, 2x, 3x), stopping at either end.
    pub fn step_display(&mut self, d: i32) {
        let c = crate::game::display::CHOICES;
        let at = c.iter().position(|x| *x == self.display_scale).unwrap_or(0) as i32;
        self.display_scale = c[(at + d).clamp(0, c.len() as i32 - 1) as usize];
    }

    /// Steps the interface size to the next or previous offered one.
    pub fn step_scale(&mut self, d: i32) {
        let at = SCALES.iter().position(|x| *x == self.ui_scale).unwrap_or(0) as i32;
        self.ui_scale = SCALES[(at + d).clamp(0, SCALES.len() as i32 - 1) as usize];
    }
}

/// Steps a volume by ten percent.
// nocite: our own client setting (volume step)
pub fn step_volume(v: u8, d: i32) -> u8 {
    (i32::from(v) + 10 * d).clamp(0, 100) as u8
}

/// Where the settings are kept.
pub trait Store {
    /// The kept text, if any can be read.
    fn read(&self) -> Option<String>;
    /// Keeps the text.
    fn write(&self, text: &str) -> Result<(), String>;
}

/// A string key-value storage that may refuse (the browser's `localStorage`).
pub trait KeyValue {
    /// The value of a key.
    fn get(&self, key: &str) -> Result<Option<String>, String>;
    /// Sets a key.
    fn set(&self, key: &str, value: &str) -> Result<(), String>;
}

/// The settings in a [`KeyValue`] storage. A storage that refuses reads as
/// empty (defaults) and reports its refusals on writes.
pub struct KvStore<K: KeyValue>(pub K);

impl<K: KeyValue> Store for KvStore<K> {
    fn read(&self) -> Option<String> {
        self.0.get(WEB_KEY).ok().flatten()
    }

    fn write(&self, text: &str) -> Result<(), String> {
        self.0.set(WEB_KEY, text)
    }
}

/// The settings file in a folder.
pub struct FileStore(pub PathBuf);

impl Store for FileStore {
    fn read(&self) -> Option<String> {
        std::fs::read_to_string(&self.0).ok()
    }

    fn write(&self, text: &str) -> Result<(), String> {
        if let Some(dir) = self.0.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        // Written beside the file and renamed over it, so a crash leaves
        // the old file whole.
        let tmp = self.0.with_extension("tmp");
        std::fs::write(&tmp, text).map_err(|e| e.to_string())?;
        std::fs::rename(&tmp, &self.0).map_err(|e| e.to_string())
    }
}

/// The host systems whose configuration folders differ.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Host {
    /// Linux and the other Unix systems.
    Unix,
    /// macOS.
    Mac,
    /// Windows.
    Windows,
}

/// The settings file's path under the platform's configuration folder, from
/// an environment lookup. `None` when no absolute folder can be named.
// nocite: our own client setting (the platform's conventional configuration folders)
pub fn config_path(host: Host, env: impl Fn(&str) -> Option<String>) -> Option<PathBuf> {
    let abs = |v: Option<String>| v.map(PathBuf::from).filter(|p| p.is_absolute());
    let base = match host {
        Host::Windows => abs(env("APPDATA")),
        Host::Mac => abs(env("HOME")).map(|h| h.join("Library").join("Application Support")),
        Host::Unix => {
            abs(env("XDG_CONFIG_HOME")).or_else(|| abs(env("HOME")).map(|h| h.join(".config")))
        }
    }?;
    Some(base.join("glasnost").join("settings.txt"))
}

/// The store of this platform: the settings file natively, `localStorage`
/// on the web; `None` when there is nowhere to keep them.
pub fn platform_store() -> Option<Box<dyn Store>> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let host = if cfg!(windows) {
            Host::Windows
        } else if cfg!(target_os = "macos") {
            Host::Mac
        } else {
            Host::Unix
        };
        let path = config_path(host, |k| std::env::var(k).ok())?;
        Some(Box::new(FileStore(path)))
    }
    #[cfg(target_arch = "wasm32")]
    {
        web::local_storage().map(|s| Box::new(KvStore(s)) as Box<dyn Store>)
    }
}

#[cfg(target_arch = "wasm32")]
mod web {
    use super::KeyValue;

    /// The page's `localStorage`.
    pub struct Local(web_sys::Storage);

    /// `localStorage`, if the browser gives it (it may throw when blocked).
    pub fn local_storage() -> Option<Local> {
        web_sys::window()?.local_storage().ok().flatten().map(Local)
    }

    impl KeyValue for Local {
        fn get(&self, key: &str) -> Result<Option<String>, String> {
            self.0.get_item(key).map_err(|e| format!("{e:?}"))
        }

        fn set(&self, key: &str, value: &str) -> Result<(), String> {
            self.0.set_item(key, value).map_err(|e| format!("{e:?}"))
        }
    }
}

#[cfg(test)]
#[path = "settings_tests.rs"]
mod tests;
