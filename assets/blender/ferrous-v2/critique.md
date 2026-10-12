# FERROUS self-critique and second pass

Reviewed against web references on October 12, 2026. These are visual judgments,
not gameplay or performance measurements. References inform composition only;
no third-party artwork or models were imported.

## Reference lessons

- [TRON: Legacy playfield photos](https://www.pinballnews.com/games/tron/index6a.html)
  show a dense illustrated surface, strong shot-linked inserts, contrasting
  mechanisms and an obvious centerpiece. The actual full playfield photograph
  was inspected in the browser. FERROUS v1 had peripheral clutter without an
  equally strong printed playfield identity.
- [Williams' archived Medieval Madness description](https://www.robertwinter.com/pinball/mm/official/index.htm)
  emphasizes an open playfield and a clear castle objective. This was a textual
  reference: decorative richness does not require filling ball travel space.
- [Gunstein's Pinball3D](https://github.com/gunstein/Pinball3D) demonstrates a
  simple readable theme with one large Christmas-tree silhouette and strongly
  differentiated ball, bumpers and rails. The full project screenshot and the
  author's lower-playfield image were inspected. Its color treatment is too
  crude for Ferrite, but the instantly recognizable theme is a useful standard.

## V1 problems

1. Object count was masquerading as richness. Repeating nine identical chips
   down each edge used triangles without adding much personality.
2. The central field had weak identity. Sparse tiny labels and isolated hash
   marks did not compete with the large bare mechanical loops.
3. Palette hierarchy was too neutral. Green accents existed mostly on the CRT
   and thin traces, with little distinction between shot destinations.
4. Some labels were tiny, hidden by rails or positioned behind the apron lip.
5. The CRT was a useful focal object but insufficient as the entire theme.

## V2 changes

- Added a large pixel daemon portrait with phosphor eyes and terminal brackets.
- Added an oversized FE chip stencil in the upper field.
- Added BANK / ORBIT / CORE diamonds, green orbit and amber ramp arrows, and
  larger TRACE / EXEC / ROOT callouts tied to the real layout.
- Reduced repeating side chips to five per edge and added staggered service
  stripes and coarse raster distress.
- Strengthened the title/subtitle hierarchy and made the apron a legible
  Q/E / Space instruction card.
- A second camera check caught an arrow through the title, rail-obstructed
  callouts and a hidden apron card. Those were corrected before the final save.

Decoration triangles: 3,878 in v1, 3,242 in v2, a reduction of about 16.4%.
Final export passes the cabinet envelope, raised-scenery safety, flat-artwork,
palette, nondegenerate geometry and triangle-budget checks. Five importer
regression tests also pass.

## Remaining limits

V2 reads more clearly as a coherent hacker pinball board. It is still deliberately
restrained compared with a commercial illustrated playfield. The upper CRT and
server cabinets are fairly blocky, and the skull, chip stencil and floor panels
need the user's taste judgment. Tiny labels will remain secondary at a small
game viewport. Decorative inserts are static artwork, not working state lamps.

The original physical routes were retained. This is an editable Blender review
asset and validated decoration export, not a tested in-game replacement. Native
integration still needs matching removal of the stock art/toy and actual
runtime occlusion/performance checks. No claim of physical playability changes.

Files: `ferrous.blend`, `preview.png`, `ferrous.json`, `validation.json.txt`.
Blender contains both v1 and v2 scenes; v2 is active. The original v1 files remain.
