
### Backend and examples

[ ] Dual chip setup for multiple video layers

[.] Use LIRQ (Line interrupt) to draw GUI
    . Will need to switch Tile bank halfway through
    . Will also need external BG Maps

[.] Debug views! Display all loaded tiles, colors and subpalettes
    [x] Display all banks
    [x] Display palette colors
--->[ ] Display subpalettes.
        [ ] New: Use tato_layout, positioning everything is a pain!

### Pipeline:

[x] Load individual named tiles into a tileset, each will be exported as a const
    . On the other end, insert single tiles into TileBank (returns TileID)

### Assets

[.] Text and Fonts
    [/] Fonts will be Cell-based assets, like Anim and Tilemaps
        . May not be needed at all! Using Tilemaps as fonts seems to work fine, and allows easy detection of flipped tiles, etc.
    [x] Write directly to the BG Map
    [?] Let the function accept a user defined slice of characters so that simple and complex fonts may be used freely.

[.] Load & Unload Tilesets
    . Wait until external tiles and Tile Maps are more stable

[ ] Tilemaps
--->[ ] Correctly map subpalettes when loading into Assets

[ ] Anims: Update to latest Assets struct

[ ] Fonts: Replace text rendering using Anim to use Fonts

[.] Finish converting Anim data to array of tilemaps in tato_pipe
    [x] "draw_patch" will then take a map as a parameter, which will bring in tile flags.

[?] Treat Palettes and SubPalettes as assets
    . Will allow easier importing from png assets, loading/unloading, etc.
    [ ] Remove palette head style counters from tato_video, move all management to Tato.

[ ] Smarter sub-palettes in Pipeline?
    . The problem is that, depending on order of tile processing, too many unnecessary palettes are generated
    . Option 1: pre-process the palette using the whole image, instead of per tile?
    . Option 2: Try Option<u8> when building color hashes, and if when inserting a color one slot is None, it is still available and we don't need a new tile hash, we can modify the existing one instead? More complex, Let's try option 1 instead first...
