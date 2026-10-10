---
id: Q-0009
title: Theater data file, tile numbering, tile files and tile placement (planned S-0010)
status: answered
related: [S-0009, S-0101, S-0102, S-0104]
answered-by: [S-0010]
---

## Need
The map viewer (M2) draws terrain from each cell's tile number and sub-tile
(S-0009 claims 7 and 8). No approved spec says which file holds a tile's
picture or where that picture goes on the screen. Planned S-0010 owns
these. Code gap: `glasnost_convert::theater` (tile catalogue) and
`glasnost_render::map` (tile drawing).

## Questions
1. Which file holds a theater's data (the file whose `[General]` and
   `[TileSetNNNN]` sections S-0009 claim 6 and its Import mapping read), for
   each of the six theaters? Is it found through the S-0005 search order
   like other data files?
2. Which tileset sections exist: is the numbering `TileSet0000`,
   `TileSet0001`, … read until the first missing number, or some other rule?
   Does a tileset without a `TilesInSet` key count as one tile (the
   glossary says tiles are numbered consecutively across tilesets)?
3. Which file holds tile *m*: the tileset's file-name key, the tile's
   1-based position in its set written with two digits, and the theater's
   extension? What happens when that file does not resolve (the tile number
   still counts, but what is drawn)?
4. Tile variants: when files with a letter suffix exist next to a tile's
   file, which one does a cell show? If it is chosen at random, from which
   stream and in which order (S-0009 claim 20's draw order)?
5. Which cell of a tile file's template does sub-tile *n* name (row-major
   index over the template's cells, empty cells included?), and what is
   drawn for a sub-tile that names an empty or missing template cell?
6. Placement: where is a sub-tile's diamond drawn relative to its cell's
   top corner (S-0101 F1), and where is its extra image (the part above
   the diamond) drawn relative to the diamond? Is the per-tile height field
   stored in the tile file used for drawing at all, or only the cell's
   height level (S-0101 claim 4)?
7. What exactly is drawn for the "theater's clear tile" of S-0009 claim 8
   (tile 0 of the clear tileset, sub-tile 0?), and for a cell whose tile
   is valid but whose sub-tile is outside its template?
8. Land type of a cell (needed by S-0107 claim 2 for *tread* on water):
   where does it come from?
9. The spelling of the shadow-caster tileset key (S-0104 Open questions).

## Possible observation
A purpose-made map using one tile of a set with variant files on a row
of cells; screenshots show which variant each cell takes, and whether it
changes between launches.

## Current workaround
The importer reads the theater data file `C`‖`MD.INI` with `C` the
theater's control name (S-0005 Table 2), falling back to `C`‖`.INI`;
it numbers tiles over `TileSet0000`… until the first missing section
(`TilesInSet` absent counts 1) and names tile files `FileName`‖`NN`‖`.E`.
This is file-loader knowledge taken from OpenRA's `LegacyTilesetImporter`
(GPL-3), used only so that terrain can be shown; it is marked provisional
in code and in the viewer. Variants are never chosen (the unsuffixed file
is drawn), sub-tile *n* is the *n*-th template cell in row-major order,
and the diamond is drawn with its top at the cell's top corner, lifted 15
pixels per height level, the extra image placed by the file's own
template coordinates. Land types are not read: no cell counts as water.
