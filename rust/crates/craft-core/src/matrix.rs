//! Port of `src/matrix.c`. Column-major 4×4 matrices, matching Craft's layout
//! and evaluation order (including mixed float/double promotions).

/// Same digits as `PI` in `util.h` (must not use `std`'s PI — different value).
#[allow(clippy::approx_constant)]
const PI: f64 = 3.141_592_653_59;

pub fn normalize(x: &mut f32, y: &mut f32, z: &mut f32) {
    let d = (*x * *x + *y * *y + *z * *z).sqrt();
    *x /= d;
    *y /= d;
    *z /= d;
}

/// In-place normalize of a 3-vector (avoids borrowing three array slots).
pub fn normalize3(v: &mut [f32; 3]) {
    let d = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    v[0] /= d;
    v[1] /= d;
    v[2] /= d;
}

pub fn mat_identity(matrix: &mut [f32; 16]) {
    *matrix = [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ];
}

pub fn mat_translate(matrix: &mut [f32; 16], dx: f32, dy: f32, dz: f32) {
    *matrix = [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, dx, dy, dz, 1.0,
    ];
}

pub fn mat_rotate(matrix: &mut [f32; 16], mut x: f32, mut y: f32, mut z: f32, angle: f32) {
    normalize(&mut x, &mut y, &mut z);
    let s = angle.sin();
    let c = angle.cos();
    let m = 1.0 - c;
    matrix[0] = m * x * x + c;
    matrix[1] = m * x * y - z * s;
    matrix[2] = m * z * x + y * s;
    matrix[3] = 0.0;
    matrix[4] = m * x * y + z * s;
    matrix[5] = m * y * y + c;
    matrix[6] = m * y * z - x * s;
    matrix[7] = 0.0;
    matrix[8] = m * z * x - y * s;
    matrix[9] = m * y * z + x * s;
    matrix[10] = m * z * z + c;
    matrix[11] = 0.0;
    matrix[12] = 0.0;
    matrix[13] = 0.0;
    matrix[14] = 0.0;
    matrix[15] = 1.0;
}

pub fn mat_vec_multiply(vector: &mut [f32; 4], a: [f32; 16], b: [f32; 4]) {
    let mut result = [0.0f32; 4];
    for i in 0..4 {
        let mut total = 0.0f32;
        for j in 0..4 {
            let p = j * 4 + i;
            let q = j;
            total += a[p] * b[q];
        }
        result[i] = total;
    }
    *vector = result;
}

pub fn mat_multiply(matrix: &mut [f32; 16], a: [f32; 16], b: [f32; 16]) {
    let mut result = [0.0f32; 16];
    for c in 0..4 {
        for r in 0..4 {
            let index = c * 4 + r;
            let mut total = 0.0f32;
            for i in 0..4 {
                let p = i * 4 + r;
                let q = c * 4 + i;
                total += a[p] * b[q];
            }
            result[index] = total;
        }
    }
    *matrix = result;
}

pub fn mat_apply(data: &mut [f32], matrix: &[f32; 16], count: i32, offset: i32, stride: i32) {
    for i in 0..count {
        let base = (offset + stride * i) as usize;
        let mut vec = [data[base], data[base + 1], data[base + 2], 1.0];
        let b = vec;
        mat_vec_multiply(&mut vec, *matrix, b);
        data[base] = vec[0];
        data[base + 1] = vec[1];
        data[base + 2] = vec[2];
    }
}

pub fn frustum_planes(planes: &mut [[f32; 4]; 6], radius: i32, matrix: &[f32; 16]) {
    let znear = 0.125f32;
    let zfar = (radius * 32 + 64) as f32;
    let m = matrix;
    planes[0][0] = m[3] + m[0];
    planes[0][1] = m[7] + m[4];
    planes[0][2] = m[11] + m[8];
    planes[0][3] = m[15] + m[12];
    planes[1][0] = m[3] - m[0];
    planes[1][1] = m[7] - m[4];
    planes[1][2] = m[11] - m[8];
    planes[1][3] = m[15] - m[12];
    planes[2][0] = m[3] + m[1];
    planes[2][1] = m[7] + m[5];
    planes[2][2] = m[11] + m[9];
    planes[2][3] = m[15] + m[13];
    planes[3][0] = m[3] - m[1];
    planes[3][1] = m[7] - m[5];
    planes[3][2] = m[11] - m[9];
    planes[3][3] = m[15] - m[13];
    planes[4][0] = znear * m[3] + m[2];
    planes[4][1] = znear * m[7] + m[6];
    planes[4][2] = znear * m[11] + m[10];
    planes[4][3] = znear * m[15] + m[14];
    planes[5][0] = zfar * m[3] - m[2];
    planes[5][1] = zfar * m[7] - m[6];
    planes[5][2] = zfar * m[11] - m[10];
    planes[5][3] = zfar * m[15] - m[14];
}

