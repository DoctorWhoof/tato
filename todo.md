
### Backend and examples

[.] Use LIRQ (Line interrupt) to draw GUI
    . Will need to switch Tile bank halfway through
    . Will also need external BG Maps

[.] Debug views! Display all loaded tiles, colors and subpalettes
    [x] Display all banks
    [x] Display palette colors
    [x] Display subpalettes.
    [x] Use tato_layout, positioning everything manually is a pain!
    [x] Mouse over display debug
    [ ] Mouse over video output:
------->[ ] Inspect any BG tile being displayed by tato_video.
        [ ] Inspect Sprites.

[ ] Dual chip setup for multiple video layers

### Pipeline:

[x] Load individual named tiles into a tileset, each will be exported as a const
    . On the other end, insert single tiles into TileBank (returns TileID)

### Assets

[.] Text and Fonts
    [/] Fonts will be Cell-based assets, like Anim and Tilemaps
        . May not be needed at all! Using Tilemaps as fonts seems to work fine, and allows easy detection of flipped tiles, etc.
    [x] Write directly to the BG Map
    [?] Let the function accept a user defined slice of characters so that simple and complex fonts may be used freely.

--->[ ] Load & Unload Tilesets

[ ] Tilemaps
    [?] Correctly map subpalettes when loading into Assets.
        . Looks done? Needs more testing

[ ] Anims: Update to latest Assets struct

[ ] Fonts: Replace text rendering using Anim to use Fonts. Update: maybe tilemaps, to allow flags?

[.] Finish converting Anim data to array of tilemaps in tato_pipe
    [x] "draw_patch" will then take a map as a parameter, which will bring in tile flags.

[?] Treat Palettes and SubPalettes as assets
    . Will allow easier importing from png assets, loading/unloading, etc.
    [x] Remove palette head style counters from tato_video, move all management to Tato.

[ ] Smarter sub-palettes in Pipeline?
    . The problem is that, depending on order of tile processing, too many unnecessary palettes are generated
    . Option 1: pre-process the palette using the whole image, instead of per tile?
    . Option 2: Try Option<u8> when building color hashes, and if when inserting a color one slot is None, it is still available and we don't need a new tile hash, we can modify the existing one instead? More complex, Let's try option 1 instead first...
    [ ] Almost there! Only remaining issue is palette swapped tiles can get flagged as separate tiles. Instead of the actual color index, the Hashmap needs to store a "difference map" that compares a pixel to its rightmost neighbor (wraps around). 1 is "different", 0 is "same".
    [x] BUG: Transparent color is not coming through in tiles in the "tilemap" example. Indices seem to come in as "1" instead of "zero"? Also subpalette sorting doesn't seem right, could be related.
