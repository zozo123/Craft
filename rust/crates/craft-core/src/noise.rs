//! Port of `deps/noise/noise.c` (simplex noise, after Casey Duncan's
//! <https://github.com/caseman/noise>).
//!
//! Arithmetic order is kept identical to the C so results stay bit-comparable
//! against the golden fixtures. Expressions here are deliberately not
//! simplified or reassociated.

// F2/G2 are written with the exact double-precision digits from the C source
// (`0.3660254037844386f`, `0.21132486540518713f`). They round to the same f32
// as any shorter form; the long literals are kept to preserve provenance.
#[allow(clippy::excessive_precision)]
const F2: f32 = 0.366_025_403_784_438_6;
#[allow(clippy::excessive_precision)]
const G2: f32 = 0.211_324_865_405_187_13;
const F3: f32 = 1.0 / 3.0;
const G3: f32 = 1.0 / 6.0;

#[rustfmt::skip]
const GRAD3: [[f32; 3]; 16] = [
    [ 1.0,  1.0, 0.0], [-1.0,  1.0, 0.0], [ 1.0, -1.0, 0.0], [-1.0, -1.0, 0.0],
    [ 1.0,  0.0, 1.0], [-1.0,  0.0, 1.0], [ 1.0,  0.0,-1.0], [-1.0,  0.0,-1.0],
    [ 0.0,  1.0, 1.0], [ 0.0, -1.0, 1.0], [ 0.0,  1.0,-1.0], [ 0.0, -1.0,-1.0],
    [ 1.0,  0.0,-1.0], [-1.0,  0.0,-1.0], [ 0.0, -1.0, 1.0], [ 0.0,  1.0, 1.0],
];

/// Ken Perlin's permutation table, duplicated to 512 entries so the nested
/// lookups below never need a modulo.
#[rustfmt::skip]
const PERM_BASE: [u8; 256] = [
    151, 160, 137,  91,  90,  15, 131,  13, 201,  95,  96,  53, 194, 233,   7, 225,
    140,  36, 103,  30,  69, 142,   8,  99,  37, 240,  21,  10,  23, 190,   6, 148,
    247, 120, 234,  75,   0,  26, 197,  62,  94, 252, 219, 203, 117,  35,  11,  32,
     57, 177,  33,  88, 237, 149,  56,  87, 174,  20, 125, 136, 171, 168,  68, 175,
     74, 165,  71, 134, 139,  48,  27, 166,  77, 146, 158, 231,  83, 111, 229, 122,
     60, 211, 133, 230, 220, 105,  92,  41,  55,  46, 245,  40, 244, 102, 143,  54,
     65,  25,  63, 161,   1, 216,  80,  73, 209,  76, 132, 187, 208,  89,  18, 169,
    200, 196, 135, 130, 116, 188, 159,  86, 164, 100, 109, 198, 173, 186,   3,  64,
     52, 217, 226, 250, 124, 123,   5, 202,  38, 147, 118, 126, 255,  82,  85, 212,
    207, 206,  59, 227,  47,  16,  58,  17, 182, 189,  28,  42, 223, 183, 170, 213,
    119, 248, 152,   2,  44, 154, 163,  70, 221, 153, 101, 155, 167,  43, 172,   9,
    129,  22,  39, 253,  19,  98, 108, 110,  79, 113, 224, 232, 178, 185, 112, 104,
    218, 246,  97, 228, 251,  34, 242, 193, 238, 210, 144,  12, 191, 179, 162, 241,
     81,  51, 145, 235, 249,  14, 239, 107,  49, 192, 214,  31, 181, 199, 106, 157,
    184,  84, 204, 176, 115, 121,  50,  45, 127,   4, 150, 254, 138, 236, 205,  93,
    222, 114,  67,  29,  24,  72, 243, 141, 128, 195,  78,  66, 215,  61, 156, 180,
];

const PERM: [u8; 512] = {
    let mut p = [0u8; 512];
    let mut i = 0;
    while i < 256 {
        p[i] = PERM_BASE[i];
        p[i + 256] = PERM_BASE[i];
        i += 1;
    }
    p
};

#[inline]
fn perm(i: usize) -> usize {
    PERM[i] as usize
}

