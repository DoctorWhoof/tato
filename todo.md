
[x] Switch data structs into Slotmaps?

[ ] Switch Anim frames to Pool<Frames>, so that only used frames are saved

[ ] Think about generic Scene<ENTITY_COUNT>. How dows that interact with the host app?

[ ] High level "scripting". Example:
    - Present screen, wait for input
    - Pause interaction
    - Load Scene 1
    - Move char A to (x,y) with speed X, anim A
    - Play anim B for 1 second
    - Move char A to (x,y) with speed X, anim C
    - Display overlay with text, wait 2 seconds
    - Load Scene 2
    - Resume interaction
  
[ ] Rendering
    [x] Proper Palettes:
        [x] Palette struct containing array with colors
        [/] GlobalPalette that combines multiple Palettes into a single, global 256 color palette.
            . Abandoned in lieu of RGBA frame buffer for the host app, while the Atlas contains byte indices + palettes.
        [x] Generate palette during build, from RGB .png files
        [x] Bonus: directly support palettized png? Seems to be working already, img is converted to RGBA, then re-palettized in ImageBuilder.

[ ] World
    [ ] Separate rendering and collision into slices.
        . Entity IDs are copied into slices per frame, according to their placement
        . Slices are rendered per thread
        . Collisions can also be calculated per thread?
    [ ] Scene Loading
        . A Scene can point to project assets and "upload" them to the World, replacing:
            - Initial camera position
            - Tile Atlases
            - Animations
            - Entities
        . A "Project" can contain all the assets, i.e. more than the limit per scene.
        . The Project idea can be ditched for now to keep things simple, but it would be more efficient in large projects since it would avoid redundant data.
        [ ] Much, much simpler option: Scenes simply save camera and entities, not assets!
            . I can still come up with more robust .map and .atlas formats
            . "map" can contain the tilemap dimensions, the tile indices and flags
            . "atlas" can contain tile dimensions
            . "anim" can contain the tile flags besides their main data
            . Tilemaps could still exist as a separate file

[.] Tilemaps
    [x] Build a basic tilemap shape with rendering (no autotiling)
    [x] Tilemap importer:
        [/] Investigate exporting JSON out of Aseprite
        [x] If that fails, try converting a whole PNG to tiles with the buid script?
        [x] BUG: Not removing duplicated tiles if they're added separately.
            . Maybe the atlas must be build at compile time?...
            . convert_xxx functions should build the atlas, and a single save at the end commits it to disk?
    [x] Tile "collision", ability to check which tile is underneath a point/rect
        . Game struct Needs to store an ENUM containing current tile, so that we can match later
            . Maybe generate look up table with all ranges, instead of lots of if/else statements?...
        . Maybe generate actual Prop structs from the tiles? Seems more flexible. Props can store individual data, etc.
            . Do this in the init function, do not create an API for it! Keep it simple!
            . A new Entity type, TileEntity, may be needed? It has a range instead of a shape?
            . Update: Only "unique" props need entities, "generic" tiles like walls and stairs can operate as tile indices only.
        . Check needs to be with a Rect, not a point
    [x] Per-Tile data? Think about a good strategy here.
        . Separate layer in aseprite with colors as tile data markers?
        . Build a tilemap editor? No, seriously...
        . Leaning towards color coded layer as a PNG.
        . Settled on pre-importing prop tiles, generating .range files per prop for later use.
    [x] Handle flipped tiles
        . Detect flipped tiles when importing
        . Devise mechanism to use flip data in the tilemap.
        . Maybe generate ".flags" file that go along with ".anim", ".map"?
        [x] Rendering of flipped tiles near the edges is buggy
    [x] Load from binary file (with header, dimensions, etc.)

    [ ] Animated BG tiles (i.e. Door has two frames, open and closed)
        [/] Maybe go back to the idea of "TileEntities", owned by the Tilemap?
            . Use tile flag to check which TileEntity a point is over, instead of using colliders
            . TileEntity can have an anim field, allowing it to change tile indices in its region
        [x] REDESIGN:
            . Inserting "Groups" generates .anim files during build
            . A Group shape can use anim files to write tiles to a tilemap
            . At build time when a tile map is converted, groups are identified and added to a "Scene" file
                [x] Group byte flagging currently not implemented
            . Maybe: a mechanism to also add sprites to the scene, from a tilemap png? Maybe a separate "layer" png, using insert_sprites(file)?
            . Maybe: Before creating the tilemap, create the scene. Tilemaps, groups and sprites can then be added to the scene!
                [.] Atlas files
                    [/] Add group entries, with name and range
                        . Decided to go with .group files instead
                        . But... groups only make sense associated with an Atlas!
                        . For now, will simply save Atlas + Groups in same folder
                        . Will need atlas "swapping" in the future, i.e. An Atlas can load the following tilesets:
                            - Main (Hero, Enemy and UI tiles)
                            - Hotel tiles
                            - Other areas' tiles
                        . The atlas in memory is always composed by loading separate tilesets
                        . Loading tile sets adjusts the loaded anim and group indices to match. In this case, the hotel tiles are offset by the main tiles
                        . This means the anims and groups should be kept in the Atlas, even if loaded from separate files!
                        . Additionally, Tilemaps and Sprites need to point to a specific tileset
                        . The Atlas will then be a collection of tilesets in memory.
                    [x] Insert / Remove tilesets!
                        [x] To keep things simple, it's a stack: you can't remove a set in the middle, you can only "pop".
                        [x] Clean up pipeline (replace AtlasBuilder references with TilesetBuilder, etc.)
                        [?] Since TileIDs have a unique_id, inserting at any index may be doable
        [x] Entities with Tile Group shapes
        [x] BUG: AnimTiles bug out whengoing off screen
        [.] Prototype Door prop with AnimTiles shape.
            [ ] Too many bugs caused by silly mistakes so far! Needs better pipeline design.
                . Groups and Anims should be tied to tilesets, so you can't use a group intended for a tileset on another.
                . Added "GroupEnum" generic parameter to TilesetBuilder
