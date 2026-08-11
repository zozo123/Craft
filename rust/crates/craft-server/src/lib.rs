//! Tokio Craft multiplayer server (Wave J).

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;

use anyhow::Context;
use craft_core::config::CHUNK_SIZE;
use craft_core::mesh::fill_chunk_map;
use craft_db::Db;
use craft_protocol::Packet;
use log::{info, warn};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, Mutex};

pub struct Shared {
    pub db: Mutex<Db>,
    pub next_id: AtomicI32,
    pub tx: broadcast::Sender<String>,
    pub blocks: Mutex<HashMap<(i32, i32, i32), i32>>,
}

fn chunked(x: f32) -> i32 {
    (x.round() / CHUNK_SIZE as f32).floor() as i32
}

pub async fn handle_client(
    stream: TcpStream,
    addr: SocketAddr,
    shared: Arc<Shared>,
) -> anyhow::Result<()> {
    let id = shared.next_id.fetch_add(1, Ordering::SeqCst);
    info!("client {addr} -> id {id}");
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut rx = shared.tx.subscribe();

    // C client: U,id,x,y,z,rx,ry — spawn at origin above terrain.
    writer
        .write_all(format!("U,{id},0.00,40.00,0.00,0.00,0.00\n").as_bytes())
        .await?;
    // C client: E,elapsed,day_length
    writer
        .write_all(
            Packet::Time {
                day_length: 600,
                time_of_day: 0.0,
            }
            .encode()
            .as_bytes(),
        )
        .await?;
    writer
        .write_all(Packet::Talk("Welcome to Craft!".into()).encode().as_bytes())
        .await?;

    let mut line = String::new();
    loop {
        tokio::select! {
            Ok(msg) = rx.recv() => {
                writer.write_all(msg.as_bytes()).await?;
            }
            result = reader.read_line(&mut line) => {
                let n = result?;
                if n == 0 {
                    break;
                }
                match Packet::parse_line(&line) {
                    Ok(Packet::Version(_)) => {}
                    Ok(Packet::Authenticate { username, .. }) => {
                        info!("auth {username} as {id}");
                        let _ = shared.tx.send(
                            Packet::Talk(format!("{username} joined")).encode(),
                        );
                    }
                    Ok(Packet::Position { x, y, z, rx, ry }) => {
                        let _ = shared.tx.send(
                            Packet::PlayerPosition { id, x, y, z, rx, ry }.encode(),
                        );
                    }
                    Ok(Packet::Block { x, y, z, w }) => {
                        let p = chunked(x as f32);
                        let q = chunked(z as f32);
                        {
                            let db = shared.db.lock().await;
                            db.insert_block(p, q, x, y, z, w)?;
                        }
                        shared.blocks.lock().await.insert((x, y, z), w);
                        let _ = shared.tx.send(
                            Packet::BlockChunk { p, q, x, y, z, w }.encode(),
                        );
                        let _ = shared.tx.send(Packet::Redraw { p, q }.encode());
                    }
                    Ok(Packet::Chunk { p, q, .. }) => {
                        let db_blocks = {
                            let db = shared.db.lock().await;
                            db.load_blocks(p, q).unwrap_or_default()
                        };
                        // Full generated chunk, then overlay DB edits (w==0 = remove).
                        let map = fill_chunk_map(p, q);
                        let mut blocks = Vec::new();
                        map.for_each(|x, y, z, w| {
                            if w > 0 {
                                blocks.push((x, y, z, w));
                            }
                        });
                        for (x, y, z, w) in &db_blocks {
                            if *w == 0 {
                                blocks.retain(|(bx, by, bz, _)| {
                                    !(*bx == *x && *by == *y && *bz == *z)
                                });
                            } else if let Some(slot) = blocks
                                .iter_mut()
                                .find(|(bx, by, bz, _)| *bx == *x && *by == *y && *bz == *z)
                            {
                                slot.3 = *w;
                            } else {
                                blocks.push((*x, *y, *z, *w));
                            }
                        }
                        for (x, y, z, w) in blocks {
                            writer
                                .write_all(
                                    Packet::BlockChunk { p, q, x, y, z, w }
                                        .encode()
                                        .as_bytes(),
                                )
                                .await?;
                        }
                        writer
                            .write_all(format!("K,{p},{q},0\n").as_bytes())
                            .await?;
                    }
                    Ok(Packet::Talk(t)) => {
                        let _ = shared.tx.send(format!("T,{id}> {t}\n"));
                    }
                    Ok(other) => warn!("unhandled from {id}: {other:?}"),
                    Err(e) => warn!("parse from {addr}: {e} ({line:?})"),
                }
                line.clear();
            }
        }
    }
    let _ = shared.tx.send(Packet::PlayerLeave(id).encode());
    info!("client {id} disconnected");
    Ok(())
}

pub async fn serve(listener: TcpListener, db_path: &str) -> anyhow::Result<()> {
    let db = Db::open(db_path).context("open db")?;
    let (tx, _) = broadcast::channel(16_384);
    let shared = Arc::new(Shared {
        db: Mutex::new(db),
        next_id: AtomicI32::new(1),
        tx,
        blocks: Mutex::new(HashMap::new()),
    });
    let addr = listener.local_addr()?;
    info!("craft-server listening on {addr}");
    loop {
        let (sock, peer) = listener.accept().await?;
        let shared = shared.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_client(sock, peer, shared).await {
                warn!("client error: {e:#}");
            }
        });
    }
}
