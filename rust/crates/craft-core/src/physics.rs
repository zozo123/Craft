//! Player motion, collision, and ray hit-testing from `main.c`.
//! Map-scoped variants (no global chunk table) for headless sim.

use crate::config::CHUNK_SIZE;
use crate::item::is_obstacle;
use crate::map::Map;

#[allow(clippy::approx_constant)]
const PI: f64 = 3.141_592_653_59;

#[inline]
fn radians(degrees: f32) -> f32 {
    (f64::from(degrees) * PI / 180.0) as f32
}

pub fn chunked(x: f32) -> i32 {
    (x.round() / CHUNK_SIZE as f32).floor() as i32
}

pub fn get_sight_vector(rx: f32, ry: f32) -> (f32, f32, f32) {
    let m = ry.cos();
    let vx = (rx - radians(90.0)).cos() * m;
    let vy = ry.sin();
    let vz = (rx - radians(90.0)).sin() * m;
    (vx, vy, vz)
}

pub fn get_motion_vector(flying: bool, sz: i32, sx: i32, rx: f32, ry: f32) -> (f32, f32, f32) {
    if sz == 0 && sx == 0 {
        return (0.0, 0.0, 0.0);
    }
    let strafe = (sz as f32).atan2(sx as f32);
    if flying {
        let mut m = ry.cos();
        let mut y = ry.sin();
        if sx != 0 {
            if sz == 0 {
                y = 0.0;
            }
            m = 1.0;
        }
        if sz > 0 {
            y = -y;
        }
        ((rx + strafe).cos() * m, y, (rx + strafe).sin() * m)
    } else {
        ((rx + strafe).cos(), 0.0, (rx + strafe).sin())
    }
}

/// Ray-march through `map`. If `previous`, returns the empty cell before the hit.
pub fn hit_test_map(
    map: &Map,
    max_distance: f32,
    previous: bool,
    mut x: f32,
    mut y: f32,
    mut z: f32,
    vx: f32,
    vy: f32,
    vz: f32,
) -> Option<(i32, i32, i32, i32)> {
    let m = 32i32;
    let mut px = 0i32;
    let mut py = 0i32;
    let mut pz = 0i32;
    let steps = (max_distance * m as f32) as i32;
    for _ in 0..steps {
        let nx = x.round() as i32;
        let ny = y.round() as i32;
        let nz = z.round() as i32;
        if nx != px || ny != py || nz != pz {
            let hw = map.get(nx, ny, nz);
            if hw > 0 {
                return if previous {
                    Some((px, py, pz, hw))
                } else {
                    Some((nx, ny, nz, hw))
                };
            }
            px = nx;
            py = ny;
            pz = nz;
        }
        x += vx / m as f32;
        y += vy / m as f32;
        z += vz / m as f32;
    }
    None
}

/// Resolve AABB against obstacles in `map`. Returns true if vertical collision
/// clamped Y (landed / hit ceiling).
pub fn collide_map(map: &Map, height: i32, x: &mut f32, y: &mut f32, z: &mut f32) -> bool {
    let mut result = false;
    let nx = x.round() as i32;
    let ny = y.round() as i32;
    let nz = z.round() as i32;
    let px = *x - nx as f32;
    let py = *y - ny as f32;
    let pz = *z - nz as f32;
    let pad = 0.25f32;
    for dy in 0..height {
        if px < -pad && is_obstacle(map.get(nx - 1, ny - dy, nz)) {
            *x = nx as f32 - pad;
        }
        if px > pad && is_obstacle(map.get(nx + 1, ny - dy, nz)) {
            *x = nx as f32 + pad;
        }
        if py < -pad && is_obstacle(map.get(nx, ny - dy - 1, nz)) {
            *y = ny as f32 - pad;
            result = true;
        }
        if py > pad && is_obstacle(map.get(nx, ny - dy + 1, nz)) {
            *y = ny as f32 + pad;
            result = true;
        }
        if pz < -pad && is_obstacle(map.get(nx, ny - dy, nz - 1)) {
            *z = nz as f32 - pad;
        }
        if pz > pad && is_obstacle(map.get(nx, ny - dy, nz + 1)) {
            *z = nz as f32 + pad;
        }
    }
    result
}

pub fn player_intersects_block(
    height: i32,
    x: f32,
    y: f32,
    z: f32,
    hx: i32,
    hy: i32,
    hz: i32,
) -> bool {
    let nx = x.round() as i32;
    let ny = y.round() as i32;
    let nz = z.round() as i32;
    for i in 0..height {
        if nx == hx && ny - i == hy && nz == hz {
            return true;
        }
    }
    false
}
