"""Build FERROUS / ROOT ACCESS in a separate Blender scene via MCP.

Requires the actual Ricochet face dump at reference_path. Does not delete the
user's scene. Palette meshes, including text, are editable and texture-free.
"""
import json
import math
import re
from pathlib import Path
import bpy
from mathutils import Vector

ROOT = Path(r"C:\Users\ryan\Dev\ferrite-ricochet")
COLORS = [(.025,.03,.03,1), (.045,.055,.052,1), (.10,.12,.115,1),
          (.18,.21,.19,1), (.19,.24,.20,1), (.29,.36,.30,1),
          (.36,.43,.36,1), (.58,.66,.56,1), (.82,.87,.75,1),
          (.18,.34,.10,1), (.57,.85,.21,1), (.92,.57,.12,1), (.73,.17,.10,1)]


def build(reference_path, revision=1):
    scene_name=f"FERROUS_ROOT_ACCESS_V{revision}"
    if bpy.data.scenes.get(scene_name):
        raise ValueError(f"{scene_name} already exists; preserve it and make a revision")
    scene = bpy.data.scenes.new(scene_name)
    bpy.context.window.scene = scene
    ref = bpy.data.collections.new("FERROUS_Functional_Reference")
    art = bpy.data.collections.new("FERROUS_Decoration_Export")
    scene.collection.children.link(ref)
    scene.collection.children.link(art)
    materials = []
    created = {}
    for i, color in enumerate(COLORS):
        mat = bpy.data.materials.new(f"FERROUS_ink_{i:02}")
        mat.diffuse_color = color
        mat.use_nodes = True
        shader = next(n for n in mat.node_tree.nodes if n.type == "BSDF_PRINCIPLED")
        shader.inputs['Base Color'].default_value = color
        shader.inputs['Roughness'].default_value = .8
        materials.append(mat)

    def mesh(name, verts, faces, ink, role="scenery", collection=art):
        data = bpy.data.meshes.new(name)
        data.from_pydata([(x,-y,z) for x,y,z in verts], [], [list(reversed(f)) for f in faces])
        data.materials.append(materials[ink])
        data.update()
        obj = bpy.data.objects.new(name, data)
        collection.objects.link(obj)
        obj['ricochet_role'] = role
        obj['ricochet_ink'] = ink
        created[name] = obj
        return obj

    # Retain all physical geometry. Exclude only the old high rear-shelf toy.
    groups = {}
    for face in json.loads(Path(reference_path).read_text(encoding='utf-8-sig')):
        points = face['points']
        if all(p[1] < 22 for p in points) and max(p[2] for p in points) > 8:
            continue
        # Replace stock etched text/dither so new lettering remains legible.
        if all(.11 < p[2] <= .4 for p in points) and max(p[2] for p in points)-min(p[2] for p in points) < .001:
            continue
        verts, faces = groups.setdefault(face['ink'], ([], []))
        offset = len(verts)
        verts.extend(points)
        faces.append(list(range(offset, offset + len(points))))
    for ink, (verts, faces) in groups.items():
        mesh(f"REF_collision_layout_ink_{ink}", verts, faces, ink, collection=ref)

    def box(name, x,y,z,w,d,h,ink):
        v = [(x+dx*w,y+dy*d,z+dz*h) for dz in (0,1) for dy in (0,1) for dx in (0,1)]
        return mesh(name,v,[(0,2,3,1),(4,5,7,6),(0,1,5,4),(2,6,7,3),(0,4,6,2),(1,3,7,5)],ink)

    def rect(name,x,y,w,h,ink,z=.34):
        return mesh(name,[(x,y,z),(x+w,y,z),(x+w,y+h,z),(x,y+h,z)],[(0,1,2,3)],ink,'artwork')

    def screen_rect(name,x,z,w,h,ink,y=19):
        return mesh(name,[(x,y,z),(x+w,y,z),(x+w,y,z+h),(x,y,z+h)],[(0,1,2,3)],ink)

    # Use the game's own 3x5 glyph vocabulary and add terminal punctuation.
    code = (ROOT/'src/scene.rs').read_text(encoding='utf-8')
    glyphs = {c:[int(n) for n in rows.split(',')] for c,rows in re.findall(r"'([^']+)' => \[([0-9, ]+)\]",code)}
    glyphs.update({'>':[4,2,1,2,4], '_':[0,0,0,0,7], '-':[0,0,7,0,0],
                   ':':[0,2,0,2,0], '[':[6,4,4,4,6], ']':[3,1,1,1,3],
                   '#':[5,7,5,7,5], '.':[0,0,0,0,2]})

    def text(name,label,x,y,s,ink=8,vertical=False):
        verts, faces = [], []
        for ci,c in enumerate(label):
            for row,bits in enumerate(glyphs.get(c,[0]*5)):
                # Merge horizontal adjacent pixels to keep the TUI geometry cheap.
                col=0
                while col<3:
                    if not bits & (1<<(2-col)):
                        col+=1
                        continue
                    start=col
                    while col<3 and bits & (1<<(2-col)):
                        col+=1
                    px=x+(ci*4+start)*s
                    width=(col-start-.16)*s
                    if vertical:
                        pz=y-row*s
                        quad=[(px,19.15,pz),(px+width,19.15,pz),(px+width,19.15,pz-.84*s),(px,19.15,pz-.84*s)]
                    else:
                        py=y+row*s
                        quad=[(px,py,.38),(px+width,py,.38),(px+width,py+.84*s,.38),(px,py+.84*s,.38)]
                    offset=len(verts)
                    verts.extend(quad)
                    faces.append(tuple(range(offset,offset+4)))
        if faces:
            return mesh(name,verts,faces,ink,'scenery' if vertical else 'artwork')

    # Terminal shrine at the back, safely outside the playable boundary.
    box('CRT_plinth',100,-2,1,128,18,8,3)
    box('CRT_case',112,0,9,104,18,51,5)
    screen_rect('CRT_black_glass',117,15,94,40,0)
    text('CRT_header','[ ROOT ACCESS ]',122,52,1.3,10,True)
    text('CRT_prompt','> BOOT FERROUS',122,42,1.15,8,True)
    text('CRT_status','SYS: ARMED',122,32,1.25,7,True)
    text('CRT_cursor','> _',122,22,1.25,10,True)
    # Visible screen scanlines and tiny punched tape readout.
    for row in range(13):
        screen_rect(f'CRT_scan_{row}',118,16+row*2.8,92,.22,4,y=19.05)
    for side,x in [('L',62),('R',239)]:
        box(f'{side}_server_stack',x,0,1,23,18,54,3)
        for row in range(5):
            screen_rect(f'{side}_rack_slot_{row}',x+2,8+row*9,19,5,0)
            screen_rect(f'{side}_rack_LED_{row}',x+3,9+row*9,2,2,10 if row%2==0 else 11)
            for i in range(4):
                screen_rect(f'{side}_vent_{row}_{i}',x+8+i*2.5,9+row*9,.7,2.5,6)
        box(f'{side}_antenna',x+9,3,55,3,3,7,7)
    for x in [94,232]:
        box(f'power_bus_{x}',x,3,2,4,12,35,5)
        for z in range(6,34,6):
            box(f'bus_fin_{x}_{z}',x-2,2,z,8,14,1.5,7)

    # Side service channels: capacitors, sockets, solder points and status bars.
    for side,x in [('left',7),('right',312)]:
        service_rows = [58,96,134,172,210,248,286,324,362] if revision == 1 else [58,134,210,286,362]
        for row,y in enumerate(service_rows):
            box(f'{side}_IC_{row}',x,y,11,10,20,3,0)
            for dy in [2,7,12,17]:
                box(f'{side}_pin_{row}_{dy}',x-1,y+dy,11,12,1,1,7)
            box(f'{side}_LED_{row}',x+3,y+3,15,3,3,1,10 if row%3 else 11)
        for y in [84,160,236,312]:
            box(f'{side}_capacitor_{y}',x+2,y,10,6,9,9,5)
            box(f'{side}_cap_mark_{y}',x+3,y+2,19.1,4,1,.5,8)

    # Flat TUI panels. Art stays flush and is distinct from actual mechanisms.
    rect('title_panel',79,239,161,35,0,z=.31)
    text('title','FERROUS',85,243,3.2,8)
    text('subtitle','ROOT ACCESS / 01',85,263,.85,10)
    rect('title_rule',80,237,159,.8,10)
    text('orbit_label','01 / TRACE',43,201,1.05,7)
    text('ramp_label','02 / EXEC',208,281,.92,7)
    text('reactor_label','03 / CORE',116,221,.92,10)
    text('apron_serial','FE-01 / TTY',95,425,1.4,8)
    text('apron_prompt','> INSERT COIN_',101,436,.75,10)
    text('control_Q','[Q]',54,398,1.2,7)
    text('control_E','[E]',237,398,1.2,7)
    text('operator_label','OPERATOR',112,339,.9,5)
    text('checksum_label','0XFE / OK',35,310,.85,6)
    # ASCII registers under the upper circuitry, near the real bumper cores.
    for x,y,label in [(77,153,'CPU'),(179,132,'RAM'),(136,208,'IO')]:
        text(f'core_{label}',label,x,y,.9,7)
    for x,y in [(66,180),(192,184),(71,296),(223,304)]:
        text(f'hash_{x}_{y}','##',x,y,1.8,5)

    # Traces run as angular engraved bus lines, with solder pads at each turn.
    def trace(name,points,ink=9):
        for i,(a,b) in enumerate(zip(points,points[1:])):
            dx,dy=b[0]-a[0],b[1]-a[1]
            length=math.hypot(dx,dy)
            nx,ny=-dy/length*.3,dx/length*.3
            mesh(f'{name}_{i}',[(a[0]+nx,a[1]+ny,.33),(b[0]+nx,b[1]+ny,.33),
                               (b[0]-nx,b[1]-ny,.33),(a[0]-nx,a[1]-ny,.33)],[(0,1,2,3)],ink,'artwork')
        for i,(x,y) in enumerate(points):
            rect(f'{name}_pad_{i}',x-1,y-1,2,2,7,z=.35)
    for i in range(5):
        trace(f'left_bus_{i}',[(41+i*3,87),(41+i*3,126),(65+i*3,150),(65+i*3,178)])
        trace(f'right_bus_{i}',[(262-i*3,80),(262-i*3,169),(234-i*3,197),(234-i*3,234)])
    for i in range(3):
        trace(f'lower_bus_{i}',[(93+i*4,284),(93+i*4,305),(119+i*4,331),(119+i*4,345)],4)
    # Coarse ordered dither patches, intentional clusters rather than noisy fill.
    verts,faces=[],[]
    bayer=[[0,8,2,10],[12,4,14,6],[3,11,1,9],[15,7,13,5]]
    for x0,y0 in [(41,270),(234,310),(93,369),(225,72)]:
        for r in range(9):
            for c in range(8):
                if bayer[r%4][c%4] < 3+r//3:
                    x,y=x0+c*2.2,y0+r*2.2
                    i=len(verts)
                    verts.extend([(x,y,.36),(x+1,y,.36),(x+1,y+1,.36),(x,y+1,.36)])
                    faces.append((i,i+1,i+2,i+3))
    mesh('ordered_dither',verts,faces,6,'artwork')

    if revision >= 2:
        # V2 critique: replace tiny repeated markings with a large authored
        # identity, shot-directed inserts and an actual terminal-panel hierarchy.
        for name in list(created):
            if name.startswith(('hash_', 'core_')) or name in (
                    'orbit_label','ramp_label','reactor_label','operator_label',
                    'checksum_label','control_Q','control_E','apron_serial',
                    'apron_prompt','subtitle','title_panel','title_rule'):
                obj=created.pop(name)
                bpy.data.objects.remove(obj,do_unlink=True)

        rect('identity_panel',82,239,158,35,0,z=.31)
        rect('identity_top_rule',82,238,158,1,10)
        text('identity_subtitle','ROOT ACCESS',85,263,1.35,10)
        # Chip/daemon portrait printed beneath the ball, with quiet dark body
        # and bright eyes. It occupies the previously empty lower playfield.
        portrait=[
            '....XXXXX....',
            '..XXXXXXXXX..',
            '.XXXXXXXXXXX.',
            'XXX.......XXX',
            'XX..GG.GG..XX',
            'XX.........XX',
            '.XX...X...XX.',
            '.XXX.....XXX.',
            '..X.X.X.X.X..',
            '...XXXXXXX...',
            '....X.X.X....',
        ]
        verts,faces=[],[]
        eyes,eye_faces=[],[]
        for row,line in enumerate(portrait):
            for col,c in enumerate(line):
                if c=='.': continue
                target_v,target_f=(eyes,eye_faces) if c=='G' else (verts,faces)
                x,y=126+col*4.7,299+row*4.7
                i=len(target_v)
                target_v.extend([(x,y,.36),(x+4.4,y,.36),(x+4.4,y+4.4,.36),(x,y+4.4,.36)])
                target_f.append((i,i+1,i+2,i+3))
        mesh('daemon_pixel_portrait',verts,faces,5,'artwork')
        mesh('daemon_phosphor_eyes',eyes,eye_faces,10,'artwork')
        text('daemon_bracket_L','[',111,311,3.6,9)
        text('daemon_bracket_R',']',195,311,3.6,9)

        def outline(name,x,y,w,h,ink):
            for i,(a,b,c,d) in enumerate([(x,y,w,.65),(x,y+h,w,.65),(x,y,.65,h),(x+w,y,.65,h)]):
                rect(f'{name}_{i}',a,b,c,d,ink,z=.34)

        # Three objective diamonds use the actual sequence: bumper bank,
        # orbit, reactor. Static artwork only, no invented scoring promises.
        for i,(x,y,ink,label) in enumerate([(110,286,7,'BANK'),(157,286,10,'ORBIT'),(196,286,11,'CORE')]):
            mesh(f'mission_insert_{i}',[(x,y-4,.35),(x+4,y,.35),(x,y+4,.35),(x-4,y,.35)],[(0,1,2,3)],ink,'artwork')
            text(f'mission_text_{i}',label,x-9,y+7,.95,ink)
        # Arrowheads sit in front of the physical gate mouths.
        for name,x,y,ink in [('orbit',51,280,10),('ramp',235,312,11)]:
            mesh(f'{name}_shot_arrow',[(x,y-8,.37),(x+4,y,.37),(x+1.4,y,.37),
                                     (x+1.4,y+6,.37),(x-1.4,y+6,.37),(x-1.4,y,.37),(x-4,y,.37)],
                 [(0,1,2,3,4,5,6)],ink,'artwork')
        text('orbit_callout','TRACE',47,228,1.3,10)
        text('ramp_callout','EXEC',205,325,1.3,11)
        text('scoop_callout','ROOT',166,228,1.35,10)
        # Quiet large upper-center chip stencil, not another lump of tiny text.
        outline('CPU_die',126,92,62,47,4)
        text('CPU_die_mark','FE',141,105,4.2,5)
        for i in range(6):
            rect(f'CPU_left_pin_{i}',121,97+i*6,4,1.4,6)
            rect(f'CPU_right_pin_{i}',189,97+i*6,4,1.4,6)
        # Asymmetric service-panel accents, tape marks and raster distress.
        for x,y,ink in [(8,104,11),(313,180,10),(8,256,10),(313,332,11)]:
            rect(f'service_strip_{x}_{y}',x,y,9,20,0,z=.31)
            for i in range(5):
                rect(f'service_stripe_{x}_{y}_{i}',x,y+i*4,9,1.2,ink,z=.37)
        # Apron is visually a terminal instruction card, at the visible front
        # rather than behind the native raised apron plate.
        rect('operator_card',88,419,158,10,0,z=.31)
        text('operator_card_text','Q/E FLIP / SPACE LAUNCH',94,421,1.05,8)
        text('left_key','[Q]',69,396,1.5,10)
        text('right_key','[E]',224,396,1.5,10)
        text('revision_serial','FE-01',35,407,1.1,6)
        # Dither scales are coarse enough to survive the game camera.
        for row in range(5):
            for col in range(7):
                if (row*3+col*5)%7 < row:
                    rect(f'distress_{row}_{col}',87+col*7,351+row*3,2.4,1.2,4,z=.36)
        scene['revision_reason']='Web critique: stronger central identity, readable shot cues, asymmetry, less repeated micro-hardware.'

    # A matching inspectable camera, aimed from the player's position.
    camera_data=bpy.data.cameras.new('FERROUS_review_camera')
    camera=bpy.data.objects.new('FERROUS_review_camera',camera_data)
    scene.collection.objects.link(camera)
    camera.location=(164.7,-665,630)
    target=Vector((164.7,-218,8))
    camera.rotation_euler=(target-camera.location).to_track_quat('-Z','Y').to_euler()
    camera_data.type='ORTHO'
    camera_data.ortho_scale=565
    scene.camera=camera
    scene.render.engine='BLENDER_WORKBENCH'
    scene.display.shading.light='FLAT'
    scene.display.shading.color_type='MATERIAL'
    scene.display.shading.show_shadows=False
    scene.display.shading.show_cavity=True
    scene.display.shading.cavity_type='BOTH'
    scene.display.shading.background_type='WORLD'
    scene.world=bpy.data.worlds.new('FERROUS_dark_world')
    scene.world.color=(.025,.025,.025)
    scene.render.resolution_x=1000
    scene.render.resolution_y=1400
    scene.render.resolution_percentage=100
    scene.view_settings.view_transform='Standard'
    for area in bpy.context.screen.areas:
        if area.type=='VIEW_3D':
            area.spaces.active.region_3d.view_perspective='CAMERA'
            area.spaces.active.shading.color_type='MATERIAL'
    scene['design_brief']='FERROUS / ROOT ACCESS: ASCII, ordered dither, computer, hacker, TUI, gritty. Equal decorative and functional visual weight.'
    scene['reference']='Actual src/scene.rs + physics::Game::default(); functional reference is not exported.'
    print(f'Created {len(art.objects)} decorative objects and {len(ref.objects)} reference groups')
    return scene,art
