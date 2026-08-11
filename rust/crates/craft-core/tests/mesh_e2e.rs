//! Headless mesh e2e: chunk (0,0) must produce a non-empty stable mesh.

use craft_core::mesh::mesh_chunk;

#[test]
fn mesh_chunk_0_0_nonempty_and_stable() {
    let (data1, s1) = mesh_chunk(0, 0);
    let (data2, s2) = mesh_chunk(0, 0);
    assert!(s1.faces > 0, "expected faces");
    assert!(s1.blocks > 0, "expected blocks");
    assert_eq!(s1.floats, data1.len());
    assert_eq!(s1.faces, s2.faces);
    assert_eq!(s1.blocks, s2.blocks);
    assert_eq!(data1, data2, "mesh must be deterministic");
    // Sanity: terrain height band from worldgen.
    assert!(s1.miny >= 0);
    assert!(s1.maxy < 72); // clouds go to 71
}
