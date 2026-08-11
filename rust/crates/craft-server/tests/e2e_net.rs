//! Wave K/L: TCP handshake → chunk stream → block round-trip.

use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::time::Duration;

use craft_protocol::Packet;
use craft_server::serve;
use tempfile::tempdir;
use tokio::net::TcpListener;
use tokio::runtime::Runtime;

fn read_until(reader: &mut BufReader<TcpStream>, pred: impl Fn(&Packet) -> bool) -> Packet {
    let mut line = String::new();
    for _ in 0..5000 {
        line.clear();
        reader.read_line(&mut line).expect("read");
        assert!(!line.is_empty(), "eof before match");
        let p = Packet::parse_line(&line).unwrap_or_else(|e| panic!("parse {line:?}: {e}"));
        if pred(&p) {
            return p;
        }
    }
    panic!("too many lines without match");
}

#[test]
fn server_handshake_chunk_block_roundtrip() {
    let dir = tempdir().unwrap();
    let db = dir.path().join("e2e.db");
    let db_s = db.to_str().unwrap().to_string();

    let rt = Runtime::new().unwrap();
    let listener = rt.block_on(TcpListener::bind("127.0.0.1:0")).unwrap();
    let addr = listener.local_addr().unwrap();
    rt.spawn(async move {
        let _ = serve(listener, &db_s).await;
    });

    std::thread::sleep(Duration::from_millis(50));
    let mut stream = TcpStream::connect(addr).expect("connect");
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .unwrap();

    let mut reader = BufReader::new(stream.try_clone().unwrap());

    // U + E + welcome
    let you = read_until(&mut reader, |p| matches!(p, Packet::You(_)));
    assert!(matches!(you, Packet::You(1)));
    let _ = read_until(&mut reader, |p| matches!(p, Packet::Time { .. }));

    write!(
        stream,
        "{}",
        Packet::Version(1).encode()
            + &Packet::Authenticate {
                username: "tester".into(),
                token: "-".into(),
            }
            .encode()
            + &Packet::Chunk { p: 0, q: 0, key: 0 }.encode()
    )
    .unwrap();
    stream.flush().unwrap();

    let mut saw_block = false;
    let mut saw_key = false;
    let mut block_count = 0u32;
    let mut line = String::new();
    for _ in 0..100_000 {
        line.clear();
        reader.read_line(&mut line).expect("chunk line");
        let p = Packet::parse_line(&line).expect("parse chunk");
        match p {
            Packet::BlockChunk { .. } => {
                saw_block = true;
                block_count += 1;
            }
            Packet::Key(_) => saw_key = true,
            Packet::Unknown(ref s) if s.starts_with('K') => saw_key = true,
            _ => {}
        }
        if saw_key {
            break;
        }
    }
    assert!(saw_block, "expected BlockChunk for chunk 0,0");
    assert!(
        block_count > 2000,
        "expected full chunk stream, got {block_count}"
    );
    assert!(saw_key, "expected K key after chunk");

    write!(
        stream,
        "{}",
        Packet::Block {
            x: 1,
            y: 10,
            z: 1,
            w: 1
        }
        .encode()
    )
    .unwrap();
    stream.flush().unwrap();

    // Own echo may not arrive (broadcast excludes sender depending on timing);
    // reconnect and request chunk — block must persist in DB.
    drop(reader);
    drop(stream);

    let mut stream2 = TcpStream::connect(addr).expect("reconnect");
    stream2
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    let mut reader2 = BufReader::new(stream2.try_clone().unwrap());
    let _ = read_until(&mut reader2, |p| matches!(p, Packet::You(_)));
    write!(stream2, "{}", Packet::Chunk { p: 0, q: 0, key: 0 }.encode()).unwrap();
    stream2.flush().unwrap();

    let mut found = false;
    for _ in 0..100_000 {
        line.clear();
        reader2.read_line(&mut line).expect("line2");
        match Packet::parse_line(&line).unwrap() {
            Packet::BlockChunk {
                x: 1,
                y: 10,
                z: 1,
                w: 1,
                ..
            } => {
                found = true;
                break;
            }
            Packet::Key(_) => break,
            Packet::Unknown(ref s) if s.starts_with('K') => break,
            _ => {}
        }
    }
    assert!(found, "persisted block missing from chunk reload");
}
