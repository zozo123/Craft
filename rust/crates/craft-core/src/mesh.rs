//! Single-chunk meshing with face culling (flat AO/light for Wave C).
//! Full occlusion/light propagation lands in Wave G.

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
}

/// Fill a Map for chunk (p, q) the same way Craft does (including pad/border).
pub fn fill_chunk_map(p: i32, q: i32) -> Map {
    let pad = 1;
    // Match Craft's typical origin: chunk corner minus pad.
    let dx = p * CHUNK_SIZE - pad;
    let dy = 0;
    let dz = q * CHUNK_SIZE - pad;
    let mut map = Map::new(dx, dy, dz, 0xfff);
    create_world(p, q, |x, y, z, w| {
        map.set(x, y, z, w);
    });
    map
}

fn opaque_at(map: &Map, x: i32, y: i32, z: i32) -> bool {
    let w = map.get(x, y, z);
    w != 0 && !is_transparent(w)
}

/// Mesh one chunk with face culling. AO/light are zero (parity for geometry
/// counts and positions; lighting comes later).
pub fn mesh_chunk(p: i32, q: i32) -> (Vec<f32>, MeshStats) {
    let map = fill_chunk_map(p, q);
    let mut faces = 0u32;
    let mut blocks = 0u32;
    let mut miny = 256i32;
    let mut maxy = 0i32;

    // First pass: count faces.
    let mut entries = Vec::new();
    map.for_each(|ex, ey, ez, ew| {
        if ew <= 0 {
            return;
        }
        // Only mesh blocks owned by this chunk (non-negative ids that fall inside).
        let x0 = p * CHUNK_SIZE;
        let z0 = q * CHUNK_SIZE;
        if ex < x0 || ez < z0 || ex >= x0 + CHUNK_SIZE || ez >= z0 + CHUNK_SIZE {
            return;
        }
        blocks += 1;
        let f1 = !opaque_at(&map, ex - 1, ey, ez);
        let f2 = !opaque_at(&map, ex + 1, ey, ez);
        let f3 = !opaque_at(&map, ex, ey + 1, ez);
        let f4 = !opaque_at(&map, ex, ey - 1, ez) && ey > 0;
        let f5 = !opaque_at(&map, ex, ey, ez - 1);
        let f6 = !opaque_at(&map, ex, ey, ez + 1);
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
    let ao = [[0.0f32; 4]; 6];
    let light = [[0.0f32; 4]; 6];

    for (ex, ey, ez, ew, f1, f2, f3, f4, f5, f6, total) in entries {
        if is_plant(ew) {
            let rotation = simplex2(ex as f32, ez as f32, 4, 0.5, 2.0) * 360.0;
            make_plant(
                &mut data[offset..],
                0.0,
                0.0,
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
    };
    data.truncate(offset);
    (data, stats)
}
