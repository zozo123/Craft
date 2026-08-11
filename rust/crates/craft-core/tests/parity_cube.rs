//! Cube/plant/player/sphere parity vs C `oracle/dump_cube.c` goldens.

mod common;

use craft_core::cube::*;

const ABS_TOL: f32 = 1e-5;

fn close(a: f32, b: f32) -> bool {
    if a == b {
        return true;
    }
    let d = (a - b).abs();
    if d <= ABS_TOL {
        return true;
    }
    let scale = a.abs().max(b.abs());
    if scale < 1e-6 {
        return d <= ABS_TOL;
    }
    d / scale <= ABS_TOL
}

fn assert_buf(tag: &str, got: &[f32], expect: &[f32]) {
    assert_eq!(
        got.len(),
        expect.len(),
        "{tag}: len got {} expect {}",
        got.len(),
        expect.len()
    );
    for (i, (&g, &e)) in got.iter().zip(expect.iter()).enumerate() {
        assert!(
            close(g, e),
            "{tag}[{i}]: got {g:e} expect {e:e} (|d|={:e})",
            (g - e).abs()
        );
    }
}

#[test]
fn cube_matches_c() {
    let contents = common::read_golden("cube.tsv");
    let mut checked = 0usize;

    for r in common::rows(&contents) {
        let tag = r[0];
        let nfloats: usize = r[1].parse().unwrap();
        let expect: Vec<f32> = r[2..2 + nfloats]
            .iter()
            .map(|s| common::parse_hex_f32(s))
            .collect();
        let mut data = vec![0.0f32; 65536];

        match tag {
            "cube_grass_all" => {
                let ao = [[0.0f32; 4]; 6];
                let light = [[0.0f32; 4]; 6];
                make_cube(
                    &mut data, &ao, &light, 1, 1, 1, 1, 1, 1, 0.0, 0.0, 0.0, 0.5, 1,
                );
            }
            "cube_wood_top" => {
                let ao = [[0.0f32; 4]; 6];
                let light = [[0.0f32; 4]; 6];
                make_cube(
                    &mut data, &ao, &light, 0, 0, 1, 0, 0, 0, 2.0, 3.0, 4.0, 0.5, 5,
                );
            }
            "cube_ao_flip" => {
                let mut ao = [[0.0f32; 4]; 6];
                ao[0][0] = 0.4;
                ao[0][3] = 0.4;
                let light = [[0.0f32; 4]; 6];
                make_cube(
                    &mut data, &ao, &light, 1, 0, 0, 0, 0, 0, 0.0, 0.0, 0.0, 0.5, 3,
                );
            }
            "cube_light_varied" => {
                let mut ao = [[0.0f32; 4]; 6];
                ao[0][0] = 0.4;
                ao[0][3] = 0.4;
                let mut light = [[0.0f32; 4]; 6];
                for i in 0..6 {
                    for j in 0..4 {
                        light[i][j] = (i * 4 + j) as f32 / 32.0;
                    }
                }
                make_cube(
                    &mut data, &ao, &light, 1, 1, 1, 1, 1, 1, -1.0, 2.0, -3.0, 0.5, 10,
                );
            }
            "plant_18_rot0" => make_plant(&mut data, 0.0, 0.0, 0.0, 0.0, 0.0, 0.5, 18, 0.0),
            "plant_23_rot45" => make_plant(&mut data, 0.25, 0.5, 1.0, 2.0, 3.0, 0.5, 23, 45.0),
            "player_origin" => make_player(&mut data, 0.0, 0.0, 0.0, 0.0, 0.0),
            "player_posed" => make_player(&mut data, 1.5, 20.0, -3.5, 0.75, -0.25),
            "wireframe" => make_cube_wireframe(&mut data, 0.0, 0.0, 0.0, 0.52),
            "char_A" => make_character(&mut data, 100.0, 50.0, 12.0, 24.0, 'A'),
            "char3d_Z" => make_character_3d(&mut data, 1.0, 2.0, 3.0, 0.5, 2, 'Z'),
            "sphere_d0" => make_sphere(&mut data, 1.0, 0),
            "sphere_d1" => make_sphere(&mut data, 1.0, 1),
            "sphere_d2" => make_sphere(&mut data, 1.0, 2),
            "sphere_d3" => make_sphere(&mut data, 1.0, 3),
            other => panic!("unexpected cube row: {other}"),
        }

        assert_buf(tag, &data[..nfloats], &expect);
        checked += 1;
    }

    assert_eq!(checked, 15, "expected 15 cube golden rows");
}
