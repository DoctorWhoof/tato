
[.] Text and Fonts
    . Write directly to the BG Map
    . Let the function accept a user defined slice of characters so that simple and complex fonts may be used freely.

[ ] Dual chip setup for multiple video layers

[.] Use LIRQ (Line interrupt) to draw GUI
    . Will need to switch Tile bank halfway through
    . Will also need external BG Maps

[ ] Load & Unload Tilesets
    . Wait until external tiles and Tile Maps are more stable

### Pipeline:

[x] Load individual named tiles into a tileset, each will be exported as a const
    . On the other end, insert single tiles into TileBank (returns TileID)

### Tilemap example

[.] Finish converting Anim data to array of tilemaps in tato_pipe
    [ ] "draw_patch" will then take an anim as a parameter, which will bring in tile flags.
    [ ] Smarter sub-palettes in Pipeline?
        . The problem is that, depending on order of tile processing, too many unnecessary palettes are generated
        . Option 1: pre-process the palette using the whole image, instead of per tile?
        . Option 2: Try Option<u8> when building color hashes, and if when inserting a color one slot is None, it is still available and we don't need a new tile hash, we can modify the existing one instead? More complex, Let's try option 1 instead first...
