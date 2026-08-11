//! Line-oriented Craft protocol: `CODE,arg,arg,...\n`

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Packet {
    Version(i32),
    Authenticate {
        username: String,
        token: String,
    },
    Position {
        x: f32,
        y: f32,
        z: f32,
        rx: f32,
        ry: f32,
    },
    Chunk {
        p: i32,
        q: i32,
        key: i32,
    },
    Block {
        x: i32,
        y: i32,
        z: i32,
        w: i32,
    },
    Light {
        x: i32,
        y: i32,
        z: i32,
        w: i32,
    },
    Sign {
        x: i32,
        y: i32,
        z: i32,
        face: i32,
        text: String,
    },
    Talk(String),
    Nick(String),
    You(i32),
    Time {
        day_length: i32,
        time_of_day: f32,
    },
    Key(i32),
    Redraw {
        p: i32,
        q: i32,
    },
    Disconnect,
    /// Server→client block payload with chunk coords: B,p,q,x,y,z,w
    BlockChunk {
        p: i32,
        q: i32,
        x: i32,
        y: i32,
        z: i32,
        w: i32,
    },
    Unknown(String),
}

#[derive(Debug)]
pub enum ParseError {
    Empty,
    BadField { packet: String, field: String },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for ParseError {}

fn parse_i32(s: &str) -> Result<i32, ParseError> {
    s.parse().map_err(|_| ParseError::BadField {
        packet: s.into(),
        field: "i32".into(),
    })
}

fn parse_f32(s: &str) -> Result<f32, ParseError> {
    s.parse().map_err(|_| ParseError::BadField {
        packet: s.into(),
        field: "f32".into(),
    })
}

impl Packet {
    pub fn encode(&self) -> String {
        match self {
            Packet::Version(v) => format!("V,{v}\n"),
            Packet::Authenticate { username, token } => format!("A,{username},{token}\n"),
            Packet::Position { x, y, z, rx, ry } => {
                format!("P,{x:.2},{y:.2},{z:.2},{rx:.2},{ry:.2}\n")
            }
            Packet::Chunk { p, q, key } => format!("C,{p},{q},{key}\n"),
            Packet::Block { x, y, z, w } => format!("B,{x},{y},{z},{w}\n"),
            Packet::Light { x, y, z, w } => format!("L,{x},{y},{z},{w}\n"),
            Packet::Sign {
                x,
                y,
                z,
                face,
                text,
            } => format!("S,{x},{y},{z},{face},{text}\n"),
            Packet::Talk(t) => format!("T,{t}\n"),
            Packet::Nick(n) => format!("N,{n}\n"),
            Packet::You(id) => format!("U,{id}\n"),
            Packet::Time {
                day_length,
                time_of_day,
            } => format!("E,{day_length},{time_of_day}\n"),
            Packet::Key(k) => format!("K,{k}\n"),
            Packet::Redraw { p, q } => format!("R,{p},{q}\n"),
            Packet::Disconnect => "D\n".into(),
            Packet::BlockChunk { p, q, x, y, z, w } => format!("B,{p},{q},{x},{y},{z},{w}\n"),
            Packet::Unknown(s) => {
                if s.ends_with('\n') {
                    s.clone()
                } else {
                    format!("{s}\n")
                }
            }
        }
    }

