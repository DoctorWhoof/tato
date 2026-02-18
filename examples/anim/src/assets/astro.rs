// Auto-generated code. Do not edit manually!
#![allow(unused)]
use tato::prelude::*;

pub const BANK_ASTRO: Bank = Bank {
    colors: COLORS_ASTRO,
    tiles: TileBank::from_tiles(&TILES_ASTRO),
};

pub const COLORS_ASTRO: ColorBank = ColorBank::new_from(&[
    RGBA12::with_transparency(0, 0, 0, 0),
    RGBA12::with_transparency(7, 7, 7, 7),
    RGBA12::with_transparency(0, 0, 0, 7),
    RGBA12::with_transparency(1, 1, 6, 7),
]);

pub const TILES_ASTRO: [Tile<2>; 41] = [
    Tile::<2>::new(0x0000000000000000, 0x0000000000000000),
    Tile::<2>::new(0x0000001500550155, 0x0155015501590059),
    Tile::<2>::new(0x0015004001550145, 0x0045001500140000),
    Tile::<2>::new(0x0015005501550155, 0x0155015500550015),
    Tile::<2>::new(0x0040015501550055, 0x0015001400000000),
    Tile::<2>::new(0x0000000000150055, 0x0155015501550155),
    Tile::<2>::new(0x0000000054005500, 0x5540554056405640),
    Tile::<2>::new(0x0055000500150011, 0x0011001400050001),
    Tile::<2>::new(0x0000000000150055, 0x0155015501550159),
    Tile::<2>::new(0x0059001500400151, 0x0151004500150000),
    Tile::<2>::new(0x6500544001405500, 0x5400540054001400),
    Tile::<2>::new(0x0000001500550155, 0x0155015501550055),
    Tile::<2>::new(0x0015004001550155, 0x0055000500000000),
    Tile::<2>::new(0x5440014055005400, 0x5400540014000000),
    Tile::<2>::new(0x0140015500550015, 0x0015001400000000),
    Tile::<2>::new(0x0000540055005540, 0x5540564056405500),
    Tile::<2>::new(0x0005001500550114, 0x0141000500000000),
    Tile::<2>::new(0x5000540015005500, 0x5400554005400000),
    Tile::<2>::new(0x0005001500150004, 0x0055015500500000),
    Tile::<2>::new(0x5000440051005140, 0x0540540000000000),
    Tile::<2>::new(0x0000000000000000, 0x0000000000150055),
    Tile::<2>::new(0x0155015501550159, 0x0F5903FF003F03C3),
    Tile::<2>::new(0x5540554055406540, 0x65F0FFC0C3F0F000),
    Tile::<2>::new(0x0000000000000000, 0x0015005501550155),
    Tile::<2>::new(0x0155015501550A55, 0x02AA002A0A02000A),
    Tile::<2>::new(0x55405540554055A0, 0xAA80AAA0A800A000),
    Tile::<2>::new(0x0155015501550955, 0x2AAA00AA0002002A),
    Tile::<2>::new(0x5540554056405670, 0xFFFCFFC0FCF0C000),
    Tile::<2>::new(0x015531550D550D59, 0x0F5903FF000F0000),
    Tile::<2>::new(0x5540554C557C6570, 0x65F0FFC0FC000000),
    Tile::<2>::new(0x0000000000000000, 0x0000001500550155),
    Tile::<2>::new(0x0155015501590D59, 0x0F5503FF003F0000),
    Tile::<2>::new(0x5540554065406570, 0x55F0FFC0C0000000),
    Tile::<2>::new(0x0155015509550A55, 0x02AA002A00020000),
    Tile::<2>::new(0x55405540556055A0, 0xAA80AA00A0000000),
    Tile::<2>::new(0x0155095509550255, 0x00AA000A00020000),
    Tile::<2>::new(0x5540556055605580, 0xAA80A800A0008000),
    Tile::<2>::new(0x0155015509552A55, 0x0AAA00AA00000000),
    Tile::<2>::new(0x55405640567055C0, 0xFF00FC0000000000),
    Tile::<2>::new(0x015501552955AA55, 0x0AAA00AA00000000),
    Tile::<2>::new(0x55405540567056FC, 0xFFC0F00000000000),
];

pub const STRIP_ASTRO: [TilemapRef; 24] = [
    TilemapRef { cells: &FRAMES_ASTRO[0], columns: 2, rows: 2 },
    TilemapRef { cells: &FRAMES_ASTRO[1], columns: 2, rows: 2 },
    TilemapRef { cells: &FRAMES_ASTRO[2], columns: 2, rows: 2 },
    TilemapRef { cells: &FRAMES_ASTRO[3], columns: 2, rows: 2 },
    TilemapRef { cells: &FRAMES_ASTRO[4], columns: 2, rows: 2 },
    TilemapRef { cells: &FRAMES_ASTRO[5], columns: 2, rows: 2 },
    TilemapRef { cells: &FRAMES_ASTRO[6], columns: 2, rows: 2 },
    TilemapRef { cells: &FRAMES_ASTRO[7], columns: 2, rows: 2 },
    TilemapRef { cells: &FRAMES_ASTRO[8], columns: 2, rows: 2 },
    TilemapRef { cells: &FRAMES_ASTRO[9], columns: 2, rows: 2 },
    TilemapRef { cells: &FRAMES_ASTRO[10], columns: 2, rows: 2 },
    TilemapRef { cells: &FRAMES_ASTRO[11], columns: 2, rows: 2 },
    TilemapRef { cells: &FRAMES_ASTRO[12], columns: 2, rows: 2 },
    TilemapRef { cells: &FRAMES_ASTRO[13], columns: 2, rows: 2 },
    TilemapRef { cells: &FRAMES_ASTRO[14], columns: 2, rows: 2 },
    TilemapRef { cells: &FRAMES_ASTRO[15], columns: 2, rows: 2 },
    TilemapRef { cells: &FRAMES_ASTRO[16], columns: 2, rows: 2 },
    TilemapRef { cells: &FRAMES_ASTRO[17], columns: 2, rows: 2 },
    TilemapRef { cells: &FRAMES_ASTRO[18], columns: 2, rows: 2 },
    TilemapRef { cells: &FRAMES_ASTRO[19], columns: 2, rows: 2 },
    TilemapRef { cells: &FRAMES_ASTRO[20], columns: 2, rows: 2 },
    TilemapRef { cells: &FRAMES_ASTRO[21], columns: 2, rows: 2 },
    TilemapRef { cells: &FRAMES_ASTRO[22], columns: 2, rows: 2 },
    TilemapRef { cells: &FRAMES_ASTRO[23], columns: 2, rows: 2 },
];

