# Blender board pipeline

Ricochet draws palette-colored polygon faces, without texture or glTF support.
This pipeline adds static decorative meshes to the existing cabinet. Physics,
gates, routes, flippers, bumpers and scoring remain authored in Rust.

## Authoring

- Create a `RicochetExport` collection containing only meshes to export.
- Blender coordinates: X right, negative Y toward player, Z up. One Blender
  unit equals one game world unit. No additional `place()` scaling is applied.
  Field dimensions are 329.4 by 448.4. Origin is the back-left playfield corner.
- Set each object's custom `ricochet_role` string to `artwork` or `scenery`.
- Set integer custom property `ricochet_ink` to a palette index: 0 sunken,
  1 background, 2 surface, 3 raised, 4 line, 5 strong line, 6 faint foreground,
  7 dim foreground, 8 foreground, 9 dim accent, 10 accent, 11 warning, 12 danger.
- Artwork is flat at game Z 0.05..0.4. Raised scenery belongs behind the back
  boundary or outside side rails, with a four-unit safety margin. The importer
  checks whole triangles, including triangles that bridge the playable area.
- Keep the board under 4,000 triangles. Modifiers and world transforms are baked
  on evaluated temporary meshes. Textures, material colors, animations and
  collection instances are unsupported; convert instances to real meshes first.

## Export from Blender or MCP

Execute this in Blender's Python console, or through `execute_blender_code`:

```python
import runpy
pipeline = runpy.run_path(r'C:\Users\ryan\Dev\ferrite-ricochet\scripts\blender\export_board.py')
pipeline['export_collection']('ferrite', r'C:\Users\ryan\Dev\ferrite-ricochet\assets\boards\ferrite.json')
```

Board keys: ferrite, mono, graphite, slate, concrete, harbor, cyanotype,
phosphor, verdigris, bruise. Export refuses to overwrite existing files. Export
to a new staging filename for revisions, then replace the intended source after
review. Keep only one JSON per board in the import directory.

## Import

```powershell
python scripts/import_board_assets.py --check
python scripts/import_board_assets.py
rustfmt --edition 2024 src/board_assets.rs
```

The importer validates every source before writing the generated Rust module
and SHA-256 receipt. Commit the source JSON, receipt and generated module together.
The renderer appends imported faces once when building the static board mesh;
missing boards use existing art. There are no runtime file paths or new crates.
An empty source directory builds an empty catalog.

Run `python scripts/test_board_assets.py` for the validation regression suite.
Before shipping real assets, inspect every affected board in the running app
for occlusion and performance. Static validation cannot prove visual quality.
Physical geometry changes need the existing trajectory and clearance replays.
