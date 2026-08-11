//! Port of `src/map.c` / `src/map.h`.
//!
//! Open-addressing hash map keyed by (x, y, z), storing a signed block id `w`.
//! Coordinates are stored relative to an origin (dx, dy, dz) and packed into a
//! single u32 exactly like the C `union { unsigned int value; struct {u8 x, u8
//! y, u8 z, i8 w} e; }` on a little-endian target. `value == 0` marks an empty
//! slot, so a slot is empty iff x=y=z=w=0.
//!
//! Integer overflow in the hash is intentional; it matches gcc's wrapping
//! signed-int arithmetic, so `wrapping_*` is used throughout.

/// Signed-int hash matching `hash_int` in the C, including overflow behaviour.
fn hash_int(mut key: i32) -> i32 {
    key = (!key).wrapping_add(key.wrapping_shl(15));
    key ^= key >> 12; // arithmetic shift, as gcc does for signed int
    key = key.wrapping_add(key.wrapping_shl(2));
    key ^= key >> 4;
    key = key.wrapping_mul(2057);
    key ^= key >> 16;
    key
}

fn hash(x: i32, y: i32, z: i32) -> i32 {
    hash_int(x) ^ hash_int(y) ^ hash_int(z)
}

/// One slot. `value == 0` means empty.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
struct Entry {
    x: u8,
    y: u8,
    z: u8,
    w: i8,
}

impl Entry {
    #[inline]
    fn is_empty(&self) -> bool {
        self.x == 0 && self.y == 0 && self.z == 0 && self.w == 0
    }
}

pub struct Map {
    pub dx: i32,
    pub dy: i32,
    pub dz: i32,
    pub mask: u32,
    pub size: u32,
    data: Vec<Entry>,
}

impl Map {
    pub fn new(dx: i32, dy: i32, dz: i32, mask: u32) -> Self {
        Map {
            dx,
            dy,
            dz,
            mask,
            size: 0,
            data: vec![Entry::default(); (mask as usize) + 1],
        }
    }

    /// Sets block `w` at (x, y, z). Returns true if the map changed.
    /// `w == 0` at a new key is a no-op (empty is not stored), matching C.
    pub fn set(&mut self, x: i32, y: i32, z: i32, w: i32) -> bool {
        let mut index = (hash(x, y, z) as u32) & self.mask;
        let rx = x - self.dx;
        let ry = y - self.dy;
        let rz = z - self.dz;

        // Coordinates are truncated to the packed width exactly as the C does
        // on assignment to unsigned/signed char.
        let key_x = rx as u8;
        let key_y = ry as u8;
        let key_z = rz as u8;
        let key_w = w as i8;

        let mut overwrite = false;
        loop {
            let entry = self.data[index as usize];
            if entry.is_empty() {
                break;
            }
            // Compare against the untruncated relative coord, as C promotes the
            // stored unsigned char back to int before comparing.
            if entry.x as i32 == rx && entry.y as i32 == ry && entry.z as i32 == rz {
                overwrite = true;
                break;
            }
            index = (index + 1) & self.mask;
        }

        if overwrite {
            if self.data[index as usize].w != key_w {
                self.data[index as usize].w = key_w;
                return true;
            }
        } else if w != 0 {
            self.data[index as usize] = Entry {
                x: key_x,
                y: key_y,
                z: key_z,
                w: key_w,
            };
            self.size += 1;
            if self.size * 2 > self.mask {
                self.grow();
            }
            return true;
        }
        false
    }

    pub fn get(&self, x: i32, y: i32, z: i32) -> i32 {
        let mut index = (hash(x, y, z) as u32) & self.mask;
        let rx = x - self.dx;
        let ry = y - self.dy;
        let rz = z - self.dz;
        if !(0..=255).contains(&rx) {
            return 0;
        }
        if !(0..=255).contains(&ry) {
            return 0;
        }
        if !(0..=255).contains(&rz) {
            return 0;
        }
        loop {
            let entry = self.data[index as usize];
            if entry.is_empty() {
                return 0;
            }
            if entry.x as i32 == rx && entry.y as i32 == ry && entry.z as i32 == rz {
                return entry.w as i32;
            }
            index = (index + 1) & self.mask;
        }
    }

    fn grow(&mut self) {
        let new_mask = (self.mask << 1) | 1;
        let mut new_map = Map::new(self.dx, self.dy, self.dz, new_mask);
        // Iterate slots in index order, exactly like MAP_FOR_EACH.
        for i in 0..=self.mask {
            let entry = self.data[i as usize];
            if entry.is_empty() {
                continue;
            }
            let ex = entry.x as i32 + self.dx;
            let ey = entry.y as i32 + self.dy;
            let ez = entry.z as i32 + self.dz;
            let ew = entry.w as i32;
            new_map.set(ex, ey, ez, ew);
        }
        self.mask = new_map.mask;
        self.size = new_map.size;
        self.data = new_map.data;
    }

    /// Visits every non-empty entry as absolute (x, y, z, w), in slot order.
    pub fn for_each<F: FnMut(i32, i32, i32, i32)>(&self, mut f: F) {
        for i in 0..=self.mask {
            let entry = self.data[i as usize];
            if entry.is_empty() {
                continue;
            }
            f(
                entry.x as i32 + self.dx,
                entry.y as i32 + self.dy,
                entry.z as i32 + self.dz,
                entry.w as i32,
            );
        }
    }
}
