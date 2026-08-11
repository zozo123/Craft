//! Chunk meshing with face culling + Craft-faithful AO / height shade.
//! Torch light flood-fill is optional (off when no light map) — Wave G.

use crate::config::CHUNK_SIZE;
use crate::cube::{make_cube, make_plant};
use crate::item::{is_plant, is_transparent};
use crate::map::Map;
use crate::noise::simplex2;
use crate::world::create_world;

#[derive(Debug, Clone)]
pub struct MeshStats {
    pub p: i32,
    pub q: i32,
    pub blocks: u32,
    pub faces: u32,
    pub floats: usize,
    pub miny: i32,
    pub maxy: i32,
    /// Sum of AO floats written into vertex data (should be >0 with AO on).
    pub ao_sum: f64,
    /// Sum of light floats written into vertex data.
    pub light_sum: f64,
}

/// Fill a Map for chunk (p, q) the same way Craft does (including pad/border).
pub fn fill_chunk_map(p: i32, q: i32) -> Map {
    let pad = 1;
    let dx = p * CHUNK_SIZE - pad;
    let dy = 0;
    let dz = q * CHUNK_SIZE - pad;
    let mut map = Map::new(dx, dy, dz, 0xfff);
    create_world(p, q, |x, y, z, w| {
        map.set(x, y, z, w);
    });
    map
}

/// Fill a 3×3 neighborhood of chunks into one map (needed for edge AO).
pub fn fill_neighborhood_map(p: i32, q: i32) -> Map {
    let pad = 1;
    let dx = (p - 1) * CHUNK_SIZE - pad;
    let dy = 0;
    let dz = (q - 1) * CHUNK_SIZE - pad;
    let mut map = Map::new(dx, dy, dz, 0x7fff);
    for dp in -1..=1 {
        for dq in -1..=1 {
            create_world(p + dp, q + dq, |x, y, z, w| {
                map.set(x, y, z, w);
            });
        }
    }
    map
}

fn opaque_at(map: &Map, x: i32, y: i32, z: i32) -> bool {
    let w = map.get(x, y, z);
    w != 0 && !is_transparent(w)
}

/// Port of `occlusion()` in `src/main.c`.
fn occlusion(
    neighbors: &[u8; 27],
    lights: &[u8; 27],
    shades: &[f32; 27],
    ao: &mut [[f32; 4]; 6],
    light: &mut [[f32; 4]; 6],
) {
    #[rustfmt::skip]
    const LOOKUP3: [[[usize; 3]; 4]; 6] = [
        [[0, 1, 3], [2, 1, 5], [6, 3, 7], [8, 5, 7]],
        [[18, 19, 21], [20, 19, 23], [24, 21, 25], [26, 23, 25]],
        [[6, 7, 15], [8, 7, 17], [24, 15, 25], [26, 17, 25]],
        [[0, 1, 9], [2, 1, 11], [18, 9, 19], [20, 11, 19]],
        [[0, 3, 9], [6, 3, 15], [18, 9, 21], [24, 15, 21]],
        [[2, 5, 11], [8, 5, 17], [20, 11, 23], [26, 17, 23]],
    ];
    #[rustfmt::skip]
    const LOOKUP4: [[[usize; 4]; 4]; 6] = [
        [[0, 1, 3, 4], [1, 2, 4, 5], [3, 4, 6, 7], [4, 5, 7, 8]],
        [[18, 19, 21, 22], [19, 20, 22, 23], [21, 22, 24, 25], [22, 23, 25, 26]],
        [[6, 7, 15, 16], [7, 8, 16, 17], [15, 16, 24, 25], [16, 17, 25, 26]],
        [[0, 1, 9, 10], [1, 2, 10, 11], [9, 10, 18, 19], [10, 11, 19, 20]],
        [[0, 3, 9, 12], [3, 6, 12, 15], [9, 12, 18, 21], [12, 15, 21, 24]],
        [[2, 5, 11, 14], [5, 8, 14, 17], [11, 14, 20, 23], [14, 17, 23, 26]],
    ];
    const CURVE: [f32; 4] = [0.0, 0.25, 0.5, 0.75];

    for i in 0..6 {
        for j in 0..4 {
            let corner = neighbors[LOOKUP3[i][j][0]] != 0;
            let side1 = neighbors[LOOKUP3[i][j][1]] != 0;
            let side2 = neighbors[LOOKUP3[i][j][2]] != 0;
            let value = if side1 && side2 {
                3
            } else {
                u8::from(corner) + u8::from(side1) + u8::from(side2)
            };
            let mut shade_sum = 0.0f32;
            let mut light_sum = 0.0f32;
            let is_light = lights[13] == 15;
            for k in 0..4 {
                shade_sum += shades[LOOKUP4[i][j][k]];
                light_sum += f32::from(lights[LOOKUP4[i][j][k]]);
            }
            if is_light {
                light_sum = 15.0 * 4.0 * 10.0;
            }
            let total = CURVE[value as usize] + shade_sum / 4.0;
            ao[i][j] = total.min(1.0);
            light[i][j] = light_sum / 15.0 / 4.0;
        }
    }
}

/// Mesh a generated chunk with its 3×3 neighborhood.
pub fn mesh_chunk(p: i32, q: i32) -> (Vec<f32>, MeshStats) {
    let map = fill_neighborhood_map(p, q);
    mesh_map(p, q, &map)
}