pub const FRAMES_ASTRO: [[Cell; 4]; 24] = [
    [Cell::new(1, 0, 288), Cell::new(1, 128, 288), Cell::new(2, 0, 256), Cell::new(2, 128, 256)],
    [Cell::new(3, 0, 256), Cell::new(3, 128, 256), Cell::new(4, 0, 256), Cell::new(4, 128, 256)],
    [Cell::new(5, 0, 256), Cell::new(6, 0, 288), Cell::new(7, 0, 256), Cell::new(7, 128, 256)],
    [Cell::new(1, 0, 288), Cell::new(1, 128, 288), Cell::new(2, 0, 256), Cell::new(2, 128, 256)],
    [Cell::new(8, 0, 288), Cell::new(8, 128, 288), Cell::new(9, 0, 288), Cell::new(10, 0, 288)],
    [Cell::new(1, 0, 288), Cell::new(1, 128, 288), Cell::new(2, 0, 256), Cell::new(2, 128, 256)],
    [Cell::new(8, 0, 288), Cell::new(8, 128, 288), Cell::new(10, 128, 288), Cell::new(9, 128, 288)],
    [Cell::new(3, 0, 256), Cell::new(3, 128, 256), Cell::new(4, 0, 256), Cell::new(4, 128, 256)],
    [Cell::new(11, 0, 256), Cell::new(11, 128, 256), Cell::new(12, 0, 256), Cell::new(13, 0, 256)],
    [Cell::new(3, 0, 256), Cell::new(3, 128, 256), Cell::new(14, 0, 256), Cell::new(14, 128, 256)],
    [
        Cell::new(11, 0, 256),
        Cell::new(11, 128, 256),
        Cell::new(13, 128, 256),
        Cell::new(12, 128, 256),
    ],
    [Cell::new(5, 0, 256), Cell::new(6, 0, 288), Cell::new(7, 0, 256), Cell::new(7, 128, 256)],
    [Cell::new(11, 0, 256), Cell::new(15, 0, 288), Cell::new(16, 0, 256), Cell::new(17, 0, 256)],
    [Cell::new(5, 0, 256), Cell::new(6, 0, 288), Cell::new(7, 0, 256), Cell::new(7, 128, 256)],
    [Cell::new(11, 0, 256), Cell::new(15, 0, 288), Cell::new(18, 0, 256), Cell::new(19, 0, 256)],
    [Cell::new(20, 0, 256), Cell::new(20, 128, 256), Cell::new(21, 0, 291), Cell::new(22, 0, 291)],
    [Cell::new(23, 0, 256), Cell::new(23, 128, 256), Cell::new(24, 0, 304), Cell::new(25, 0, 304)],
    [Cell::new(20, 0, 256), Cell::new(20, 128, 256), Cell::new(26, 0, 304), Cell::new(27, 0, 291)],
    [Cell::new(20, 0, 256), Cell::new(20, 128, 256), Cell::new(28, 0, 291), Cell::new(29, 0, 291)],
    [Cell::new(30, 0, 256), Cell::new(30, 128, 256), Cell::new(31, 0, 291), Cell::new(32, 0, 291)],
    [Cell::new(30, 0, 256), Cell::new(30, 128, 256), Cell::new(33, 0, 304), Cell::new(34, 0, 304)],
    [Cell::new(23, 0, 256), Cell::new(23, 128, 256), Cell::new(35, 0, 304), Cell::new(36, 0, 304)],
    [Cell::new(30, 0, 256), Cell::new(30, 128, 256), Cell::new(37, 0, 304), Cell::new(38, 0, 291)],
    [Cell::new(20, 0, 256), Cell::new(20, 128, 256), Cell::new(39, 0, 304), Cell::new(40, 0, 291)],
];

pub const ANIM_DOWN: Anim = Anim {
    fps: 10,
    repeat: true,
    frames: &[4, 5, 6, 5],
    strip: &STRIP_ASTRO,
};

pub const ANIM_UP: Anim = Anim {
    fps: 10,
    repeat: true,
    frames: &[8, 9, 10, 9],
    strip: &STRIP_ASTRO,
};

pub const ANIM_RIGHT: Anim = Anim {
    fps: 10,
    repeat: true,
    frames: &[12, 13, 14, 13],
    strip: &STRIP_ASTRO,
};

pub const EMPTY: Cell = Cell::new(0, 0, 0);
