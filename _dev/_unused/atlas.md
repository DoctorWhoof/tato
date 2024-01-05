
| Section       | Size (bytes)  | Note                                      |
|---------------|---------------|-------------------------------------------|
| Header        | 9             | 'atlas_1.0'                               |
| Columns       | 1             | 255 max                                   |
| Rows          | 1             | 255 max                                   |
| Tile Width    | 1             | 255 max                                   |
| Tile Heigh    | 1             | 255 max                                   |
| Tile Count    | 2             | 65535 max                                 |
| Group Count   | 1             | 64 max (first 6 bits)                     |
| Group entries | variable      | 18 bytes per entry                        |
| Pixels        | variable      | 1 byte per pixel                          |


| Group section | Size (bytes)  | Note                                      |
|---------------|---------------|-------------------------------------------|
| Name          | 16            | ASCII                                     |
| Range start   | 1             |                                           |
| Range length  | 1             |                                           |