/// Mesh chunk `(p, q)` from an externally supplied block map.
///
/// This is used by the online client after receiving authoritative blocks.
pub fn mesh_map(p: i32, q: i32, map: &Map) -> (Vec<f32>, MeshStats) {
    let mut faces = 0u32;
    let mut blocks = 0u32;
    let mut miny = 256i32;
    let mut maxy = 0i32;

    // highest[y] per column for shade — keyed by (x,z) world coords via map scan.
    let mut highest: std::collections::HashMap<(i32, i32), i32> = std::collections::HashMap::new();
    map.for_each(|ex, ey, ez, ew| {
        if ew != 0 && !is_transparent(ew) {
            let e = highest.entry((ex, ez)).or_insert(0);
            *e = (*e).max(ey);
        }
    });

    let mut entries = Vec::new();
    let x0 = p * CHUNK_SIZE;
    let z0 = q * CHUNK_SIZE;
    map.for_each(|ex, ey, ez, ew| {
        if ew <= 0 {
            return;
        }
        if ex < x0 || ez < z0 || ex >= x0 + CHUNK_SIZE || ez >= z0 + CHUNK_SIZE {
            return;
        }
        blocks += 1;
        let f1 = !opaque_at(map, ex - 1, ey, ez);
        let f2 = !opaque_at(map, ex + 1, ey, ez);
        let f3 = !opaque_at(map, ex, ey + 1, ez);
        let f4 = !opaque_at(map, ex, ey - 1, ez) && ey > 0;
        let f5 = !opaque_at(map, ex, ey, ez - 1);
        let f6 = !opaque_at(map, ex, ey, ez + 1);
        let mut total = i32::from(f1)
            + i32::from(f2)
            + i32::from(f3)
            + i32::from(f4)
            + i32::from(f5)
            + i32::from(f6);
        if total == 0 {
            return;
        }
        if is_plant(ew) {
            total = 4;
        }
        miny = miny.min(ey);
        maxy = maxy.max(ey);
        faces += total as u32;
        entries.push((ex, ey, ez, ew, f1, f2, f3, f4, f5, f6, total));
    });

    let mut data = vec![0.0f32; faces as usize * 60];
    let mut offset = 0usize;
    let mut ao_sum = 0.0f64;
    let mut light_sum = 0.0f64;

    for (ex, ey, ez, ew, f1, f2, f3, f4, f5, f6, total) in entries {
        let mut neighbors = [0u8; 27];
        let lights = [0u8; 27];
        let mut shades = [0.0f32; 27];
        let mut index = 0;
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    let nx = ex + dx;
                    let ny = ey + dy;
                    let nz = ez + dz;
                    neighbors[index] = u8::from(opaque_at(map, nx, ny, nz));
                    shades[index] = 0.0;
                    let hi = highest.get(&(nx, nz)).copied().unwrap_or(0);
                    if ny <= hi {
                        for oy in 0..8 {
                            if opaque_at(map, nx, ny + oy, nz) {
                                shades[index] = 1.0 - oy as f32 * 0.125;
                                break;
                            }
                        }
                    }
                    index += 1;
                }
            }
        }
        let mut ao = [[0.0f32; 4]; 6];
        let mut light = [[0.0f32; 4]; 6];
        occlusion(&neighbors, &lights, &shades, &mut ao, &mut light);

        if is_plant(ew) {
            let mut min_ao = 1.0f32;
            let mut max_light = 0.0f32;
            for a in &ao {
                for &b in a {
                    min_ao = min_ao.min(b);
                }
            }
            for row in &light {
                for &v in row {
                    max_light = max_light.max(v);
                }
            }
            let rotation = simplex2(ex as f32, ez as f32, 4, 0.5, 2.0) * 360.0;
            make_plant(
                &mut data[offset..],
                min_ao,
                max_light,
                ex as f32,
                ey as f32,
                ez as f32,
                0.5,
                ew,
                rotation,
            );
        } else {
            make_cube(
                &mut data[offset..],
                &ao,
                &light,
                i32::from(f1),
                i32::from(f2),
                i32::from(f3),
                i32::from(f4),
                i32::from(f5),
                i32::from(f6),
                ex as f32,
                ey as f32,
                ez as f32,
                0.5,
                ew,
            );
        }
        let slice = &data[offset..offset + (total as usize) * 60];
        for chunk in slice.chunks_exact(10) {
            ao_sum += f64::from(chunk[8]);
            light_sum += f64::from(chunk[9]);
        }
        offset += (total as usize) * 60;
    }

    let stats = MeshStats {
        p,
        q,
        blocks,
        faces,
        floats: offset,
        miny: if faces == 0 { 0 } else { miny },
        maxy,
        ao_sum,
        light_sum,
    };
    data.truncate(offset);
    (data, stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ao_nontrivial_on_chunk_0() {
        let (_data, s) = mesh_chunk(0, 0);
        assert!(s.faces > 0);
        assert!(
            s.ao_sum > 0.0,
            "AO must shade some vertices, got {}",
            s.ao_sum
        );
    }

    #[test]
    fn occlusion_corner_case() {
        let mut neighbors = [0u8; 27];
        // Fully surrounded center → strong AO on exposed faces still depends on neighbors.
        neighbors[13] = 1;
        neighbors[4] = 1; // above-ish in dx/dy/dz order — just exercise path
        let lights = [0u8; 27];
        let shades = [0.0f32; 27];
        let mut ao = [[0.0f32; 4]; 6];
        let mut light = [[0.0f32; 4]; 6];
        occlusion(&neighbors, &lights, &shades, &mut ao, &mut light);
        // At least some AO slots may be zero; function must not panic and stay in range.
        for face in &ao {
            for &v in face {
                assert!((0.0..=1.0).contains(&v));
            }
        }
    }
}
