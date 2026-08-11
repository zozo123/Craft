//! Headless physics script gates.

use craft_core::item::GRASS;
use craft_core::map::Map;
use craft_core::mesh::fill_chunk_map;
use craft_core::physics::{
    collide_map, get_motion_vector, get_sight_vector, hit_test_map, player_intersects_block,
};

#[test]
fn motion_forward_grounded_has_no_y() {
    let (vx, vy, vz) = get_motion_vector(false, 1, 0, 0.0, 0.0);
    assert_eq!(vy, 0.0);
    assert!(vx.abs() > 0.0 || vz.abs() > 0.0);
}

#[test]
fn collide_stops_fall_onto_terrain() {
    let map = fill_chunk_map(0, 0);
    // Start above origin column and fall.
    let mut x = 0.0f32;
    let mut y = 40.0f32;
    let mut z = 0.0f32;
    for _ in 0..200 {
        y -= 0.2;
        collide_map(&map, 2, &mut x, &mut y, &mut z);
    }
    // Should rest near the ground, not fall forever.
    assert!(y > 8.0 && y < 35.0, "unexpected y={y}");
}

#[test]
fn hit_test_finds_block_looking_down() {
    let map = fill_chunk_map(0, 0);
    // Straight down (sight_vector with rx=0 also yaws; use an explicit down ray).
    let hit = hit_test_map(&map, 32.0, false, 0.0, 30.0, 0.0, 0.0, -1.0, 0.0);
    assert!(hit.is_some(), "expected terrain hit looking down");
    let (hx, hy, hz, hw) = hit.unwrap();
    assert_eq!((hx, hz), (0, 0));
    assert!(hw > 0 && hy < 30);
    // previous=true yields the empty cell above the hit.
    let prev = hit_test_map(&map, 32.0, true, 0.0, 30.0, 0.0, 0.0, -1.0, 0.0).unwrap();
    assert_eq!(prev.1, hy + 1);
}

#[test]
fn place_blocked_when_intersecting_player() {
    assert!(player_intersects_block(2, 1.2, 10.0, 3.4, 1, 10, 3));
    assert!(!player_intersects_block(2, 1.2, 10.0, 3.4, 5, 10, 5));
}

#[test]
fn break_and_place_mutates_map() {
    let mut map = Map::new(0, 0, 0, 0xff);
    assert!(map.set(2, 5, 2, GRASS));
    assert_eq!(map.get(2, 5, 2), GRASS);
    assert!(map.set(2, 5, 2, 0)); // overwrite with 0 does NOT clear in C!
                                  // In C map_set with w=0 on existing key: overwrite path sets w=0...
                                  // Looking at map_set: if overwrite and entry->e.w != w, set w. So w=0 is stored!
                                  // But EMPTY_ENTRY is value==0, so the slot becomes empty-looking!
                                  // Actually setting w=0 on existing: entry->e.w = 0 makes value possibly non-zero if x,y,z nonzero.
                                  // MapEntry: x,y,z,w - if w=0 but x,y,z non-zero, value != 0 so not EMPTY.
                                  // get returns 0. size unchanged.
    assert_eq!(map.get(2, 5, 2), 0);
}
