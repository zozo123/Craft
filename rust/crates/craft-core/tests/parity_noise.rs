//! Parity for simplex noise. When goldens are generated on the same machine
//! that runs the test (the CI case), results are bit-identical; a small
//! tolerance is allowed so the test still holds when comparing across
//! platforms (observed worst-case cross-platform drift ~1.5e-7 absolute).

mod common;

use craft_core::noise::{simplex2, simplex3};

const ABS_TOL: f32 = 1e-5;

fn close(a: f32, b: f32) -> bool {
    a == b || (a - b).abs() <= ABS_TOL
}

#[test]
fn simplex_matches_c() {
    let contents = common::read_golden("noise.tsv");
    let mut n2 = 0;
    let mut n3 = 0;

    for r in common::rows(&contents) {
        match r[0] {
            "simplex2" => {
                // simplex2 <tag> x y octaves persistence lacunarity value
                let x = common::parse_hex_f32(r[2]);
                let y = common::parse_hex_f32(r[3]);
                let octaves: i32 = r[4].parse().unwrap();
                let persistence = common::parse_hex_f32(r[5]);
                let lacunarity = common::parse_hex_f32(r[6]);
                let expect = common::parse_hex_f32(r[7]);
                let got = simplex2(x, y, octaves, persistence, lacunarity);
                assert!(
                    close(got, expect),
                    "simplex2 {} : got {got:e} expect {expect:e} (|d|={:e})",
                    r[1],
                    (got - expect).abs()
                );
                n2 += 1;
            }
            "simplex3" => {
                // simplex3 <tag> x y z octaves persistence lacunarity value
                let x = common::parse_hex_f32(r[2]);
                let y = common::parse_hex_f32(r[3]);
                let z = common::parse_hex_f32(r[4]);
                let octaves: i32 = r[5].parse().unwrap();
                let persistence = common::parse_hex_f32(r[6]);
                let lacunarity = common::parse_hex_f32(r[7]);
                let expect = common::parse_hex_f32(r[8]);
                let got = simplex3(x, y, z, octaves, persistence, lacunarity);
                assert!(
                    close(got, expect),
                    "simplex3 {} : got {got:e} expect {expect:e} (|d|={:e})",
                    r[1],
                    (got - expect).abs()
                );
                n3 += 1;
            }
            other => panic!("unexpected noise row: {other}"),
        }
    }

    assert!(
        n2 > 0 && n3 > 0,
        "no noise samples checked (n2={n2}, n3={n3})"
    );
}