pub fn mat_frustum(
    matrix: &mut [f32; 16],
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    znear: f32,
    zfar: f32,
) {
    // `2.0 * znear` is computed in double in C (literal 2.0).
    let temp = (2.0f64 * f64::from(znear)) as f32;
    let temp2 = right - left;
    let temp3 = top - bottom;
    let temp4 = zfar - znear;
    matrix[0] = temp / temp2;
    matrix[1] = 0.0;
    matrix[2] = 0.0;
    matrix[3] = 0.0;
    matrix[4] = 0.0;
    matrix[5] = temp / temp3;
    matrix[6] = 0.0;
    matrix[7] = 0.0;
    matrix[8] = (right + left) / temp2;
    matrix[9] = (top + bottom) / temp3;
    matrix[10] = (-zfar - znear) / temp4;
    matrix[11] = -1.0;
    matrix[12] = 0.0;
    matrix[13] = 0.0;
    matrix[14] = (-temp * zfar) / temp4;
    matrix[15] = 0.0;
}

pub fn mat_perspective(matrix: &mut [f32; 16], fov: f32, aspect: f32, znear: f32, zfar: f32) {
    // `fov * PI / 360.0` is double in C before narrowing into tanf.
    let ymax = znear * ((f64::from(fov) * PI / 360.0) as f32).tan();
    let xmax = ymax * aspect;
    mat_frustum(matrix, -xmax, xmax, -ymax, ymax, znear, zfar);
}

pub fn mat_ortho(
    matrix: &mut [f32; 16],
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    near: f32,
    far: f32,
) {
    matrix[0] = 2.0 / (right - left);
    matrix[1] = 0.0;
    matrix[2] = 0.0;
    matrix[3] = 0.0;
    matrix[4] = 0.0;
    matrix[5] = 2.0 / (top - bottom);
    matrix[6] = 0.0;
    matrix[7] = 0.0;
    matrix[8] = 0.0;
    matrix[9] = 0.0;
    matrix[10] = -2.0 / (far - near);
    matrix[11] = 0.0;
    matrix[12] = -(right + left) / (right - left);
    matrix[13] = -(top + bottom) / (top - bottom);
    matrix[14] = -(far + near) / (far - near);
    matrix[15] = 1.0;
}

pub fn set_matrix_2d(matrix: &mut [f32; 16], width: i32, height: i32) {
    mat_ortho(matrix, 0.0, width as f32, 0.0, height as f32, -1.0, 1.0);
}

pub fn set_matrix_3d(
    matrix: &mut [f32; 16],
    width: i32,
    height: i32,
    x: f32,
    y: f32,
    z: f32,
    rx: f32,
    ry: f32,
    fov: f32,
    ortho: i32,
    radius: i32,
) {
    let mut a = [0.0f32; 16];
    let mut b = [0.0f32; 16];
    let aspect = width as f32 / height as f32;
    let znear = 0.125f32;
    let zfar = (radius * 32 + 64) as f32;
    mat_identity(&mut a);
    mat_translate(&mut b, -x, -y, -z);
    let prev = a;
    mat_multiply(&mut a, b, prev);
    mat_rotate(&mut b, rx.cos(), 0.0, rx.sin(), ry);
    let prev = a;
    mat_multiply(&mut a, b, prev);
    mat_rotate(&mut b, 0.0, 1.0, 0.0, -rx);
    let prev = a;
    mat_multiply(&mut a, b, prev);
    if ortho != 0 {
        let size = ortho as f32;
        mat_ortho(
            &mut b,
            -size * aspect,
            size * aspect,
            -size,
            size,
            -zfar,
            zfar,
        );
    } else {
        mat_perspective(&mut b, fov, aspect, znear, zfar);
    }
    let prev = a;
    mat_multiply(&mut a, b, prev);
    mat_identity(matrix);
    let prev = *matrix;
    mat_multiply(matrix, a, prev);
}

pub fn set_matrix_item(matrix: &mut [f32; 16], width: i32, height: i32, scale: i32) {
    let mut a = [0.0f32; 16];
    let mut b = [0.0f32; 16];
    let aspect = width as f32 / height as f32;
    let size = (64 * scale) as f32;
    let box_ = height as f32 / size / 2.0;
    let xoffset = 1.0 - size / width as f32 * 2.0;
    let yoffset = 1.0 - size / height as f32 * 2.0;
    mat_identity(&mut a);
    mat_rotate(&mut b, 0.0, 1.0, 0.0, (-PI / 4.0) as f32);
    let prev = a;
    mat_multiply(&mut a, b, prev);
    mat_rotate(&mut b, 1.0, 0.0, 0.0, (-PI / 10.0) as f32);
    let prev = a;
    mat_multiply(&mut a, b, prev);
    mat_ortho(
        &mut b,
        -box_ * aspect,
        box_ * aspect,
        -box_,
        box_,
        -1.0,
        1.0,
    );
    let prev = a;
    mat_multiply(&mut a, b, prev);
    mat_translate(&mut b, -xoffset, -yoffset, 0.0);
    let prev = a;
    mat_multiply(&mut a, b, prev);
    mat_identity(matrix);
    let prev = *matrix;
    mat_multiply(matrix, a, prev);
}
