//! Product-level multiplayer e2e (Wave K/L), fully headless.
//!
//! Two online clients connect to a real `craft-server`; edits by one are
//! persisted and broadcast to the other, positions propagate, and the received
//! world meshes non-empty. No GPU/window required.

use std::time::Duration;

use craft_client::online::OnlineWorld;
use craft_server::serve;
use tempfile::tempdir;
use tokio::net::TcpListener;
use tokio::runtime::Runtime;

fn spawn_server() -> (Runtime, String) {
    let dir = tempdir().unwrap();
    let db = dir.path().join("mp.db").to_str().unwrap().to_string();
    // Keep the tempdir alive for the whole process.
    std::mem::forget(dir);
    let rt = Runtime::new().unwrap();
    let listener = rt.block_on(TcpListener::bind("127.0.0.1:0")).unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    rt.spawn(async move {
        let _ = serve(listener, &db).await;
    });
    (rt, addr)
}

#[test]
fn two_clients_share_edits_and_positions() {
    let (_rt, addr) = spawn_server();

    let mut a = OnlineWorld::connect(&addr, "alice").expect("A connect");
    a.request_chunk(0, 0).unwrap();
    assert!(
        a.pump_until(
            |w| w.chunks_keyed >= 1 && w.blocks_received > 2000,
            Duration::from_secs(15)
        ),
        "A never received full chunk 0,0 (blocks={}, keyed={})",
        a.blocks_received,
        a.chunks_keyed
    );

    let mut b = OnlineWorld::connect(&addr, "bob").expect("B connect");
    b.request_chunk(0, 0).unwrap();
    assert!(
        b.pump_until(
            |w| w.chunks_keyed >= 1 && w.blocks_received > 2000,
            Duration::from_secs(15)
        ),
        "B never received full chunk 0,0"
    );

    assert_ne!(a.id, b.id, "ids must be distinct");

    // A places a block high in the air (empty), B must observe it.
    let (bx, by, bz, bw) = (5, 80, 5, 1);
    a.edit_block(bx, by, bz, bw).unwrap();
    assert!(
        b.pump_until(|w| w.map.get(bx, by, bz) == bw, Duration::from_secs(5)),
        "B never saw placed block"
    );
    assert_eq!(a.map.get(bx, by, bz), bw, "A local state");

    // A breaks it, B must observe removal.
    a.edit_block(bx, by, bz, 0).unwrap();
    assert!(
        b.pump_until(|w| w.map.get(bx, by, bz) == 0, Duration::from_secs(5)),
        "B never saw block removed"
    );

    // Position propagation A -> B.
    a.x = 3.25;
    a.z = -1.5;
    a.send_position().unwrap();
    assert!(
        b.pump_until(|w| w.players.contains_key(&a.id), Duration::from_secs(5)),
        "B never saw A's position"
    );
    let ap = b.players.get(&a.id).copied().unwrap();
    assert!((ap.x - 3.25).abs() < 0.01 && (ap.z + 1.5).abs() < 0.01);

    // Received world meshes non-empty with AO active.
    let (data, stats) = b.mesh(0, 0);
    assert_eq!(data.len(), stats.floats);
    assert!(stats.faces > 0, "meshed online world empty");
    assert!(stats.ao_sum > 0.0, "AO inactive on online world");
}

#[test]
fn edit_persists_across_reconnect() {
    let (_rt, addr) = spawn_server();

    let (bx, by, bz, bw) = (7, 78, 3, 6);
    {
        let mut a = OnlineWorld::connect(&addr, "writer").expect("connect");
        a.request_chunk(0, 0).unwrap();
        a.pump_until(|w| w.chunks_keyed >= 1, Duration::from_secs(5));
        a.edit_block(bx, by, bz, bw).unwrap();
        // Give the server a moment to persist.
        a.pump_until(|w| w.map.get(bx, by, bz) == bw, Duration::from_secs(2));
        std::thread::sleep(Duration::from_millis(100));
    }

    let mut c = OnlineWorld::connect(&addr, "reader").expect("reconnect");
    c.request_chunk(0, 0).unwrap();
    assert!(
        c.pump_until(|w| w.map.get(bx, by, bz) == bw, Duration::from_secs(5)),
        "persisted edit missing after reconnect"
    );
}
