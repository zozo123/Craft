//! Behavioural tests for the hash map. There is no C golden for the map, so
//! these encode the observable contract from map.c: set/get, overwrite,
//! delete-by-zero semantics, automatic growth, and origin offsets.

use craft_core::map::Map;

#[test]
fn set_get_roundtrip() {
    let mut m = Map::new(0, 0, 0, 0xf);
    assert_eq!(m.get(1, 2, 3), 0);
    assert!(m.set(1, 2, 3, 5));
    assert_eq!(m.get(1, 2, 3), 5);
    assert_eq!(m.size, 1);
}

#[test]
fn setting_zero_at_new_key_is_noop() {
    let mut m = Map::new(0, 0, 0, 0xf);
    assert!(!m.set(4, 5, 6, 0));
    assert_eq!(m.get(4, 5, 6), 0);
    assert_eq!(m.size, 0);
}

#[test]
fn overwrite_changes_value_and_reports_change() {
    let mut m = Map::new(0, 0, 0, 0xf);
    assert!(m.set(1, 1, 1, 2));
    assert!(!m.set(1, 1, 1, 2)); // same value -> no change
    assert!(m.set(1, 1, 1, 3)); // different -> change
    assert_eq!(m.get(1, 1, 1), 3);
    assert_eq!(m.size, 1);
}

#[test]
fn negative_w_survives_roundtrip() {
    // Border blocks are stored with a negated id; the slot's w is signed.
    let mut m = Map::new(0, 0, 0, 0xf);
    assert!(m.set(2, 3, 4, -16));
    assert_eq!(m.get(2, 3, 4), -16);
}

#[test]
fn grows_and_preserves_all_entries() {
    let mut m = Map::new(0, 0, 0, 0x3); // tiny, forces several growths
    let n = 200;
    for i in 0..n {
        let x = i % 16;
        let y = (i / 16) % 16;
        let z = i / 256;
        assert!(m.set(x, y, z, (i % 63) + 1));
    }
    assert_eq!(m.size, n as u32);
    for i in 0..n {
        let x = i % 16;
        let y = (i / 16) % 16;
        let z = i / 256;
        assert_eq!(m.get(x, y, z), (i % 63) + 1, "lost entry {i} after grow");
    }
    assert!(m.mask >= 0x3);
}

#[test]
fn origin_offset_applies_to_keys() {
    let mut m = Map::new(-5, -5, -5, 0xf);
    assert!(m.set(0, 0, 0, 7)); // relative (5,5,5)
    assert_eq!(m.get(0, 0, 0), 7);
    // Out-of-window on the relative side returns 0 in get().
    assert_eq!(m.get(1000, 0, 0), 0);
}

#[test]
fn for_each_visits_all_absolute_coords() {
    let mut m = Map::new(10, 20, 30, 0xf);
    let inserts = [(10, 20, 30, 1), (11, 21, 31, 2), (12, 22, 32, 3)];
    for &(x, y, z, w) in &inserts {
        m.set(x, y, z, w);
    }
    let mut seen = Vec::new();
    m.for_each(|x, y, z, w| seen.push((x, y, z, w)));
    seen.sort();
    let mut expect = inserts.to_vec();
    expect.sort();
    assert_eq!(seen, expect);
}
