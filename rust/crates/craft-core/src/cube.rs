//! Port of `src/cube.c` — block/plant/player/character/sphere mesh builders.

use crate::item::{BLOCKS, PLANTS};
use crate::matrix::{mat_apply, mat_identity, mat_multiply, mat_rotate, mat_translate, normalize3};

#[allow(clippy::approx_constant)]
const PI: f64 = 3.141_592_653_59;

#[inline]
fn radians(degrees: f32) -> f32 {
    (f64::from(degrees) * PI / 180.0) as f32
}

#[allow(clippy::too_many_arguments)]
pub fn make_cube_faces(
    data: &mut [f32],
    ao: &[[f32; 4]; 6],
    light: &[[f32; 4]; 6],
    left: i32,
    right: i32,
    top: i32,
    bottom: i32,
    front: i32,
    back: i32,
    wleft: i32,
    wright: i32,
    wtop: i32,
    wbottom: i32,
    wfront: i32,
    wback: i32,
    x: f32,
    y: f32,
    z: f32,
    n: f32,
) {
    #[rustfmt::skip]
    const POSITIONS: [[[f32; 3]; 4]; 6] = [
        [[-1.0, -1.0, -1.0], [-1.0, -1.0,  1.0], [-1.0,  1.0, -1.0], [-1.0,  1.0,  1.0]],
        [[ 1.0, -1.0, -1.0], [ 1.0, -1.0,  1.0], [ 1.0,  1.0, -1.0], [ 1.0,  1.0,  1.0]],
        [[-1.0,  1.0, -1.0], [-1.0,  1.0,  1.0], [ 1.0,  1.0, -1.0], [ 1.0,  1.0,  1.0]],
        [[-1.0, -1.0, -1.0], [-1.0, -1.0,  1.0], [ 1.0, -1.0, -1.0], [ 1.0, -1.0,  1.0]],
        [[-1.0, -1.0, -1.0], [-1.0,  1.0, -1.0], [ 1.0, -1.0, -1.0], [ 1.0,  1.0, -1.0]],
        [[-1.0, -1.0,  1.0], [-1.0,  1.0,  1.0], [ 1.0, -1.0,  1.0], [ 1.0,  1.0,  1.0]],
    ];
    #[rustfmt::skip]
    const NORMALS: [[f32; 3]; 6] = [
        [-1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0],
        [0.0, -1.0, 0.0], [0.0, 0.0, -1.0], [0.0, 0.0, 1.0],
    ];
    #[rustfmt::skip]
    const UVS: [[[f32; 2]; 4]; 6] = [
        [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]],
        [[1.0, 0.0], [0.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
        [[0.0, 1.0], [0.0, 0.0], [1.0, 1.0], [1.0, 0.0]],
        [[0.0, 0.0], [0.0, 1.0], [1.0, 0.0], [1.0, 1.0]],
        [[0.0, 0.0], [0.0, 1.0], [1.0, 0.0], [1.0, 1.0]],
        [[1.0, 0.0], [1.0, 1.0], [0.0, 0.0], [0.0, 1.0]],
    ];
    #[rustfmt::skip]
    const INDICES: [[usize; 6]; 6] = [
        [0, 3, 2, 0, 1, 3], [0, 3, 1, 0, 2, 3], [0, 3, 2, 0, 1, 3],
        [0, 3, 1, 0, 2, 3], [0, 3, 2, 0, 1, 3], [0, 3, 1, 0, 2, 3],
    ];
    #[rustfmt::skip]
    const FLIPPED: [[usize; 6]; 6] = [
        [0, 1, 2, 1, 3, 2], [0, 2, 1, 2, 3, 1], [0, 1, 2, 1, 3, 2],
        [0, 2, 1, 2, 3, 1], [0, 1, 2, 1, 3, 2], [0, 2, 1, 2, 3, 1],
    ];

    let s = 0.0625f32;
    // `1 / 2048.0` is double division in C.
    let a = 0.0 + (1.0f64 / 2048.0) as f32;
    let b = s - (1.0f64 / 2048.0) as f32;
    let faces = [left, right, top, bottom, front, back];
    let tiles = [wleft, wright, wtop, wbottom, wfront, wback];
    let mut d = 0usize;
    for i in 0..6 {
        if faces[i] == 0 {
            continue;
        }
        let du = (tiles[i] % 16) as f32 * s;
        let dv = (tiles[i] / 16) as f32 * s;
        let flip = ao[i][0] + ao[i][3] > ao[i][1] + ao[i][2];
        for v in 0..6 {
            let j = if flip { FLIPPED[i][v] } else { INDICES[i][v] };
            data[d] = x + n * POSITIONS[i][j][0];
            d += 1;
            data[d] = y + n * POSITIONS[i][j][1];
            d += 1;
            data[d] = z + n * POSITIONS[i][j][2];
            d += 1;
            data[d] = NORMALS[i][0];
            d += 1;
            data[d] = NORMALS[i][1];
            d += 1;
            data[d] = NORMALS[i][2];
            d += 1;
            data[d] = du + if UVS[i][j][0] != 0.0 { b } else { a };
            d += 1;
            data[d] = dv + if UVS[i][j][1] != 0.0 { b } else { a };
            d += 1;
            data[d] = ao[i][j];
            d += 1;
            data[d] = light[i][j];
            d += 1;
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn make_cube(
    data: &mut [f32],
    ao: &[[f32; 4]; 6],
    light: &[[f32; 4]; 6],
    left: i32,
    right: i32,
    top: i32,
    bottom: i32,
    front: i32,
    back: i32,
    x: f32,
    y: f32,
    z: f32,
    n: f32,
    w: i32,
) {
    let wleft = BLOCKS[w as usize][0];
    let wright = BLOCKS[w as usize][1];
    let wtop = BLOCKS[w as usize][2];
    let wbottom = BLOCKS[w as usize][3];
    let wfront = BLOCKS[w as usize][4];
    let wback = BLOCKS[w as usize][5];
    make_cube_faces(
        data, ao, light, left, right, top, bottom, front, back, wleft, wright, wtop, wbottom,
        wfront, wback, x, y, z, n,
    );
}

#[allow(clippy::too_many_arguments)]
pub fn make_plant(
    data: &mut [f32],
    ao: f32,
    light: f32,
    px: f32,
    py: f32,
    pz: f32,
    n: f32,
    w: i32,
    rotation: f32,
) {
    #[rustfmt::skip]
    const POSITIONS: [[[f32; 3]; 4]; 4] = [
        [[ 0.0, -1.0, -1.0], [ 0.0, -1.0,  1.0], [ 0.0,  1.0, -1.0], [ 0.0,  1.0,  1.0]],
        [[ 0.0, -1.0, -1.0], [ 0.0, -1.0,  1.0], [ 0.0,  1.0, -1.0], [ 0.0,  1.0,  1.0]],
        [[-1.0, -1.0,  0.0], [-1.0,  1.0,  0.0], [ 1.0, -1.0,  0.0], [ 1.0,  1.0,  0.0]],
        [[-1.0, -1.0,  0.0], [-1.0,  1.0,  0.0], [ 1.0, -1.0,  0.0], [ 1.0,  1.0,  0.0]],
    ];
    #[rustfmt::skip]
    const NORMALS: [[f32; 3]; 4] = [
        [-1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, -1.0], [0.0, 0.0, 1.0],
    ];
    #[rustfmt::skip]
    const UVS: [[[f32; 2]; 4]; 4] = [
        [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]],
        [[1.0, 0.0], [0.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
        [[0.0, 0.0], [0.0, 1.0], [1.0, 0.0], [1.0, 1.0]],
        [[1.0, 0.0], [1.0, 1.0], [0.0, 0.0], [0.0, 1.0]],
    ];
    #[rustfmt::skip]
    const INDICES: [[usize; 6]; 4] = [
        [0, 3, 2, 0, 1, 3], [0, 3, 1, 0, 2, 3], [0, 3, 2, 0, 1, 3], [0, 3, 1, 0, 2, 3],
    ];

    let s = 0.0625f32;
    let a = 0.0f32;
    let b = s;
    let du = (PLANTS[w as usize] % 16) as f32 * s;
    let dv = (PLANTS[w as usize] / 16) as f32 * s;
    let mut d = 0usize;
    for i in 0..4 {
        for v in 0..6 {
            let j = INDICES[i][v];
            data[d] = n * POSITIONS[i][j][0];
            d += 1;
            data[d] = n * POSITIONS[i][j][1];
            d += 1;
            data[d] = n * POSITIONS[i][j][2];
            d += 1;
            data[d] = NORMALS[i][0];
            d += 1;
            data[d] = NORMALS[i][1];
            d += 1;
            data[d] = NORMALS[i][2];
            d += 1;
            data[d] = du + if UVS[i][j][0] != 0.0 { b } else { a };
            d += 1;
            data[d] = dv + if UVS[i][j][1] != 0.0 { b } else { a };
            d += 1;
            data[d] = ao;
            d += 1;
            data[d] = light;
            d += 1;
        }
    }
    let mut ma = [0.0f32; 16];
    let mut mb = [0.0f32; 16];
    mat_identity(&mut ma);
    mat_rotate(&mut mb, 0.0, 1.0, 0.0, radians(rotation));
    let prev = ma;
    mat_multiply(&mut ma, mb, prev);
    mat_apply(data, &ma, 24, 3, 10);
    mat_translate(&mut mb, px, py, pz);
    let prev = ma;
    mat_multiply(&mut ma, mb, prev);
    mat_apply(data, &ma, 24, 0, 10);
}

pub fn make_player(data: &mut [f32], x: f32, y: f32, z: f32, rx: f32, ry: f32) {
    let ao = [[0.0f32; 4]; 6];
    let light = [[0.8f32; 4]; 6];
    make_cube_faces(
        data, &ao, &light, 1, 1, 1, 1, 1, 1, 226, 224, 241, 209, 225, 227, 0.0, 0.0, 0.0, 0.4,
    );
    let mut ma = [0.0f32; 16];
    let mut mb = [0.0f32; 16];
    mat_identity(&mut ma);
    mat_rotate(&mut mb, 0.0, 1.0, 0.0, rx);
    let prev = ma;
    mat_multiply(&mut ma, mb, prev);
    mat_rotate(&mut mb, rx.cos(), 0.0, rx.sin(), -ry);
    let prev = ma;
    mat_multiply(&mut ma, mb, prev);
    mat_apply(data, &ma, 36, 3, 10);
    mat_translate(&mut mb, x, y, z);
    let prev = ma;
    mat_multiply(&mut ma, mb, prev);
    mat_apply(data, &ma, 36, 0, 10);
}

pub fn make_cube_wireframe(data: &mut [f32], x: f32, y: f32, z: f32, n: f32) {
    #[rustfmt::skip]
    const POSITIONS: [[f32; 3]; 8] = [
        [-1.0, -1.0, -1.0], [-1.0, -1.0,  1.0], [-1.0,  1.0, -1.0], [-1.0,  1.0,  1.0],
        [ 1.0, -1.0, -1.0], [ 1.0, -1.0,  1.0], [ 1.0,  1.0, -1.0], [ 1.0,  1.0,  1.0],
    ];
    #[rustfmt::skip]
    const INDICES: [usize; 24] = [
        0, 1, 0, 2, 0, 4, 1, 3, 1, 5, 2, 3, 2, 6, 3, 7, 4, 5, 4, 6, 5, 7, 6, 7,
    ];
    let mut d = 0usize;
    for &j in &INDICES {
        data[d] = x + n * POSITIONS[j][0];
        d += 1;
        data[d] = y + n * POSITIONS[j][1];
        d += 1;
        data[d] = z + n * POSITIONS[j][2];
        d += 1;
    }
}

pub fn make_character(data: &mut [f32], x: f32, y: f32, n: f32, m: f32, c: char) {
    let s = 0.0625f32;
    let a = s;
    let b = s * 2.0;
    let w = c as i32 - 32;
    let du = (w % 16) as f32 * a;
    let dv = 1.0 - (w / 16) as f32 * b - b;
    let mut d = 0usize;
    let write = |d: &mut usize, data: &mut [f32], vx: f32, vy: f32, u: f32, v: f32| {
        data[*d] = vx;
        *d += 1;
        data[*d] = vy;
        *d += 1;
        data[*d] = u;
        *d += 1;
        data[*d] = v;
        *d += 1;
    };
    write(&mut d, data, x - n, y - m, du, dv);
    write(&mut d, data, x + n, y - m, du + a, dv);
    write(&mut d, data, x + n, y + m, du + a, dv + b);
    write(&mut d, data, x - n, y - m, du, dv);
    write(&mut d, data, x + n, y + m, du + a, dv + b);
    write(&mut d, data, x - n, y + m, du, dv + b);
}

pub fn make_character_3d(
    data: &mut [f32],
    mut x: f32,
    mut y: f32,
    mut z: f32,
    n: f32,
    face: i32,
    c: char,
) {
    #[rustfmt::skip]
    const POSITIONS: [[[f32; 3]; 6]; 8] = [
        [[0.0,-2.0,-1.0],[0.0, 2.0, 1.0],[0.0, 2.0,-1.0],[0.0,-2.0,-1.0],[0.0,-2.0, 1.0],[0.0, 2.0, 1.0]],
        [[0.0,-2.0,-1.0],[0.0, 2.0, 1.0],[0.0,-2.0, 1.0],[0.0,-2.0,-1.0],[0.0, 2.0,-1.0],[0.0, 2.0, 1.0]],
        [[-1.0,-2.0,0.0],[ 1.0, 2.0,0.0],[ 1.0,-2.0,0.0],[-1.0,-2.0,0.0],[-1.0, 2.0,0.0],[ 1.0, 2.0,0.0]],
        [[-1.0,-2.0,0.0],[ 1.0,-2.0,0.0],[ 1.0, 2.0,0.0],[-1.0,-2.0,0.0],[ 1.0, 2.0,0.0],[-1.0, 2.0,0.0]],
        [[-1.0,0.0, 2.0],[ 1.0,0.0, 2.0],[ 1.0,0.0,-2.0],[-1.0,0.0, 2.0],[ 1.0,0.0,-2.0],[-1.0,0.0,-2.0]],
        [[-2.0,0.0, 1.0],[ 2.0,0.0,-1.0],[-2.0,0.0,-1.0],[-2.0,0.0, 1.0],[ 2.0,0.0, 1.0],[ 2.0,0.0,-1.0]],
        [[ 1.0,0.0, 2.0],[-1.0,0.0,-2.0],[-1.0,0.0, 2.0],[ 1.0,0.0, 2.0],[ 1.0,0.0,-2.0],[-1.0,0.0,-2.0]],
        [[ 2.0,0.0,-1.0],[-2.0,0.0, 1.0],[ 2.0,0.0, 1.0],[ 2.0,0.0,-1.0],[-2.0,0.0,-1.0],[-2.0,0.0, 1.0]],
    ];
    #[rustfmt::skip]
    const UVS: [[[f32; 2]; 6]; 8] = [
        [[0.0,0.0],[1.0,1.0],[0.0,1.0],[0.0,0.0],[1.0,0.0],[1.0,1.0]],
        [[1.0,0.0],[0.0,1.0],[0.0,0.0],[1.0,0.0],[1.0,1.0],[0.0,1.0]],
        [[1.0,0.0],[0.0,1.0],[0.0,0.0],[1.0,0.0],[1.0,1.0],[0.0,1.0]],
        [[0.0,0.0],[1.0,0.0],[1.0,1.0],[0.0,0.0],[1.0,1.0],[0.0,1.0]],
        [[0.0,0.0],[1.0,0.0],[1.0,1.0],[0.0,0.0],[1.0,1.0],[0.0,1.0]],
        [[0.0,1.0],[1.0,0.0],[1.0,1.0],[0.0,1.0],[0.0,0.0],[1.0,0.0]],
        [[0.0,1.0],[1.0,0.0],[1.0,1.0],[0.0,1.0],[0.0,0.0],[1.0,0.0]],
        [[0.0,1.0],[1.0,0.0],[1.0,1.0],[0.0,1.0],[0.0,0.0],[1.0,0.0]],
    ];
    #[rustfmt::skip]
    const OFFSETS: [[f32; 3]; 8] = [
        [-1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, -1.0], [0.0, 0.0, 1.0],
        [0.0, 1.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0, 0.0],
    ];

    let s = 0.0625f32;
    let pu = s / 5.0;
    let pv = s / 2.5;
    let u1 = pu;
    let v1 = pv;
    let u2 = s - pu;
    let v2 = s * 2.0 - pv;
    let p = 0.5f32;
    let w = c as i32 - 32;
    let du = (w % 16) as f32 * s;
    let dv = 1.0 - (w / 16 + 1) as f32 * s * 2.0;
    let face = face as usize;
    x += p * OFFSETS[face][0];
    y += p * OFFSETS[face][1];
    z += p * OFFSETS[face][2];
    let mut d = 0usize;
    for i in 0..6 {
        data[d] = x + n * POSITIONS[face][i][0];
        d += 1;
        data[d] = y + n * POSITIONS[face][i][1];
        d += 1;
        data[d] = z + n * POSITIONS[face][i][2];
        d += 1;
        data[d] = du + if UVS[face][i][0] != 0.0 { u2 } else { u1 };
        d += 1;
        data[d] = dv + if UVS[face][i][1] != 0.0 { v2 } else { v1 };
        d += 1;
    }
}

fn make_sphere_rec(
    data: &mut [f32],
    r: f32,
    detail: i32,
    a: [f32; 3],
    b: [f32; 3],
    c: [f32; 3],
    ta: [f32; 2],
    tb: [f32; 2],
    tc: [f32; 2],
) -> usize {
    if detail == 0 {
        let mut d = 0usize;
        for (p, t) in [(a, ta), (b, tb), (c, tc)] {
            data[d] = p[0] * r;
            d += 1;
            data[d] = p[1] * r;
            d += 1;
            data[d] = p[2] * r;
            d += 1;
            data[d] = p[0];
            d += 1;
            data[d] = p[1];
            d += 1;
            data[d] = p[2];
            d += 1;
            data[d] = t[0];
            d += 1;
            data[d] = t[1];
            d += 1;
        }
        1
    } else {
        let mut ab = [0.0f32; 3];
        let mut ac = [0.0f32; 3];
        let mut bc = [0.0f32; 3];
        for i in 0..3 {
            ab[i] = (a[i] + b[i]) / 2.0;
            ac[i] = (a[i] + c[i]) / 2.0;
            bc[i] = (b[i] + c[i]) / 2.0;
        }
        normalize3(&mut ab);
        normalize3(&mut ac);
        normalize3(&mut bc);
        // C: `1 - acosf(ab[1]) / PI` with PI the double macro from util.h.
        let tab = [0.0, 1.0 - (f64::from(ab[1].acos()) / PI) as f32];
        let tac = [0.0, 1.0 - (f64::from(ac[1].acos()) / PI) as f32];
        let tbc = [0.0, 1.0 - (f64::from(bc[1].acos()) / PI) as f32];

        let mut total = 0usize;
        let mut offset = 0usize;
        let n = make_sphere_rec(&mut data[offset..], r, detail - 1, a, ab, ac, ta, tab, tac);
        total += n;
        offset += n * 24;
        let n = make_sphere_rec(&mut data[offset..], r, detail - 1, b, bc, ab, tb, tbc, tab);
        total += n;
        offset += n * 24;
        let n = make_sphere_rec(&mut data[offset..], r, detail - 1, c, ac, bc, tc, tac, tbc);
        total += n;
        offset += n * 24;
        let n = make_sphere_rec(
            &mut data[offset..],
            r,
            detail - 1,
            ab,
            bc,
            ac,
            tab,
            tbc,
            tac,
        );
        total += n;
        let _ = offset;
        total
    }
}

pub fn make_sphere(data: &mut [f32], r: f32, detail: i32) {
    #[rustfmt::skip]
    const INDICES: [[usize; 3]; 8] = [
        [4, 3, 0], [1, 4, 0], [3, 4, 5], [4, 1, 5],
        [0, 3, 2], [0, 2, 1], [5, 2, 3], [5, 1, 2],
    ];
    #[rustfmt::skip]
    const POSITIONS: [[f32; 3]; 6] = [
        [0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, -1.0, 0.0],
        [-1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0],
    ];
    #[rustfmt::skip]
    const UVS: [[f32; 2]; 6] = [
        [0.0, 0.5], [0.0, 0.5], [0.0, 0.0], [0.0, 0.5], [0.0, 1.0], [0.0, 0.5],
    ];

    let mut offset = 0usize;
    for i in 0..8 {
        let n = make_sphere_rec(
            &mut data[offset..],
            r,
            detail,
            POSITIONS[INDICES[i][0]],
            POSITIONS[INDICES[i][1]],
            POSITIONS[INDICES[i][2]],
            UVS[INDICES[i][0]],
            UVS[INDICES[i][1]],
            UVS[INDICES[i][2]],
        );
        offset += n * 24;
    }
}
