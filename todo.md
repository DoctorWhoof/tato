### Priority List

- [ ] Better strategy to update the Dashboard bank texture, to prevent the stale data bug I just ran into...

- [ ] Pipeline: Ensure names for assets are always unique, to avoid accidentally overwriting silently.

### General Engine

- [.] Implement command-agnostic console
    - For common debug actions like "warp x,y", "reset", "toggle x", etc.
    - Provides text input and parses the text line into command + args
    - Does not actually process commands - that will be on the Game side
    - Maybe returns an Option<CommandLine> struct with an u8 array + indices for each argument?
    - [x] Needs to ignore input when not visible, and capture input when visible
    - [x] Ignore punctuation (treat the same as spaces), or at least commas
    - [x] Maybe instead of merely getting the command line, a way to actually process it, which involves getting the command line, processing it and returning a result message tht is displayed in the console? (look at herobot "Game::process_console" for a use case)
    - [ ] Move console to its own crate, store history in internal arena (since it's persistent).
    - [ ] Up and down arrows should navigate command history.

- [x] Engine pausing
    - Internal timer will freeze, but input can still be updated in the main loop.
    - Game logic (update and draw) must be skipped in the main loop if tato.is_paused(), not in the engine.

- [x] Proper error handling/messages when arena handles are invalid.
    - Most "Option" types need to become "Result" with proper error messages
    - It's super hard to debug arena related errors right now due to that
    - maybe "expect" instead of unwrap, so i get error messages?

- [?] New PixelIter that favors iteration speed over memory compactness (i.e. frame buffer) - There's a bug "somewhere" (I blame the O.S...) where if I don't print anything, the game uses more CPU and takes longer to finish. Maybe it's using efficiency cores instead of performance ones? - In any case, with printing (sigh) iter times are now around 0.5ms in release, solidly in the "Good Enough" zone. I still want to redesign the iterator like this, but it's not a priority

- [x] Groups must be tileset-independent!
      . GroupBuilder is separate from each TilesetBuilder, saves to its own module.
      . Gets passed to tilesetbuilder::new, just like palette?

- [x] Collider should be a TileFlag bit, not a group
      . Will allow 255 groups (u8::MAX), instead of 8 (1 bit per group)
      . Tile can only be in one group (water, door, powerup, etc), but can have multiple flags like "collider" or "trigger"
      . Will be ready if i decide to implement sprite collisions
      . Doesn't feel right: flags like "trigger" don't belong in the video chip, since they're gameplay related. Maybe just a "collider" bit, plus 3 "custom" bits that the user can choose how to use.
    - [ ] Once tiles use the "collider" flag, figure out a way for the pipeline to mark it.

- [x] Proper errors when pushing new assets, etc. ("Result" instead of "Option)

- [ ] Drawing
    - [x] BgOp should just take col, row and cell
    - [x] Tilemap to Tilemap
    - [x] Patch to Tilemap
    - [x] Anim to Sprite Layer
    - [ ] Tilemap to Sprite Layer (basic sprite, no anim. Just a wrapper)
    - [ ] Animations to Tilemaps
          . Wait until Anim pipeline is more stable (i.e. when I can generate Anim structs from the tileset itself)
          . Will be useful to create BG interactions (i.e. door opening)

- [x] Color behavior is really confusing (there are default colors, but you can push new ones to overwrite them).
      . I'm considering breaking away from default colors and requiring colors to be always defined by the user
      . There could be a "default_colors()" function that pushes the default ones, if needed

- [/] Eliminate subpalettes... replace with 4 colors per tile.
  . Simplify pipeline, reduce errors due to subpalette limit reached
  . 4 colors per tile stays. Maybe a "Mode 1" where we can have 16 colors per tile?...
  . Clusters can stay as is? (2 bits per pixel)
  . To draw a cluster you'll need a cell anyway, which means you'll know which color each index is.
  . Should still allow palette swap, will need a palette override mechanism
  . Subpalette bits can now be groups, up to 15 + None
  . Instead of using bits, simply use named groups like WALL and DOOR
  . This will keep Cell at 4 bytes: - Id: 1 byte - Flags: 1 byte - Palette: 2 bytes (4 bits per color)
  . UPDATE: Will simply use more subpalettes instead. Allows for more groups while still keeping Cell size to 4 bytes.
  . May just adopt 16 color clusters (i.e. Master System) in the future, with some way to override/remap colors.

### Arenas

    - [.] Arena-backed strings
        . Implements Into<&str>
        . Some sort of string formatting would be great

    - [/] Rename Buffer to something nicer!

### Backend and examples

- [ ] Pure Rust backend to make compiling and testing in Linux less miserable. Macroquad?

- [x] Eliminate DashArgs, replace with Backend functions, since now the Dashboard has direct access to Backend

- [x] Some way to easily send information to the backend.
      . Some static mut shenanigans? Should be OK since it won't affect gameplay. Investigate.
- [x] Debug rects with colors
- [x] Debug text

- [.] Use LIRQ (Line interrupt) to draw Game GUI
  . Will need to switch Tile bank halfway through
  . Will also need external BG Maps

- [.] Debug views! Display all loaded tiles, colors and subpalettes
    - [x] Display all banks
    - [x] Display palette colors
    - [x] Display subpalettes.
    - [x] Use tato_layout, positioning everything manually is a pain!
    - [x] Mouse over display debug
    - [ ] Mouse over video output:
    - [ ] Inspect any BG tile being displayed by tato_video.
    - [ ] Inspect Sprites.
    - [x] Shrink tile view to used tiles Only
    - [x] Shrink subpalettes size
    - [.] Indicate colors added Vs. default colors
    - [ ] Indicate bank usage (as FG, BG bank or unused)
    - [ ] Display FPS, average pixel iteration time (will need a simple AvgBuffer).

- [ ] Dual chip setup for multiple video layers

### Pipeline:

- [ ] Bank::new_from always generates a 256 tile bank in memory, even if the tile count is smaller. I need to generate a tile array instead, and append(tiles, colors, mappings)?
    - Sounds too complicated, using a Bank neatly wraps all that
    - Maybe a BankRef struct that marely refers to colors, tiles and mappings? That could be handy when a size-erased Bank becomes necessary?
    - Skipping for now since
        - a: It works already
        - b: I want to avoid adding more types.

- [x] Modify import paths so that all paths are relative to the "global" import path, and don't require the full path to be called on every method that takes in import paths.

- [x] Bank.append should get the offset, apply to tilemap indices. Somehow.

- [x] Init anim should still exist, so that we can create animations from strip frames - Maybe as part of the build phase, instead of runtime init?

- [x] Load individual named tiles into a tileset, each will be exported as a const
      . On the other end, insert single tiles into TileBank (returns TileID)

- [?] Invalid tiles (such as when color count is higher than allowed) should pinpoint tile coordinates where error occurred.

- [ ] Group detection needs testing in new 4bpp Pipeline

### Assets

- [x] Text and Fonts
    - [/] Fonts will be Cell-based assets, like Anim and Tilemaps
      . Using Tilemaps as fonts seems to work better, allows easy detection of flipped tiles, etc.
- [x] Write directly to the BG Map
    - [ ] Let the function accept a user defined slice of characters so that fonts of any length may be used. I.e. Very basic games may only need numbers.

- [.] Load & Unload Tilesets.
  . May do just a "pop" for now (won't be able to unload a tileset "in the middle", only the topmost one)
- [.] Arena approach!
- [.] Basic push/pop implemented, needs testing!
    - [x] Once tilesets + tilemaps are working, implement Anims!
          . Since animations use tilemaps, I just need a way to load multiple tilemaps from the "frames" array, and some draw_anim_to_fg mechanism to retrieve the TilemapRef from the Arena, already with the correct offset.
          . Maybe "load_animation_frames", which result in an AnimEntry with the frames data (start, count, frame_length)
    - [ ] Detect and prevent loading "empty" animation frames
    - [ ] Think about auto-loading assets? "load_tilemap" seems simple enough to allow this.

- [x] Tilemaps
    - [x] Correctly map subpalettes when loading into Assets.
          . Looks done? Needs more testing

- [x] Anims: Update to latest Assets struct
    - [x] Frames should just be Tilemaps?
    - [x] Create Anims out of a "frame array"
    - [x] Add and use "tato.time" to draw animations, instead of video.frame_number(), to ensure frame rate independence.
    - [x] AnimID(0) should mean "no animation"

- [x] Fonts: Replace text rendering using Anim to use Fonts.
      . Update: Fonts are just tilemaps, to allow flags

- [x] Finish converting Anim data to array of tilemaps in tato_pipe
    - [x] "draw_patch" will then take a map as a parameter, which will bring in tile flags.

- [?] Treat Palettes and SubPalettes as assets
  . Will allow easier importing from png assets, loading/unloading, etc. - [x] Remove palette head style counters from tato_video, move all management to Tato.

- [x] Smarter sub-palettes in Pipeline?
      . The problem is that, depending on order of tile processing, too many unnecessary palettes are generated
      . Option 1: pre-process the palette using the whole image, instead of per tile?
      . Option 2: Try Option<u8> when building color hashes, and if when inserting a color one slot is None, it is still available and we don't need a new tile hash, we can modify the existing one instead? More complex, Let's try option 1 instead first...
- [x] Almost there! Only remaining issue is palette swapped tiles can get flagged as separate tiles. Instead of the actual color index, the Hashmap needs to store a "difference map" that compares a pixel to its rightmost neighbor (wraps around). 1 is "different", 0 is "same".
- [x] BUG: Transparent color is not coming through in tiles in the "tilemap" example. Indices seem to come in as "1" instead of "zero"? Also subpalette sorting doesn't seem right, could be related.
