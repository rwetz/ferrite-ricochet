"""Run in Blender: export_collection('ferrite', '/absolute/path/ferrite.json')."""
import json
from pathlib import Path


def export_collection(board, destination, collection_name="RicochetExport"):
    import bpy

    collection = bpy.data.collections.get(collection_name)
    if collection is None:
        raise ValueError(f"Create a collection named {collection_name}")
    depsgraph = bpy.context.evaluated_depsgraph_get()
    objects = []
    for obj in sorted(collection.all_objects, key=lambda o: o.name):
        if obj.type != "MESH":
            continue
        role = obj.get("ricochet_role")
        ink = obj.get("ricochet_ink")
        if role not in ("artwork", "scenery") or type(ink) is not int or not 0 <= ink <= 12:
            raise ValueError(f"{obj.name}: set ricochet_role and ricochet_ink (0..12)")
        evaluated = obj.evaluated_get(depsgraph)
        mesh = evaluated.to_mesh()
        try:
            mesh.calc_loop_triangles()
            # Blender X right, Y toward back, Z up; game Y runs toward player.
            vertices = [[v.x, -v.y, v.z] for v in
                        (evaluated.matrix_world @ v.co for v in mesh.vertices)]
            # Reflection reverses winding.
            triangles = [list(reversed(t.vertices)) for t in mesh.loop_triangles]
            objects.append(dict(name=obj.name, role=role, ink=ink,
                                vertices=vertices, triangles=triangles))
        finally:
            evaluated.to_mesh_clear()
    payload = dict(version=1, board=board, coordinates="ricochet-world", objects=objects)
    destination = Path(destination)
    if destination.exists():
        raise FileExistsError(f"Refusing to overwrite {destination}")
    destination.parent.mkdir(parents=True, exist_ok=True)
    with destination.open("x", encoding="utf-8") as stream:
        json.dump(payload, stream, allow_nan=False, indent=2)
    return str(destination)
