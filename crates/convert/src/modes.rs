//! The mode list: the multiplayer and skirmish game modes, each with its
//! kind and game-mode file, and the mode a skirmish plays (S-0006 claim 13).

use glasnost_ini::Document;
use glasnost_ini::value::decimal_integer;

/// The mode list file (S-0006 Import mapping; looked up without regard to
/// letter case, S-0005).
pub const MODE_LIST: &str = "mpmodesmd.ini"; // spec: S-0006 Import mapping (the mode list)

/// The six mode kinds, one per section of the mode list, in reading order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ModeKind {
    /// `[Battle]`.
    Battle,
    /// `[ManBattle]`.
    ManBattle,
    /// `[Siege]`.
    Siege,
    /// `[Unholy]`.
    Unholy,
    /// `[FreeForAll]`.
    FreeForAll,
    /// `[Cooperative]`.
    Cooperative,
}

impl ModeKind {
    /// Every kind, in the reading order of the mode list's sections.
    // spec: S-0006 Import mapping (mode-list sections, one per mode kind, in reading order)
    pub const ALL: [ModeKind; 6] = [
        ModeKind::Battle,
        ModeKind::ManBattle,
        ModeKind::Siege,
        ModeKind::Unholy,
        ModeKind::FreeForAll,
        ModeKind::Cooperative,
    ];

    /// The section that lists modes of this kind (matched exactly, letter
    /// case included: S-0006 claim 13, S-0001 claim 13).
    pub fn section(self) -> &'static str {
        // spec: S-0006 Import mapping (mode-list section names)
        match self {
            ModeKind::Battle => "Battle",
            ModeKind::ManBattle => "ManBattle",
            ModeKind::Siege => "Siege",
            ModeKind::Unholy => "Unholy",
            ModeKind::FreeForAll => "FreeForAll",
            ModeKind::Cooperative => "Cooperative",
        }
    }
}

/// One listed mode (S-0006 claim 13).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mode {
    /// The mode number: the entry's key in the decimal integer form.
    pub number: i32,
    /// The kind its section gives.
    pub kind: ModeKind,
    /// Item 1: the string-table label of its name.
    pub name_label: Vec<u8>,
    /// Item 2: the string-table label of its tooltip.
    pub tooltip_label: Vec<u8>,
    /// Item 3: the game-mode file's name, as written once its spaces are
    /// removed.
    pub file: Vec<u8>,
    /// Item 4: the map filter.
    pub map_filter: Vec<u8>,
    /// Item 5: off only for `false`, ignoring case.
    pub flag: bool,
    /// Line of the entry in the mode list.
    pub line: u32,
}

impl Mode {
    /// The game-mode file's name as text (bytes 128 to 255 kept as Latin-1
    /// code points, so the name resolves as written).
    pub fn file_name(&self) -> String {
        self.file.iter().map(|&b| char::from(b)).collect()
    }
}

/// How the mode of a skirmish is chosen (S-0006 claim 13).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ModeChoice {
    /// The mode the skirmish screen preselects when the player's settings
    /// remember no number.
    #[default]
    Preselected,
    /// The preselection for a remembered number *r*.
    Remembered(i32),
    /// The mode at this place (0-based) in the lobby's list, chosen by the
    /// player.
    Listed(usize),
}

/// The space byte, the only byte removed around an item (S-0006 claim 13).
const SPACE: u8 = b' '; // spec: S-0006 claim 13 (only the space byte is removed around an item)

/// Items 1 to 4 are needed; item 5 is optional; later items are ignored.
const NEEDED_ITEMS: usize = 4; // spec: S-0006 claim 13 (items 1 to 4 are needed)
/// The flag is the fifth item.
const FLAG_ITEM: usize = 4; // spec: S-0006 claim 13 (item 5: the flag)

fn trim_spaces(mut s: &[u8]) -> &[u8] {
    while let [SPACE, rest @ ..] = s {
        s = rest;
    }
    while let [rest @ .., SPACE] = s {
        s = rest;
    }
    s
}

