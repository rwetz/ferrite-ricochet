# FERROUS / ROOT ACCESS, first review

Created through Blender MCP from the actual FERROUS scene mesh and default game
state. Review `ferrous.blend` or `preview.png`. The Blender file retains the
original default scene and adds `FERROUS_ROOT_ACCESS_V1`.

- `FERROUS_Functional_Reference`: existing cabinet, rails, routes, mechanisms.
  Stock etched text and tall rear-shelf sculpture are omitted in the preview.
- `FERROUS_Decoration_Export`: new CRT, server racks, side electronics, circuitry,
  ASCII text and ordered dither. 3,878 triangles, palette roles only.
- `ferrous.json`: validated decoration export, in game world coordinates.

The source builder is `scripts/blender/create_ferrous.py`; it requires a dump of
the native scene as documented in its header. No textures or paid generators.

This version is staged for visual feedback, not installed in the game. Importing
it unchanged over the stock scene would retain competing labels and rear toys;
the scene needs matching replacement logic when integrating the approved design.
Collision and gameplay are unchanged. GPU performance and live app appearance
have not been verified for these assets.
