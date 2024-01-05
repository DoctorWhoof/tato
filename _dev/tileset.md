
| Header        | Size (bytes)  | Note                                      |
|---------------|---------------|-------------------------------------------|
| Text          | 11            | 'tileset_1.0'                             |
| Tile Width    | 1             | 255 max                                   |
| Tile Heigh    | 1             | 255 max                                   |
| Pixel Count   | 2             | 65535 max                                 |
| Font Count    | 1             | 255 max                                   |
| Anim Count    | 1             | 255 max                                   |
| Maps Count    | 1             | 255 max                                   |
| Pixel Section | variable      |                                           |
| Group Section | variable      |                                           |
| Anim Section  | variable      |                                           |
| Maps Section  | variable      |                                           |

| Pixel Section | Size (bytes)  | Note                                      |
|---------------|---------------|-------------------------------------------|
| Pixels        | variable      | 1 byte per pixel                          |

| Font Section  | Size (bytes)  | Note                                      |
|---------------|---------------|-------------------------------------------|
| Font Id       | 1             |                                           |
| Start Index   | 1             |                                           |
| Length        | 1             |                                           |

| Anim Section  | Size (bytes)  | Note                                      |
|---------------|---------------|-------------------------------------------|
| Anim ID       | 1             |                                           |
| Group ID      | 1             | 0 means no group                          |
| FPS           | 1             |                                           |
| Length        | 1             |                                           |
| Frames        | Variable      |                                           |

| Maps Section  | Size (bytes)  | Note                                      |
|---------------|---------------|-------------------------------------------|
| Map ID        | 1             |                                           |
| Columns       | 1             |                                           |
| Rows          | 1             |                                           |
| Tiles         | Variable      | (index, flag); Cols x Rows x Frames x 2   |


