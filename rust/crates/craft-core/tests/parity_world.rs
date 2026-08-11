//! Exact parity for world generation. Every emitted (x, y, z, w) must match
//! the C, in the same order (order matters: later writes overwrite earlier
//! ones, e.g. tree trunks over leaves).

mod common;

use craft_core::world::create_world;

fn check_chunk(p: i32, q: i32) {
    let name = format!("world_{p}_{q}.tsv");
    let contents = common::read_golden(&name);

    let expected: Vec<(i32, i32, i32, i32)> = common::rows(&contents)
        .into_iter()
        .map(|r| {
            (
                r[0].parse().unwrap(),
                r[1].parse().unwrap(),
                r[2].parse().unwrap(),
                r[3].parse().unwrap(),
            )
        })
        .collect();

    let mut got = Vec::with_capacity(expected.len());
    create_world(p, q, |x, y, z, w| got.push((x, y, z, w)));

    assert_eq!(
        got.len(),
        expected.len(),
        "chunk ({p},{q}) emission count: got {} expect {}",
        got.len(),
        expected.len()
    );

    for (i, (g, e)) in got.iter().zip(expected.iter()).enumerate() {
        assert_eq!(g, e, "chunk ({p},{q}) emission {i}: got {g:?} expect {e:?}");
    }
}

#[test]
fn worldgen_matches_c() {
    // Must match the WORLD_CHUNKS list in oracle/Makefile.
    for (p, q) in [(0, 0), (1, 0), (0, 1), (-1, -1), (3, -7), (12, 34)] {
        check_chunk(p, q);
    }
}