    pub fn parse_line(line: &str) -> Result<Packet, ParseError> {
        let line = line.trim_end_matches(['\n', '\r']);
        if line.is_empty() {
            return Err(ParseError::Empty);
        }
        let mut parts = line.splitn(2, ',');
        let code = parts.next().unwrap();
        let rest = parts.next().unwrap_or("");
        match code {
            "V" => Ok(Packet::Version(parse_i32(rest)?)),
            "A" => {
                let mut it = rest.splitn(2, ',');
                let username = it.next().unwrap_or("").into();
                let token = it.next().unwrap_or("").into();
                Ok(Packet::Authenticate { username, token })
            }
            "P" => {
                let f: Vec<_> = rest.split(',').collect();
                if f.len() < 5 {
                    return Err(ParseError::BadField {
                        packet: line.into(),
                        field: "P".into(),
                    });
                }
                Ok(Packet::Position {
                    x: parse_f32(f[0])?,
                    y: parse_f32(f[1])?,
                    z: parse_f32(f[2])?,
                    rx: parse_f32(f[3])?,
                    ry: parse_f32(f[4])?,
                })
            }
            "C" => {
                let f: Vec<_> = rest.split(',').collect();
                Ok(Packet::Chunk {
                    p: parse_i32(f.first().copied().unwrap_or(""))?,
                    q: parse_i32(f.get(1).copied().unwrap_or(""))?,
                    key: parse_i32(f.get(2).copied().unwrap_or("0"))?,
                })
            }
            "B" => {
                let f: Vec<_> = rest.split(',').collect();
                if f.len() >= 6 {
                    Ok(Packet::BlockChunk {
                        p: parse_i32(f[0])?,
                        q: parse_i32(f[1])?,
                        x: parse_i32(f[2])?,
                        y: parse_i32(f[3])?,
                        z: parse_i32(f[4])?,
                        w: parse_i32(f[5])?,
                    })
                } else if f.len() >= 4 {
                    Ok(Packet::Block {
                        x: parse_i32(f[0])?,
                        y: parse_i32(f[1])?,
                        z: parse_i32(f[2])?,
                        w: parse_i32(f[3])?,
                    })
                } else {
                    Err(ParseError::BadField {
                        packet: line.into(),
                        field: "B".into(),
                    })
                }
            }
            "L" => {
                let f: Vec<_> = rest.split(',').collect();
                Ok(Packet::Light {
                    x: parse_i32(f.first().copied().unwrap_or(""))?,
                    y: parse_i32(f.get(1).copied().unwrap_or(""))?,
                    z: parse_i32(f.get(2).copied().unwrap_or(""))?,
                    w: parse_i32(f.get(3).copied().unwrap_or(""))?,
                })
            }
            "S" => {
                let mut it = rest.splitn(5, ',');
                Ok(Packet::Sign {
                    x: parse_i32(it.next().unwrap_or(""))?,
                    y: parse_i32(it.next().unwrap_or(""))?,
                    z: parse_i32(it.next().unwrap_or(""))?,
                    face: parse_i32(it.next().unwrap_or(""))?,
                    text: it.next().unwrap_or("").into(),
                })
            }
            "T" => Ok(Packet::Talk(rest.into())),
            "N" => Ok(Packet::Nick(rest.into())),
            "U" => Ok(Packet::You(parse_i32(rest)?)),
            "E" => {
                let f: Vec<_> = rest.split(',').collect();
                Ok(Packet::Time {
                    day_length: parse_i32(f.first().copied().unwrap_or(""))?,
                    time_of_day: parse_f32(f.get(1).copied().unwrap_or("0"))?,
                })
            }
            "K" => Ok(Packet::Key(parse_i32(rest)?)),
            "R" => {
                let f: Vec<_> = rest.split(',').collect();
                Ok(Packet::Redraw {
                    p: parse_i32(f.first().copied().unwrap_or(""))?,
                    q: parse_i32(f.get(1).copied().unwrap_or(""))?,
                })
            }
            "D" => Ok(Packet::Disconnect),
            _ => Ok(Packet::Unknown(line.into())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_common_packets() {
        let samples = [
            Packet::Version(1),
            Packet::Authenticate {
                username: "alice".into(),
                token: "tok".into(),
            },
            Packet::Position {
                x: 1.5,
                y: 20.0,
                z: -3.25,
                rx: 0.5,
                ry: -0.25,
            },
            Packet::Block {
                x: 1,
                y: 2,
                z: 3,
                w: 5,
            },
            Packet::Talk("hello".into()),
            Packet::Disconnect,
        ];
        for p in samples {
            let enc = p.encode();
            let got = Packet::parse_line(&enc).unwrap();
            assert_eq!(got, p, "roundtrip {enc:?}");
        }
    }

    #[test]
    fn parses_c_client_examples() {
        assert_eq!(Packet::parse_line("V,1\n").unwrap(), Packet::Version(1));
        assert_eq!(
            Packet::parse_line("B,1,2,3,4\n").unwrap(),
            Packet::Block {
                x: 1,
                y: 2,
                z: 3,
                w: 4
            }
        );
        assert_eq!(
            Packet::parse_line("C,0,1,42\n").unwrap(),
            Packet::Chunk {
                p: 0,
                q: 1,
                key: 42
            }
        );
    }
}
