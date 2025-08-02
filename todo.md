### General Engine

[ ] Proper errors when pushing new assets, etc. ("Result" instead of "Option)

[ ] Drawing
    [x] Tilemap to Tilemap
    [x] Patch to Tilemap
    [x] Anim to Sprite Layer
    [ ] Tilemap to Sprite Layer (basic sprite, no anim. Just a wrapper)
    [ ] Animations to Tilemaps
        . Wait until Anim pipeline is more stable (i.e. when I can generate Anim structs from the tileset itself)
        . Will be useful to create BG interactions (i.e. door opening)

### Backend and examples

[.] Use LIRQ (Line interrupt) to draw Game GUI
    . Will need to switch Tile bank halfway through
    . Will also need external BG Maps

[.] Debug views! Display all loaded tiles, colors and subpalettes
    [x] Display all banks
    [x] Display palette colors
    [x] Display subpalettes.
    [x] Use tato_layout, positioning everything manually is a pain!
    [x] Mouse over display debug
    [ ] Mouse over video output:
        [ ] Inspect any BG tile being displayed by tato_video.
        [ ] Inspect Sprites.
    [x] Shrink tile view to used tiles Only
    [x] Shrink subpalettes size
    [.] Indicate colors added Vs. default colors
    [ ] Indicate bank usage (as FG, BG bank or unused)
    [ ] Display FPS, average pixel iteration time (will need a simple SmoothBuffer).

[ ] Dual chip setup for multiple video layers

### Pipeline:

[x] Load individual named tiles into a tileset, each will be exported as a const
    . On the other end, insert single tiles into TileBank (returns TileID)

[.] Invalid tiles (such as when color count is higher than allowed) should pinpoint tile coordinates where error occurred.

[ ] "Build Kit", temporary tiles to generate procedural tilemaps
    . This is essential to iterate fast on map designs.

### Assets

[x] Text and Fonts
    [/] Fonts will be Cell-based assets, like Anim and Tilemaps
        . Using Tilemaps as fonts seems to work better, allows easy detection of flipped tiles, etc.
    [x] Write directly to the BG Map
    [ ] Let the function accept a user defined slice of characters so that fonts of any length may be used. I.e. Very basic games may only need numbers.

[.] Load & Unload Tilesets.
    . May do just a "pop" for now (won't be able to unload a tileset "in the middle", only the topmost one)
    [.] Arena approach!
        [.] Basic push/pop implemented, needs testing!
        [x] Once tilesets + tilemaps are working, implement Anims!
        . Since animations use tilemaps, I just need a way to load multiple tilemaps from the "frames" array, and some draw_anim mechanism to retrieve the TilemapRef from the Arena, already with the correct offset.
        . Maybe "load_animation_frames", which result in an AnimEntry with the frames data (start, count, frame_length)
        [ ] Detect and prevent loading "empty" animation frames
        [ ] Think about auto-loading assets? "load_tilemap" seems simple enough to allow this.

[x] Tilemaps
    [x] Correctly map subpalettes when loading into Assets.
        . Looks done? Needs more testing

[x] Anims: Update to latest Assets struct
    [x] Frames should just be Tilemaps?
    [x] Create Anims out of a "frame array"
--->[ ] Add and use "tato.time" to draw animations, instead of video.frame_count(), to ensure frame rate independence.

[x] Fonts: Replace text rendering using Anim to use Fonts.
    . Update: Fonts are just tilemaps, to allow flags

[x] Finish converting Anim data to array of tilemaps in tato_pipe
    [x] "draw_patch" will then take a map as a parameter, which will bring in tile flags.

[?] Treat Palettes and SubPalettes as assets
    . Will allow easier importing from png assets, loading/unloading, etc.
    [x] Remove palette head style counters from tato_video, move all management to Tato.

[x] Smarter sub-palettes in Pipeline?
    . The problem is that, depending on order of tile processing, too many unnecessary palettes are generated
    . Option 1: pre-process the palette using the whole image, instead of per tile?
    . Option 2: Try Option<u8> when building color hashes, and if when inserting a color one slot is None, it is still available and we don't need a new tile hash, we can modify the existing one instead? More complex, Let's try option 1 instead first...
    [x] Almost there! Only remaining issue is palette swapped tiles can get flagged as separate tiles. Instead of the actual color index, the Hashmap needs to store a "difference map" that compares a pixel to its rightmost neighbor (wraps around). 1 is "different", 0 is "same".
    [x] BUG: Transparent color is not coming through in tiles in the "tilemap" example. Indices seem to come in as "1" instead of "zero"? Also subpalette sorting doesn't seem right, could be related.