--------------->. Proceed to actually save that data so that initializing a group also restores its enum assignment
            [ ] Detect all door instances at build time, when importing the tilemap.
            [ ] Save/Load basic Scene file containing door entities
            [ ] Add "Group" functionality to anims (look at TilesetBuilder.insert_group)
                [ ] Will need to be saved and loaded with .anim files
                [ ] Also needs "add_anim" closure that updates the "groups" array, just like "add_group"
                    . This right here needs to be streamlined. Maybe groups are just part of the Atlas?
        [ ] Art: Create new "area" (tilemap + distinct tiles) to test moving data in and out of Atlas

[x] Anim files
    [x] Binary with cols/rows
    [x] Flags

[ ] Add additional debug data?
    [ ] Entity Names
    [ ] Debug only scene list (App side)

[.] HUD
    [x] Render a smaller window (instead of rendering entire view, then overwriting the HUD pixels)
    [x] Text tiles rendering
        . Works with minimal set (0 to 9, capital A to Z no punctuation)
    [x] Debug overlay messages passed to the host app
    [ ] BUG: Works at the bottom, doesn't work in any position yet.

[/] Should the colliders be moved out of entities, and Entity renamed "Graphic" or similar?
    . Colliders would live in the Game, and only certain game play structs (like Hero or Enemy) would have one.
    . This would make graphics smaller, and allow collider-less gameplay entities.
    . Update: makes no difference in the Entity size

[x] Think about the tile packing

[x] Automatic tile index remapping
    . When running the "tilify" build function, generate a series of tile numbers representing the frame's original tile mappings

[x] Animated sprite format
    . Will use a series of Frame structs, each containing the tile indices

[x] Add json as a build dependency, then generate .pix and .anim files directly from aseprite exported json file?

[x] Redesign "Entity" as "Sprite"
    . Remove EntityKind, TileKind, etc. Those should not exist on the graphics engine side.
    . Separate gameplay structs into their own mod.
    . Gameplay structs will then contain a SpriteID, and the logic update will happen outside the "engine"