pub fn noise2(x: f32, y: f32) -> f32 {
    let s = (x + y) * F2;
    let i = (x + s).floor();
    let j = (y + s).floor();
    let t = (i + j) * G2;

    let mut xx = [0.0f32; 3];
    let mut yy = [0.0f32; 3];
    let mut f = [0.0f32; 3];
    let mut noise = [0.0f32; 3];
    let mut g = [0usize; 3];

    xx[0] = x - (i - t);
    yy[0] = y - (j - t);

    let i1 = usize::from(xx[0] > yy[0]);
    let j1 = usize::from(xx[0] <= yy[0]);

    xx[2] = xx[0] + G2 * 2.0 - 1.0;
    yy[2] = yy[0] + G2 * 2.0 - 1.0;
    xx[1] = xx[0] - i1 as f32 + G2;
    yy[1] = yy[0] - j1 as f32 + G2;

    let big_i = (i as i32 & 255) as usize;
    let big_j = (j as i32 & 255) as usize;
    g[0] = perm(big_i + perm(big_j)) % 12;
    g[1] = perm(big_i + i1 + perm(big_j + j1)) % 12;
    g[2] = perm(big_i + 1 + perm(big_j + 1)) % 12;

    for c in 0..=2 {
        f[c] = 0.5 - xx[c] * xx[c] - yy[c] * yy[c];
    }

    for c in 0..=2 {
        if f[c] > 0.0 {
            noise[c] =
                f[c] * f[c] * f[c] * f[c] * (GRAD3[g[c]][0] * xx[c] + GRAD3[g[c]][1] * yy[c]);
        }
    }

    (noise[0] + noise[1] + noise[2]) * 70.0
}

pub fn noise3(x: f32, y: f32, z: f32) -> f32 {
    let s = (x + y + z) * F3;
    let i = (x + s).floor();
    let j = (y + s).floor();
    let k = (z + s).floor();
    let t = (i + j + k) * G3;

    let mut pos = [[0.0f32; 3]; 4];
    let mut f = [0.0f32; 4];
    let mut noise = [0.0f32; 4];
    let mut g = [0usize; 4];

    pos[0][0] = x - (i - t);
    pos[0][1] = y - (j - t);
    pos[0][2] = z - (k - t);

    let (o1, o2): ([usize; 3], [usize; 3]) = if pos[0][0] >= pos[0][1] {
        if pos[0][1] >= pos[0][2] {
            ([1, 0, 0], [1, 1, 0])
        } else if pos[0][0] >= pos[0][2] {
            ([1, 0, 0], [1, 0, 1])
        } else {
            ([0, 0, 1], [1, 0, 1])
        }
    } else if pos[0][1] < pos[0][2] {
        ([0, 0, 1], [0, 1, 1])
    } else if pos[0][0] < pos[0][2] {
        ([0, 1, 0], [0, 1, 1])
    } else {
        ([0, 1, 0], [1, 1, 0])
    };

    for c in 0..=2 {
        pos[3][c] = pos[0][c] - 1.0 + 3.0 * G3;
        pos[2][c] = pos[0][c] - o2[c] as f32 + 2.0 * G3;
        pos[1][c] = pos[0][c] - o1[c] as f32 + G3;
    }

    let big_i = (i as i32 & 255) as usize;
    let big_j = (j as i32 & 255) as usize;
    let big_k = (k as i32 & 255) as usize;
    g[0] = perm(big_i + perm(big_j + perm(big_k))) % 12;
    g[1] = perm(big_i + o1[0] + perm(big_j + o1[1] + perm(o1[2] + big_k))) % 12;
    g[2] = perm(big_i + o2[0] + perm(big_j + o2[1] + perm(o2[2] + big_k))) % 12;
    g[3] = perm(big_i + 1 + perm(big_j + 1 + perm(big_k + 1))) % 12;

    for c in 0..=3 {
        f[c] = 0.6 - pos[c][0] * pos[c][0] - pos[c][1] * pos[c][1] - pos[c][2] * pos[c][2];
    }

    for c in 0..=3 {
        if f[c] > 0.0 {
            let dot = pos[c][0] * GRAD3[g[c]][0]
                + pos[c][1] * GRAD3[g[c]][1]
                + pos[c][2] * GRAD3[g[c]][2];
            noise[c] = f[c] * f[c] * f[c] * f[c] * dot;
        }
    }

    (noise[0] + noise[1] + noise[2] + noise[3]) * 32.0
}

pub fn simplex2(x: f32, y: f32, octaves: i32, persistence: f32, lacunarity: f32) -> f32 {
    let mut freq = 1.0f32;
    let mut amp = 1.0f32;
    let mut max = 1.0f32;
    let mut total = noise2(x, y);
    for _ in 1..octaves {
        freq *= lacunarity;
        amp *= persistence;
        max += amp;
        total += noise2(x * freq, y * freq) * amp;
    }
    (1.0 + total / max) / 2.0
}

pub fn simplex3(x: f32, y: f32, z: f32, octaves: i32, persistence: f32, lacunarity: f32) -> f32 {
    let mut freq = 1.0f32;
    let mut amp = 1.0f32;
    let mut max = 1.0f32;
    let mut total = noise3(x, y, z);
    for _ in 1..octaves {
        freq *= lacunarity;
        amp *= persistence;
        max += amp;
        total += noise3(x * freq, y * freq, z * freq) * amp;
    }
    (1.0 + total / max) / 2.0
}
