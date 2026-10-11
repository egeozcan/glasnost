//! Type kinds, their lists and the rank order of their reading passes
//! (S-0004).

/// A type kind (glossary, *type kind*) other than sides, which follow
/// their own rules (S-0004 claim 9).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Kind {
    /// Countries.
    Countries,
    /// Superweapons.
    Superweapons,
    /// Animations (sections from the art database).
    Animations,
    /// Buildings.
    Buildings,
    /// Aircraft.
    Aircraft,
    /// Vehicles.
    Vehicles,
    /// Infantry.
    Infantry,
    /// Weapons (by reference only).
    Weapons,
    /// Projectiles (by reference only).
    Projectiles,
    /// Warheads.
    Warheads,
    /// Terrain objects.
    TerrainObjects,
    /// Smudges.
    Smudges,
    /// Overlays.
    Overlays,
    /// Particles.
    Particles,
    /// Particle systems.
    ParticleSystems,
    /// Voxel animations.
    VoxelAnimations,
    /// Tiberiums (matched by key number, S-0004 claim 10).
    Tiberiums,
}

impl Kind {
    /// Kinds in the rank order of the reading passes (S-0004 claim 6).
    /// Tiberiums have no reading pass; they read at the end of a layer.
    // spec: S-0004 claim 6 (ranks 1 to 16)
    pub const PASSES: [Kind; 16] = [
        Kind::Countries,
        Kind::Superweapons,
        Kind::Animations,
        Kind::Buildings,
        Kind::Aircraft,
        Kind::Vehicles,
        Kind::Infantry,
        Kind::Weapons,
        Kind::Projectiles,
        Kind::Warheads,
        Kind::TerrainObjects,
        Kind::Smudges,
        Kind::Overlays,
        Kind::Particles,
        Kind::ParticleSystems,
        Kind::VoxelAnimations,
    ];

    /// Every kind, tiberiums last.
    pub fn all() -> impl Iterator<Item = Kind> {
        Self::PASSES.into_iter().chain([Kind::Tiberiums])
    }

    /// Rank of the kind's reading pass (S-0004 claim 6); tiberiums after
    /// every pass.
    pub fn rank(self) -> usize {
        Self::PASSES
            .iter()
            .position(|k| *k == self)
            .map_or(Self::PASSES.len() + 1, |i| i + 1)
    }

    /// The kind's type list section (S-0004 Import mapping), if it has one.
    pub fn list_section(self) -> Option<&'static [u8]> {
        // spec: S-0004 Import mapping (type list sections)
        Some(match self {
            Kind::Countries => b"Countries",
            Kind::Superweapons => b"SuperWeaponTypes",
            Kind::Animations => b"Animations",
            Kind::Buildings => b"BuildingTypes",
            Kind::Aircraft => b"AircraftTypes",
            Kind::Vehicles => b"VehicleTypes",
            Kind::Infantry => b"InfantryTypes",
            Kind::Warheads => b"Warheads",
            Kind::TerrainObjects => b"TerrainTypes",
            Kind::Smudges => b"SmudgeTypes",
            Kind::Overlays => b"OverlayTypes",
            Kind::Particles => b"Particles",
            Kind::ParticleSystems => b"ParticleSystems",
            Kind::VoxelAnimations => b"VoxelAnims",
            Kind::Tiberiums => b"Tiberiums",
            // spec: S-0004 claim 3 (weapons and projectiles have no type list)
            Kind::Weapons | Kind::Projectiles => return None,
        })
    }

    /// Our name for the kind in the rules format.
    pub fn key(self) -> &'static str {
        match self {
            Kind::Countries => "countries",
            Kind::Superweapons => "superweapons",
            Kind::Animations => "animations",
            Kind::Buildings => "buildings",
            Kind::Aircraft => "aircraft",
            Kind::Vehicles => "vehicles",
            Kind::Infantry => "infantry",
            Kind::Weapons => "weapons",
            Kind::Projectiles => "projectiles",
            Kind::Warheads => "warheads",
            Kind::TerrainObjects => "terrain_objects",
            Kind::Smudges => "smudges",
            Kind::Overlays => "overlays",
            Kind::Particles => "particles",
            Kind::ParticleSystems => "particle_systems",
            Kind::VoxelAnimations => "voxel_animations",
            Kind::Tiberiums => "tiberiums",
        }
    }

    /// Whether the importer implements every way a type of this kind can
    /// be created, so that its registration indices are complete (S-0004
    /// claims 3, 4 and 13). The layering implements the type lists and
    /// every creating reference field, and the game-start particle system.
    /// Two creations after all rules layers are not implemented: terrain
    /// objects that a map's placements name, and animations that the
    /// theater control file's tile animations name (S-0004 claim 3).
    pub fn registry_complete(self) -> bool {
        // spec: S-0004 claim 3 (creations after the rules layers: map terrain-object placements, theater tile animations)
        !matches!(self, Kind::TerrainObjects | Kind::Animations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// spec: S-0004 claim 6
    #[test]
    fn ranks() {
        assert_eq!(Kind::Countries.rank(), 1);
        assert_eq!(Kind::Animations.rank(), 3);
        assert_eq!(Kind::Weapons.rank(), 8);
        assert_eq!(Kind::VoxelAnimations.rank(), 16);
        assert!(Kind::Tiberiums.rank() > 16);
        assert_eq!(Kind::all().count(), 17);
    }
}