/// Reads the mode list (S-0006 claim 13): the listed modes in lobby order,
/// and the importer's bad-data warnings as (line, message).
pub fn read(doc: &Document) -> (Vec<Mode>, Vec<(u32, String)>) {
    let mut modes = Vec::new();
    let mut warnings = Vec::new();
    for kind in ModeKind::ALL {
        let Some(section) = doc.section(kind.section().as_bytes()) else {
            continue;
        };
        // spec: S-0006 claim 13 (every entry is a candidate, every repeat of a key included, with the value S-0002 gives that key)
        for (entry, winner) in section.list_positions() {
            let key = String::from_utf8_lossy(&entry.key);
            // spec: S-0006 claim 13 (the mode number: the key in the decimal form of S-0003 claim 3, no `$` or `h` form)
            let number = decimal_integer(&entry.key);
            let v = &winner.value;
            // spec: S-0006 claim 13 (split at commas, empty items skipped)
            let raw: Vec<&[u8]> = v.split(|&b| b == b',').collect();
            if raw.iter().any(|i| i.is_empty()) {
                warnings.push((
                    entry.line,
                    format!("[{}] {key}: an empty item moves every later item one place earlier (S-0006 claim 13)", kind.section()),
                ));
            }
            let raw: Vec<&[u8]> = raw.into_iter().filter(|i| !i.is_empty()).collect();
            if raw.len() > FLAG_ITEM + 1 {
                warnings.push((
                    entry.line,
                    format!(
                        "[{}] {key}: items after the fifth are ignored (S-0006 claim 13)",
                        kind.section()
                    ),
                ));
            }
            // spec: S-0006 claim 13 (each item has its leading and trailing spaces removed, only the space byte)
            let items: Vec<&[u8]> = raw
                .iter()
                .take(FLAG_ITEM + 1)
                .map(|i| trim_spaces(i))
                .collect();
            if items.iter().any(|i| {
                matches!(i.first(), Some(b) if b.is_ascii_whitespace())
                    || matches!(i.last(), Some(b) if b.is_ascii_whitespace())
            }) {
                warnings.push((
                    entry.line,
                    format!("[{}] {key}: an item begins or ends with whitespace other than the space byte, which stays part of the item (S-0006 claim 13)", kind.section()),
                ));
            }
            // spec: S-0006 claim 13 (items 3 and 4 must not be empty once their spaces are removed)
            if items.len() < NEEDED_ITEMS || items[2].is_empty() || items[3].is_empty() {
                warnings.push((
                    entry.line,
                    format!("[{}] {key}: the entry lacks a needed item, so the mode is not listed (S-0006 claim 13)", kind.section()),
                ));
                continue;
            }
            let flag = match items.get(FLAG_ITEM) {
                // spec: S-0006 claim 13 (the flag is off only for `false`, ignoring case)
                Some(f) => {
                    if !f.eq_ignore_ascii_case(b"false") && !f.eq_ignore_ascii_case(b"true") {
                        warnings.push((
                            entry.line,
                            format!("[{}] {key}: the fifth item is neither true nor false; the flag is on (S-0006 claim 13)", kind.section()),
                        ));
                    }
                    !f.eq_ignore_ascii_case(b"false")
                }
                None => true,
            };
            modes.push(Mode {
                number,
                kind,
                name_label: items[0].to_vec(),
                tooltip_label: items[1].to_vec(),
                file: items[2].to_vec(),
                map_filter: items[3].to_vec(),
                flag,
                line: entry.line,
            });
        }
    }
    // spec: S-0006 claim 13 (ascending mode number; equal numbers keep their reading order)
    modes.sort_by_key(|m| m.number);
    for w in modes.windows(2) {
        if w[0].number == w[1].number {
            warnings.push((
                w[1].line,
                format!(
                    "mode number {} is shared; the number refers to the first listed mode with it (S-0006 claim 13)",
                    w[1].number
                ),
            ));
        }
    }
    let numbered = modes
        .iter()
        .enumerate()
        .all(|(i, m)| i64::from(m.number) == i as i64 + 1);
    if !modes.is_empty() && !numbered {
        warnings.push((
            0,
            format!(
                "the mode numbers are not exactly 1 to {}, which the remembered-mode check assumes (S-0006 claim 13)",
                modes.len()
            ),
        ));
    }
    (modes, warnings)
}

/// The first listed mode with `number`, if any.
fn first_with(modes: &[Mode], number: i32) -> Option<usize> {
    modes.iter().position(|m| m.number == number)
}

