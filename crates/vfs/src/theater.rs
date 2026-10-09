//! The six theaters and the names the engine derives from each.
//!
//! These are engine constants of the original, not stock data: S-0005 Table 2
//! (archive names), S-0007 Table 1 (sprite extension and substitution
//! letter), S-0103 Constants (palette names) and S-0009 Import mapping (the
//! map's theater value).

/// A theater.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Theater {
    /// Temperate (index 0).
    Temperate,
    /// Snow (index 1).
    Snow,
    /// Urban (index 2).
    Urban,
    /// Desert (index 3).
    Desert,
    /// New urban (index 4).
    NewUrban,
    /// Lunar (index 5).
    Lunar,
}

impl Theater {
    /// Every theater in index order (S-0009 Import mapping: theaters 0 to 5).
    pub const ALL: [Theater; 6] = [
        Self::Temperate,
        Self::Snow,
        Self::Urban,
        Self::Desert,
        Self::NewUrban,
        Self::Lunar,
    ];

    /// The map's theater value (S-0009 Import mapping, S-0103 Constants:
    /// identifier).
    pub fn identifier(self) -> &'static str {
        // spec: S-0009 Import mapping ([Map] Theater), S-0103 Constants
        match self {
            Self::Temperate => "TEMPERATE",
            Self::Snow => "SNOW",
            Self::Urban => "URBAN",
            Self::Desert => "DESERT",
            Self::NewUrban => "NEWURBAN",
            Self::Lunar => "LUNAR",
        }
    }

    /// Reads a map's theater value, ignoring ASCII case (S-0009 Import
    /// mapping). `None` for an unknown name (our engine then uses temperate
    /// and the importer warns, S-0009 Edge cases).
    pub fn from_identifier(value: &[u8]) -> Option<Self> {
        // spec: S-0009 Import mapping ([Map] Theater, case-insensitive)
        Self::ALL
            .into_iter()
            .find(|t| value.eq_ignore_ascii_case(t.identifier().as_bytes()))
    }

    /// Control name C (S-0005 Table 2), also the palette base name *B* of
    /// S-0103.
    pub fn control_name(self) -> &'static str {
        // spec: S-0005 Table 2, S-0103 Constants (B)
        match self {
            Self::Temperate => "TEMPERAT",
            Self::Snow => "SNOW",
            Self::Urban => "URBAN",
            Self::Desert => "DESERT",
            Self::NewUrban => "URBANN",
            Self::Lunar => "LUNAR",
        }
    }

    /// Extension E (S-0005 Table 2, S-0007 Table 1), also the palette suffix
    /// *X* of S-0103.
    pub fn extension(self) -> &'static str {
        // spec: S-0005 Table 2, S-0007 Table 1, S-0103 Constants (X)
        match self {
            Self::Temperate => "TEM",
            Self::Snow => "SNO",
            Self::Urban => "URB",
            Self::Desert => "DES",
            Self::NewUrban => "UBN",
            Self::Lunar => "LUN",
        }
    }

    /// Palette name P (S-0005 Table 2).
    pub fn palette_name(self) -> &'static str {
        // spec: S-0005 Table 2
        match self {
            Self::Temperate => "ISOTEM",
            Self::Snow => "ISOSNO",
            Self::Urban => "ISOURB",
            Self::Desert => "ISODES",
            Self::NewUrban => "ISOUBN",
            Self::Lunar => "ISOLUN",
        }
    }

    /// Tile-art name A (S-0005 Table 2).
    pub fn tile_art_name(self) -> &'static str {
        // spec: S-0005 Table 2
        match self {
            Self::Temperate => "ISOTEMP",
            Self::Snow => "ISOSNOW",
            Self::Urban => "ISOURB",
            Self::Desert => "ISODES",
            Self::NewUrban => "ISOUBN",
            Self::Lunar => "ISOLUN",
        }
    }

    /// Substitution letter L (S-0007 Table 1).
    pub fn letter(self) -> u8 {
        // spec: S-0007 Table 1
        match self {
            Self::Temperate => b'T',
            Self::Snow => b'A',
            Self::Urban => b'U',
            Self::Desert => b'D',
            Self::NewUrban => b'N',
            Self::Lunar => b'L',
        }
    }

    /// The theater group's archives in rank order (S-0005 claim 14).
    pub fn archive_group(self) -> Vec<String> {
        let mut out = Vec::new();
        // spec: S-0005 claim 14 (snowmd.mix for the snow theater only)
        if self == Self::Snow {
            out.push(format!("{}md.mix", self.control_name()));
        }
        // spec: S-0005 claim 14 (C.mix, E.mix, P md.mix, A.mix)
        out.push(format!("{}.mix", self.control_name()));
        out.push(format!("{}.mix", self.extension()));
        out.push(format!("{}md.mix", self.palette_name()));
        out.push(format!("{}.mix", self.tile_art_name()));
        out
    }

    /// The unit palette's file name (S-0103 claim 2).
    pub fn unit_palette(self) -> String {
        // spec: S-0103 claim 2 (UNIT X .PAL)
        format!("UNIT{}.PAL", self.extension())
    }

    /// The tile palette's file name (S-0103 claim 2).
    pub fn tile_palette(self) -> String {
        // spec: S-0103 claim 2 (ISO X .PAL)
        format!("ISO{}.PAL", self.extension())
    }

    /// The theater palette's file name (S-0103 claim 2).
    pub fn theater_palette(self) -> String {
        // spec: S-0103 claim 2 (B .PAL)
        format!("{}.PAL", self.control_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// spec: S-0005 claim 14, S-0103 claim 2 (temperate examples), S-0009
    #[test]
    fn names() {
        assert_eq!(
            Theater::Snow.archive_group(),
            [
                "SNOWmd.mix",
                "SNOW.mix",
                "SNO.mix",
                "ISOSNOmd.mix",
                "ISOSNOW.mix"
            ]
        );
        assert_eq!(Theater::Temperate.archive_group().len(), 4);
        assert_eq!(Theater::Temperate.unit_palette(), "UNITTEM.PAL");
        assert_eq!(Theater::Temperate.tile_palette(), "ISOTEM.PAL");
        assert_eq!(Theater::Temperate.theater_palette(), "TEMPERAT.PAL");
        assert_eq!(
            Theater::from_identifier(b"newUrban"),
            Some(Theater::NewUrban)
        );
        assert_eq!(Theater::from_identifier(b"MARS"), None);
    }
}
