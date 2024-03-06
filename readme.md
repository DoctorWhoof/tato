# PLEASE DO NOT USE! IT IS NOT IN A RELIABLE STATE YET!

Ridiculous old fashioned game engine. You probably shouldn't use this.
For very simple games that would feel uncomfortable in a large, bloated, modern engine.

Features:
- Minimal dependencies (slotmap, libm and num-traits). No standard library required.
- Old school, tile based, software renderer with up to (but no more than) 256 colors.
- Basic AABB collisions with reactions.
- Minimal runtime. This thing can't even read a PNG! Or any file! Check "tato_pipe" for a way to convert PNG graphics into binary data at build time.
