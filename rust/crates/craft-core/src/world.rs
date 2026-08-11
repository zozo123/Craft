//! Port of `src/world.c`.
//!
//! Two C conversion rules drive the odd-looking casts below, and dropping
//! either one changes generated terrain:
//!
//! 1. `x * 0.01` multiplies an `int` by a `double` literal, so the product is
//!    computed in double precision and only narrowed to `float` when passed
//!    to `simplex2`. Doing the multiply in `f32` yields different noise.
//! 2. `simplex2(...) > 0.6` promotes the `float` result to `double` before
//!    comparing against the `double` literal. Comparing against `0.6f32`
//!    instead flips the result for values between the two representations,
//!    because `0.6f32` is 0.6000000238 while `0.6f64` is 0.5999999999.

use crate::config::CHUNK_SIZE;
use crate::config::{SHOW_CLOUDS, SHOW_PLANTS, SHOW_TREES};
use crate::noise::{simplex2, simplex3};

/// Generates chunk `(p, q)`, invoking `func(x, y, z, w)` for every block.
///
/// Emission order is significant: later calls overwrite earlier ones, which is
/// how tree trunks replace the leaves written just before them. Blocks outside
/// the chunk proper carry a negated `w` to mark them as neighbour-owned.
pub fn create_world<F>(p: i32, q: i32, mut func: F)
where
    F: FnMut(i32, i32, i32, i32),
{
    let pad = 1;
    for dx in -pad..CHUNK_SIZE + pad {
        for dz in -pad..CHUNK_SIZE + pad {
            let mut flag = 1;
            if dx < 0 || dz < 0 || dx >= CHUNK_SIZE || dz >= CHUNK_SIZE {
                flag = -1;
            }
            let x = p * CHUNK_SIZE + dx;
            let z = q * CHUNK_SIZE + dz;

            let f = simplex2(
                (x as f64 * 0.01) as f32,
                (z as f64 * 0.01) as f32,
                4,
                0.5,
                2.0,
            );
            let g = simplex2(
                (-x as f64 * 0.01) as f32,
                (-z as f64 * 0.01) as f32,
                2,
                0.9,
                2.0,
            );
            let mh = (g * 32.0 + 16.0) as i32;
            let mut h = (f * mh as f32) as i32;
            let mut w = 1;
            let t = 12;
            if h <= t {
                h = t;
                w = 2;
            }

            // sand and grass terrain
            for y in 0..h {
                func(x, y, z, w * flag);
            }

            if w == 1 {
                if SHOW_PLANTS {
                    // grass
                    let grass = simplex2(
                        (-x as f64 * 0.1) as f32,
                        (z as f64 * 0.1) as f32,
                        4,
                        0.8,
                        2.0,
                    );
                    if grass as f64 > 0.6 {
                        func(x, h, z, 17 * flag);
                    }
                    // flowers
                    let flower = simplex2(
                        (x as f64 * 0.05) as f32,
                        (-z as f64 * 0.05) as f32,
                        4,
                        0.8,
                        2.0,
                    );
                    if flower as f64 > 0.7 {
                        let kind = simplex2(
                            (x as f64 * 0.1) as f32,
                            (z as f64 * 0.1) as f32,
                            4,
                            0.8,
                            2.0,
                        );
                        let w = (18.0 + kind * 7.0) as i32;
                        func(x, h, z, w * flag);
                    }
                }

                // trees
                let mut ok = SHOW_TREES;
                if dx - 4 < 0 || dz - 4 < 0 || dx + 4 >= CHUNK_SIZE || dz + 4 >= CHUNK_SIZE {
                    ok = false;
                }
                if ok && simplex2(x as f32, z as f32, 6, 0.5, 2.0) as f64 > 0.84 {
                    for y in h + 3..h + 8 {
                        for ox in -3..=3 {
                            for oz in -3..=3 {
                                let d = (ox * ox) + (oz * oz) + (y - (h + 4)) * (y - (h + 4));
                                if d < 11 {
                                    func(x + ox, y, z + oz, 15);
                                }
                            }
                        }
                    }
                    for y in h..h + 7 {
                        func(x, y, z, 5);
                    }
                }
            }

            // clouds
            if SHOW_CLOUDS {
                for y in 64..72 {
                    let c = simplex3(
                        (x as f64 * 0.01) as f32,
                        (y as f64 * 0.1) as f32,
                        (z as f64 * 0.01) as f32,
                        8,
                        0.5,
                        2.0,
                    );
                    if c as f64 > 0.75 {
                        func(x, y, z, 16 * flag);
                    }
                }
            }
        }
    }
}
