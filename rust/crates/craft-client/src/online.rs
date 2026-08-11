//! Headless online world engine (Wave K).
//!
//! Owns the network connection to `craft-server`, applies authoritative block
//! edits into a [`Map`], tracks remote players, and can (re)mesh the received
//! world with the same core meshing path the renderer uses. This is GPU-free so
//! it can be driven deterministically in tests and over SSH sandboxes.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use craft_core::map::Map;
use craft_core::mesh::{mesh_map, MeshStats};
use craft_core::physics::{get_sight_vector, hit_test_map};
use craft_protocol::Packet;

#[derive(Debug, Clone, Copy, Default)]
pub struct RemotePlayer {
    pub id: i32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub rx: f32,
    pub ry: f32,
}

/// Live networked world state for one local player.
pub struct OnlineWorld {
    writer: TcpStream,
    inbox: Receiver<Packet>,
    pub id: i32,
    pub map: Map,
    pub players: HashMap<i32, RemotePlayer>,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub rx: f32,
    pub ry: f32,
    pub blocks_received: u64,
    pub chunks_keyed: u64,
    dirty: bool,
}

impl OnlineWorld {
    /// Connect, perform the handshake, and read the initial `U`/`E` packets.
    pub fn connect(addr: &str, username: &str) -> std::io::Result<Self> {
        let stream = TcpStream::connect(addr)?;
        stream.set_nodelay(true).ok();
        let reader_stream = stream.try_clone()?;
        let (tx, rx) = mpsc::channel::<Packet>();
        thread::spawn(move || {
            let mut reader = BufReader::new(reader_stream);
            let mut line = String::new();
            loop {
                line.clear();
                match reader.read_line(&mut line) {
                    Ok(0) | Err(_) => break,
                    Ok(_) => {
                        if let Ok(p) = Packet::parse_line(&line) {
                            if tx.send(p).is_err() {
                                break;
                            }
                        }
                    }
                }
            }
        });

        // Wide origin so a few chunks around spawn fit in one Map (rel coords 0..255).
        let map = Map::new(-64, 0, -64, 0x3ffff);
        let mut world = OnlineWorld {
            writer: stream,
            inbox: rx,
            id: 0,
            map,
            players: HashMap::new(),
            x: 0.0,
            y: 40.0,
            z: 0.0,
            rx: 0.0,
            ry: 0.0,
            blocks_received: 0,
            chunks_keyed: 0,
            dirty: false,
        };

        // Wait for the server's YOU to learn our id / spawn.
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if Instant::now() > deadline {
                break;
            }
            match world.inbox.recv_timeout(Duration::from_millis(200)) {
                Ok(p) => {
                    let done = matches!(p, Packet::You(_));
                    world.apply(p);
                    if done {
                        break;
                    }
                }
                Err(_) => continue,
            }
        }

        world.send(&Packet::Version(1))?;
        world.send(&Packet::Authenticate {
            username: username.to_string(),
            token: "-".to_string(),
        })?;
        Ok(world)
    }

    fn send(&mut self, p: &Packet) -> std::io::Result<()> {
        self.writer.write_all(p.encode().as_bytes())
    }

    /// Ask the server for chunk (p, q).
    pub fn request_chunk(&mut self, p: i32, q: i32) -> std::io::Result<()> {
        self.send(&Packet::Chunk { p, q, key: 0 })
    }

    fn apply(&mut self, p: Packet) {
        match p {
            Packet::You(id) => self.id = id,
            Packet::BlockChunk { x, y, z, w, .. } => {
                self.map.set(x, y, z, w);
                self.blocks_received += 1;
                self.dirty = true;
            }
            Packet::Block { x, y, z, w } => {
                self.map.set(x, y, z, w);
                self.dirty = true;
            }
            Packet::Key(_) => self.chunks_keyed += 1,
            Packet::PlayerPosition {
                id,
                x,
                y,
                z,
                rx,
                ry,
            } => {
                if id != self.id {
                    self.players.insert(
                        id,
                        RemotePlayer {
                            id,
                            x,
                            y,
                            z,
                            rx,
                            ry,
                        },
                    );
                }
            }
            Packet::PlayerLeave(id) => {
                self.players.remove(&id);
            }
            _ => {}
        }
    }

    /// Drain all currently available packets. Returns how many were applied.
    pub fn pump(&mut self) -> usize {
        let mut n = 0;
        while let Ok(p) = self.inbox.try_recv() {
            self.apply(p);
            n += 1;
        }
        n
    }

    /// Block until `pred(self)` is true or `timeout` elapses. Pumps in between.
    pub fn pump_until<F: Fn(&Self) -> bool>(&mut self, pred: F, timeout: Duration) -> bool {
        let deadline = Instant::now() + timeout;
        loop {
            self.pump();
            if pred(self) {
                return true;
            }
            if Instant::now() > deadline {
                return false;
            }
            thread::sleep(Duration::from_millis(5));
        }
    }

    /// Send our current position to the server.
    pub fn send_position(&mut self) -> std::io::Result<()> {
        let (x, y, z, rx, ry) = (self.x, self.y, self.z, self.rx, self.ry);
        self.send(&Packet::Position { x, y, z, rx, ry })
    }

    /// Authoritative edit: persist + broadcast through the server, apply locally.
    pub fn edit_block(&mut self, x: i32, y: i32, z: i32, w: i32) -> std::io::Result<()> {
        self.map.set(x, y, z, w);
        self.dirty = true;
        self.send(&Packet::Block { x, y, z, w })
    }

    /// Break the block the player is looking at. Returns the removed cell.
    pub fn break_block(&mut self) -> std::io::Result<Option<(i32, i32, i32)>> {
        let (vx, vy, vz) = get_sight_vector(self.rx, self.ry);
        if let Some((hx, hy, hz, hw)) =
            hit_test_map(&self.map, 8.0, false, self.x, self.y, self.z, vx, vy, vz)
        {
            if hw > 0 {
                self.edit_block(hx, hy, hz, 0)?;
                return Ok(Some((hx, hy, hz)));
            }
        }
        Ok(None)
    }

    /// Place block `w` against the face being looked at. Returns the new cell.
    pub fn place_block(&mut self, w: i32) -> std::io::Result<Option<(i32, i32, i32)>> {
        let (vx, vy, vz) = get_sight_vector(self.rx, self.ry);
        if let Some((px, py, pz, _)) =
            hit_test_map(&self.map, 8.0, true, self.x, self.y, self.z, vx, vy, vz)
        {
            self.edit_block(px, py, pz, w)?;
            return Ok(Some((px, py, pz)));
        }
        Ok(None)
    }

    pub fn dirty(&self) -> bool {
        self.dirty
    }

    /// Mesh chunk (p, q) from the received world; clears the dirty flag.
    pub fn mesh(&mut self, p: i32, q: i32) -> (Vec<f32>, MeshStats) {
        self.dirty = false;
        mesh_map(p, q, &self.map)
    }
}
