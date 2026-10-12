from PIL import Image, ImageDraw
from pathlib import Path
p=Path('ferrite-ricochet/assets')
im=Image.new('RGBA',(256,256),(11,11,12,255));d=ImageDraw.Draw(im)
d.rectangle((16,16,239,239),outline=(61,61,66),width=8)
for y in range(24,232,4):
 for x in range(24,232,4):
  if (x//4+y//4)%4==0:d.rectangle((x,y,x+1,y+1),fill=(42,42,46))
d.line([(48,188),(48,64),(72,40),(184,40),(208,64),(208,188)],fill=(142,142,146),width=8)
for x,y in [(88,96),(164,88),(128,136)]:
 d.ellipse((x-17,y-17,x+17,y+17),fill=(24,24,27),outline=(232,232,230),width=5)
d.line([(68,190),(111,172)],fill=(242,169,59),width=12)
d.line([(188,190),(145,172)],fill=(242,169,59),width=12)
d.rectangle((116,56,130,70),fill=(242,169,59))
im.save(p/'logo.png');im.save(p/'logo.ico',sizes=[(16,16),(32,32),(48,48),(64,64),(128,128),(256,256)])
svg='<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256"><defs><pattern id="d" width="8" height="8" patternUnits="userSpaceOnUse"><path d="M0 0h2v2H0zM4 4h2v2H4z" fill="#2a2a2e"/></pattern></defs><path fill="#0b0b0c" d="M0 0h256v256H0z"/><path fill="url(#d)" stroke="#3d3d42" stroke-width="8" d="M20 20h216v216H20z"/><path d="M48 188V64l24-24h112l24 24v124" fill="none" stroke="#8e8e92" stroke-width="8"/><g fill="#18181b" stroke="#e8e8e6" stroke-width="5"><circle cx="88" cy="96" r="17"/><circle cx="164" cy="88" r="17"/><circle cx="128" cy="136" r="17"/></g><path d="m68 190 43-18m77 18-43-18" stroke="#f2a93b" stroke-width="12"/><path fill="#f2a93b" d="M116 56h14v14h-14z"/></svg>'
(p/'logo.svg').write_text(svg)