/// The listed mode a skirmish plays (S-0006 claim 13), as a place in
/// `modes`; `None` when no mode is listed. `first_wins` is
/// `quirks.mode_number_first_wins`.
pub fn select(modes: &[Mode], choice: ModeChoice, first_wins: bool) -> Option<usize> {
    if modes.is_empty() {
        return None;
    }
    let count = modes.len();
    let preselect = |r: Option<i32>| -> usize {
        // spec: S-0006 claim 13 (the remembered-mode table: the bound is the count of listed modes; a gap or an out-of-range number gives the first listed mode)
        match r {
            Some(r) if r >= 1 && i64::from(r) <= count as i64 => first_with(modes, r).unwrap_or(0),
            _ => 0,
        }
    };
    Some(match choice {
        ModeChoice::Preselected => preselect(None),
        ModeChoice::Remembered(r) => preselect(Some(r)),
        ModeChoice::Listed(i) => {
            let i = i.min(count - 1);
            if first_wins {
                // spec: S-0006 claim 13 (a game identifies its mode by its number: the first listed mode with it)
                first_with(modes, modes[i].number).unwrap_or(i)
            } else {
                // spec: S-0006 claim 13 (switch off: the mode's place in the list)
                i
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use glasnost_ini::Quirks;

    use super::*;

    fn modes(text: &str) -> (Vec<Mode>, Vec<(u32, String)>) {
        read(&Document::parse(
            text.as_bytes(),
            Quirks::default(),
            &mut Vec::new(),
        ))
    }

    fn files(m: &[Mode]) -> Vec<String> {
        m.iter().map(Mode::file_name).collect()
    }

    /// spec: S-0006 claim 13 (sections, items, needed items, flag, order; scenario Q)
    #[test]
    fn items_and_order() {
        let (m, w) = modes(
            "[Siege]\n2=N2,T2,s.ini,F\n[Battle]\n1=N1,T1, b.ini ,F1,FALSE\n3=N3,,T3,c.ini,F3\n\
             4=N4,T4,,F4\n5=N5,T5,d.ini\n6=N6,   ,e.ini,F6,maybe,extra\n[battle]\n7=N,T,x.ini,F\n\
             [Other]\n8=N,T,y.ini,F\n",
        );
        // Kind order Battle before Siege, then sorted by number.
        assert_eq!(files(&m), ["b.ini", "s.ini", "c.ini", "e.ini"]);
        assert!(!m[0].flag);
        assert_eq!(m[0].kind, ModeKind::Battle);
        assert_eq!(m[1].kind, ModeKind::Siege);
        // An empty item moves the later items one place earlier.
        assert_eq!(m[2].tooltip_label, b"T3");
        assert_eq!(m[2].map_filter, b"F3");
        // A label made only of spaces counts as present; a fifth item other than false leaves the flag on.
        assert_eq!(m[3].tooltip_label, b"");
        assert!(m[3].flag);
        let says = |t: &str| w.iter().any(|(_, s)| s.contains(t));
        assert!(says("empty item"));
        assert!(says("lacks a needed item"));
        assert!(says("neither true nor false"));
        assert!(says("after the fifth"));
        assert!(says("not exactly 1 to"));
    }

    /// spec: S-0006 claim 13 (the mode number's decimal form, scenario Q12)
    #[test]
    fn mode_numbers() {
        let (m, _) = modes("[Battle]\n$5=N,T,a.ini,F\n5h=N,T,b.ini,F\n -3=N,T,c.ini,F\n");
        let n: Vec<i32> = m.iter().map(|m| m.number).collect();
        assert_eq!(n, [-3, 0, 5]);
        assert_eq!(files(&m), ["c.ini", "a.ini", "b.ini"]);
    }

    /// spec: S-0006 claim 13 (a repeated key gives two identical candidates)
    #[test]
    fn repeated_key() {
        let (m, w) = modes("[Battle]\n1=N,T,a.ini,F\n1=N,T,b.ini,F\n");
        assert_eq!(m.len(), 2);
        assert_eq!(m[0].file, m[1].file);
        assert!(w.iter().any(|(_, s)| s.contains("shared")));
    }

    /// spec: S-0006 claim 13 (the remembered-mode table and `quirks.mode_number_first_wins`)
    #[test]
    fn selection() {
        let (m, _) = modes(
            "[Battle]\n1=N,T,a.ini,F\n2=N,T,b.ini,F\n2=N,T2,b.ini,F\n[Siege]\n2=N,T,c.ini,F\n9=N,T,d.ini,F\n",
        );
        assert_eq!(files(&m), ["a.ini", "b.ini", "b.ini", "c.ini", "d.ini"]);
        let pick = |c, f| select(&m, c, f);
        assert_eq!(pick(ModeChoice::Preselected, true), Some(0));
        assert_eq!(pick(ModeChoice::Remembered(0), true), Some(0));
        assert_eq!(pick(ModeChoice::Remembered(-5), true), Some(0));
        assert_eq!(pick(ModeChoice::Remembered(2), true), Some(1));
        // 9 is above the count of listed modes (5): the first listed mode.
        assert_eq!(pick(ModeChoice::Remembered(9), true), Some(0));
        // 4: no listed mode has it.
        assert_eq!(pick(ModeChoice::Remembered(4), true), Some(0));
        // Choosing the Siege mode numbered 2 plays the first mode numbered 2.
        assert_eq!(pick(ModeChoice::Listed(3), true), Some(1));
        assert_eq!(pick(ModeChoice::Listed(3), false), Some(3));
        assert_eq!(pick(ModeChoice::Listed(4), true), Some(4));
        assert_eq!(select(&[], ModeChoice::Preselected, true), None);
    }
}
