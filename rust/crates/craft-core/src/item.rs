//! Port of `src/item.c` and `src/item.h`.

pub const EMPTY: i32 = 0;
pub const GRASS: i32 = 1;
pub const SAND: i32 = 2;
pub const STONE: i32 = 3;
pub const BRICK: i32 = 4;
pub const WOOD: i32 = 5;
pub const CEMENT: i32 = 6;
pub const DIRT: i32 = 7;
pub const PLANK: i32 = 8;
pub const SNOW: i32 = 9;
pub const GLASS: i32 = 10;
pub const COBBLE: i32 = 11;
pub const LIGHT_STONE: i32 = 12;
pub const DARK_STONE: i32 = 13;
pub const CHEST: i32 = 14;
pub const LEAVES: i32 = 15;
pub const CLOUD: i32 = 16;
pub const TALL_GRASS: i32 = 17;
pub const YELLOW_FLOWER: i32 = 18;
pub const RED_FLOWER: i32 = 19;
pub const PURPLE_FLOWER: i32 = 20;
pub const SUN_FLOWER: i32 = 21;
pub const WHITE_FLOWER: i32 = 22;
pub const BLUE_FLOWER: i32 = 23;

/// Blocks the player can place, in hotbar order.
pub const ITEMS: [i32; 54] = [
    GRASS,
    SAND,
    STONE,
    BRICK,
    WOOD,
    CEMENT,
    DIRT,
    PLANK,
    SNOW,
    GLASS,
    COBBLE,
    LIGHT_STONE,
    DARK_STONE,
    CHEST,
    LEAVES,
    TALL_GRASS,
    YELLOW_FLOWER,
    RED_FLOWER,
    PURPLE_FLOWER,
    SUN_FLOWER,
    WHITE_FLOWER,
    BLUE_FLOWER,
    // COLOR_00 through COLOR_31 are contiguous ids 32..=63.
    32,
    33,
    34,
    35,
    36,
    37,
    38,
    39,
    40,
    41,
    42,
    43,
    44,
    45,
    46,
    47,
    48,
    49,
    50,
    51,
    52,
    53,
    54,
    55,
    56,
    57,
    58,
    59,
    60,
    61,
    62,
    63,
];

pub const ITEM_COUNT: usize = ITEMS.len();

/// `w` => (left, right, top, bottom, front, back) texture tiles.
pub static BLOCKS: [[i32; 6]; 256] = build_blocks();

const fn build_blocks() -> [[i32; 6]; 256] {
    let mut t = [[0i32; 6]; 256];
    t[1] = [16, 16, 32, 0, 16, 16]; // grass
    t[2] = [1, 1, 1, 1, 1, 1]; // sand
    t[3] = [2, 2, 2, 2, 2, 2]; // stone
    t[4] = [3, 3, 3, 3, 3, 3]; // brick
    t[5] = [20, 20, 36, 4, 20, 20]; // wood
    t[6] = [5, 5, 5, 5, 5, 5]; // cement
    t[7] = [6, 6, 6, 6, 6, 6]; // dirt
    t[8] = [7, 7, 7, 7, 7, 7]; // plank
    t[9] = [24, 24, 40, 8, 24, 24]; // snow
    t[10] = [9, 9, 9, 9, 9, 9]; // glass
    t[11] = [10, 10, 10, 10, 10, 10]; // cobble
    t[12] = [11, 11, 11, 11, 11, 11]; // light stone
    t[13] = [12, 12, 12, 12, 12, 12]; // dark stone
    t[14] = [13, 13, 13, 13, 13, 13]; // chest
    t[15] = [14, 14, 14, 14, 14, 14]; // leaves
    t[16] = [15, 15, 15, 15, 15, 15]; // cloud

    // Colours 32..=63 map to tiles 176..=207.
    let mut w = 32;
    while w < 64 {
        let tile = 176 + (w - 32);
        t[w as usize] = [tile, tile, tile, tile, tile, tile];
        w += 1;
    }
    t
}

/// `w` => tile, for the cross-billboard plant blocks.
pub static PLANTS: [i32; 256] = build_plants();

const fn build_plants() -> [i32; 256] {
    let mut t = [0i32; 256];
    t[17] = 48; // tall grass
    t[18] = 49; // yellow flower
    t[19] = 50; // red flower
    t[20] = 51; // purple flower
    t[21] = 52; // sun flower
    t[22] = 53; // white flower
    t[23] = 54; // blue flower
    t
}

pub fn is_plant(w: i32) -> bool {
    matches!(
        w,
        TALL_GRASS
            | YELLOW_FLOWER
            | RED_FLOWER
            | PURPLE_FLOWER
            | SUN_FLOWER
            | WHITE_FLOWER
            | BLUE_FLOWER
    )
}

pub fn is_obstacle(w: i32) -> bool {
    let w = w.abs();
    if is_plant(w) {
        return false;
    }
    !matches!(w, EMPTY | CLOUD)
}

pub fn is_transparent(w: i32) -> bool {
    if w == EMPTY {
        return true;
    }
    let w = w.abs();
    if is_plant(w) {
        return true;
    }
    matches!(w, EMPTY | GLASS | LEAVES)
}

/// Note the missing `abs()`: unlike the predicates above, the C original
/// tests `w` directly, so a border block stored as `-16` is destructable
/// while cloud `16` is not.
pub fn is_destructable(w: i32) -> bool {
    !matches!(w, EMPTY | CLOUD)
}
