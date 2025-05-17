
[.] Text and Fonts
    . Write directly to the BG Map
    . Let the function accept a user defined slice of characters so that simple and complex fonts may be used freely.

[ ] Dual chip setup for multiple video layers

[.] Use LIRQ (Line interrupt) to draw GUI
    . Will need to switch Tile bank halfway through
    . Will also need external BG Maps

[ ] Load & Unload Tilesets
    . Wait until external tiles and BG Maps are more stable

### Pipeline:

[x] Load individual named tiles into a tileset, each will be exported as a const
    . On the other end, insert single tiles into TileBank (returns TileID)
