# Main Sections

| Section Name  | Size (bytes) |   Note                                       |
|---------------|--------------|----------------------------------------------|
| Header        | 6            | 'scn1.0'                                     |
| Camera        | 8            | 4 bytes per f32 coordinate                   |
| Tile Count    | 2            | Max 65536 tiles                              |
| Tile Length   | 2            | How many pixels per tile                     |
| Tile pixels   | Variable     | Tile Count * Tile Length                     |
| Frame count   | 2            |                                              |
| Frames        | Variable     | One item per frame                           |
| Anim count    | 2            |                                              |
| Anims         | Variable     | One item per animation                       |
| Entity count  | 2            |                                              |
| Entities      | Variable     | One item per entity                          |


### Repeating item sub-sections

| Frames        | Size (bytes) | Note                                         |
|---------------|--------------|----------------------------------------------|
| Frame Length  | 1            | Each frame can have up to 256 tiles          |
| Frame Data    | Variable     | 1 byte per tile                              |

| Animations    | Size (bytes) | Note                                         |
|---------------|--------------|----------------------------------------------|
| Anim Length   | 1            | Each anim can have up to 256 frames          |
| Anim Data     | Variable     | 1 byte per frame                             |

| Entities      | Size (bytes) | Note                                         |
|---------------|--------------|----------------------------------------------|
| Shape Kind    | 1            |                                              |
| Shape Data    | 4            | Up to 4 bytes can be used for enum payload   |
| Position      | 8            | 4 bytes per f32 coordinate                   |
| Collider      | 4            | 4 i8 values for x,y,w,h, all zero if none    |
| Render Offset | 2            | 2 i8 values for x,y                          |
