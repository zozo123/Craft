//! SQLite layer matching `src/db.c` schema (block/light/sign/key/state).

use rusqlite::{params, Connection, OptionalExtension, Result};

const SCHEMA: &str = r#"
attach database 'auth.db' as auth;
create table if not exists auth.identity_token (
    username text not null,
    token text not null,
    selected int not null
);
create unique index if not exists auth.identity_token_username_idx
    on identity_token (username);
create table if not exists state (
    x float not null,
    y float not null,
    z float not null,
    rx float not null,
    ry float not null
);
create table if not exists block (
    p int not null,
    q int not null,
    x int not null,
    y int not null,
    z int not null,
    w int not null
);
create table if not exists light (
    p int not null,
    q int not null,
    x int not null,
    y int not null,
    z int not null,
    w int not null
);
create table if not exists key (
    p int not null,
    q int not null,
    key int not null
);
create table if not exists sign (
    p int not null,
    q int not null,
    x int not null,
    y int not null,
    z int not null,
    face int not null,
    text text not null
);
create unique index if not exists block_pqxyz_idx on block (p, q, x, y, z);
create unique index if not exists light_pqxyz_idx on light (p, q, x, y, z);
create unique index if not exists key_pq_idx on key (p, q);
create unique index if not exists sign_xyzface_idx on sign (x, y, z, face);
create index if not exists sign_pq_idx on sign (p, q);
"#;

pub type PlayerState = (f32, f32, f32, f32, f32);

pub struct Db {
    conn: Connection,
}

impl Db {
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        // auth.db attach uses a relative path; open from a temp dir in tests.
        conn.execute_batch(SCHEMA)?;
        Ok(Db { conn })
    }

    pub fn insert_block(&self, p: i32, q: i32, x: i32, y: i32, z: i32, w: i32) -> Result<()> {
        self.conn.execute(
            "insert or replace into block (p, q, x, y, z, w) values (?1,?2,?3,?4,?5,?6)",
            params![p, q, x, y, z, w],
        )?;
        Ok(())
    }

    pub fn load_blocks(&self, p: i32, q: i32) -> Result<Vec<(i32, i32, i32, i32)>> {
        let mut stmt = self
            .conn
            .prepare("select x, y, z, w from block where p = ?1 and q = ?2")?;
        let rows = stmt.query_map(params![p, q], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?;
        rows.collect()
    }

    pub fn set_key(&self, p: i32, q: i32, key: i32) -> Result<()> {
        self.conn.execute(
            "insert or replace into key (p, q, key) values (?1,?2,?3)",
            params![p, q, key],
        )?;
        Ok(())
    }

    pub fn get_key(&self, p: i32, q: i32) -> Result<Option<i32>> {
        self.conn
            .query_row(
                "select key from key where p = ?1 and q = ?2",
                params![p, q],
                |row| row.get(0),
            )
            .optional()
    }

    pub fn save_state(&self, x: f32, y: f32, z: f32, rx: f32, ry: f32) -> Result<()> {
        self.conn.execute("delete from state", [])?;
        self.conn.execute(
            "insert into state (x, y, z, rx, ry) values (?1,?2,?3,?4,?5)",
            params![x, y, z, rx, ry],
        )?;
        Ok(())
    }

    pub fn load_state(&self) -> Result<Option<PlayerState>> {
        self.conn
            .query_row("select x, y, z, rx, ry from state", [], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            })
            .optional()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;
    use std::fs;

    #[test]
    fn block_roundtrip_and_key() {
        let dir = temp_dir().join(format!("craft-db-{}", std::process::id()));
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("craft.db");
        // SCHEMA attaches auth.db relative to CWD — chdir into temp.
        let prev = std::env::current_dir().unwrap();
        std::env::set_current_dir(&dir).unwrap();
        let db = Db::open(path.to_str().unwrap()).unwrap();
        db.insert_block(0, 0, 1, 2, 3, 5).unwrap();
        db.insert_block(0, 0, 1, 2, 3, 7).unwrap(); // replace
        let blocks = db.load_blocks(0, 0).unwrap();
        assert_eq!(blocks, vec![(1, 2, 3, 7)]);
        db.set_key(0, 0, 99).unwrap();
        assert_eq!(db.get_key(0, 0).unwrap(), Some(99));
        db.save_state(1.0, 2.0, 3.0, 0.1, 0.2).unwrap();
        assert_eq!(db.load_state().unwrap(), Some((1.0, 2.0, 3.0, 0.1, 0.2)));
        std::env::set_current_dir(prev).unwrap();
        let _ = fs::remove_dir_all(&dir);
    }
}
