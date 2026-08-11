//! Matrix parity vs C `oracle/dump_matrix.c` goldens.

mod common;

use craft_core::matrix::*;

const ABS_TOL: f32 = 1e-5;

fn close(a: f32, b: f32) -> bool {
    if a == b {
        return true;
    }
    let d = (a - b).abs();
    if d <= ABS_TOL {
        return true;
    }
    // Near-zero entries (seen across macOS/Linux libm) compare absolute-only.
    let scale = a.abs().max(b.abs());
    if scale < 1e-6 {
        return d <= ABS_TOL;
    }
    d / scale <= ABS_TOL
}

fn assert_mat(tag: &str, got: &[f32; 16], expect: &[f32]) {
    assert_eq!(expect.len(), 16, "{tag}: expected 16 floats");
    for i in 0..16 {
        assert!(
            close(got[i], expect[i]),
            "{tag}[{i}]: got {:e} expect {:e} (|d|={:e})",
            got[i],
            expect[i],
            (got[i] - expect[i]).abs()
        );
    }
}

#[test]
fn matrix_matches_c() {
    let contents = common::read_golden("matrix.tsv");
    let mut checked = 0usize;

    for r in common::rows(&contents) {
        let tag = r[0];
        if tag.starts_with("plane_") {
            // Planes validated against the matrix_3d dump below by recompute.
            continue;
        }
        if tag == "normalize" {
            let mut x = 3.0f32;
            let mut y = 4.0f32;
            let mut z = 12.0f32;
            normalize(&mut x, &mut y, &mut z);
            let ex = common::parse_hex_f32(r[1]);
            let ey = common::parse_hex_f32(r[2]);
            let ez = common::parse_hex_f32(r[3]);
            assert!(close(x, ex) && close(y, ey) && close(z, ez), "normalize");
            checked += 1;
            continue;
        }

        let expect: Vec<f32> = r[1..].iter().map(|s| common::parse_hex_f32(s)).collect();
        let mut m = [0.0f32; 16];
        match tag {
            "identity" => mat_identity(&mut m),
            "translate" => mat_translate(&mut m, 1.5, -2.25, 3.75),
            // Exact angle literal from oracle/dump_matrix.c (not std FRAC_PI_4).
            #[allow(clippy::approx_constant, clippy::excessive_precision)]
            "rotate_y_45" => mat_rotate(&mut m, 0.0, 1.0, 0.0, 0.785_398_163_4),
            "rotate_x_0p5" => mat_rotate(&mut m, 1.0, 0.0, 0.0, 0.5),
            "multiply" => {
                let mut a = [0.0f32; 16];
                let mut b = [0.0f32; 16];
                mat_rotate(&mut a, 0.0, 1.0, 0.0, 0.3);
                mat_rotate(&mut b, 1.0, 0.0, 0.0, 0.2);
                mat_multiply(&mut m, a, b);
            }
            "frustum" => mat_frustum(&mut m, -1.0, 1.0, -0.75, 0.75, 0.125, 512.0),
            "perspective" => mat_perspective(&mut m, 65.0, 1.333_333_3, 0.125, 512.0),
            "ortho" => mat_ortho(&mut m, -10.0, 10.0, -7.5, 7.5, -1.0, 1.0),
            "matrix_2d" => set_matrix_2d(&mut m, 1024, 768),
            "matrix_item" => set_matrix_item(&mut m, 1024, 768, 2),
            "matrix_3d" => set_matrix_3d(&mut m, 1024, 768, 1.0, 18.0, 3.0, 0.5, 0.25, 65.0, 0, 10),
            "matrix_3d_ortho" => {
                set_matrix_3d(&mut m, 1024, 768, 1.0, 18.0, 3.0, 0.5, 0.25, 65.0, 64, 10)
            }
            "matrix_3d_alt" => set_matrix_3d(
                &mut m, 800, 600, -12.5, 33.25, 7.125, -1.75, 0.6, 45.0, 0, 24,
            ),
            other => panic!("unexpected matrix row: {other}"),
        }
        assert_mat(tag, &m, &expect);
        checked += 1;
    }

    // Frustum planes from the same matrix_3d call as the C dumper.
    let mut m = [0.0f32; 16];
    set_matrix_3d(&mut m, 1024, 768, 1.0, 18.0, 3.0, 0.5, 0.25, 65.0, 0, 10);
    let mut planes = [[0.0f32; 4]; 6];
    frustum_planes(&mut planes, 10, &m);
    for r in common::rows(&contents) {
        if !r[0].starts_with("plane_") {
            continue;
        }
        let i: usize = r[0].strip_prefix("plane_").unwrap().parse().unwrap();
        for k in 0..4 {
            let expect = common::parse_hex_f32(r[1 + k]);
            assert!(
                close(planes[i][k], expect),
                "plane_{i}[{k}]: got {:e} expect {:e}",
                planes[i][k],
                expect
            );
        }
        checked += 1;
    }

    assert!(checked >= 14, "too few matrix checks: {checked}");
}
