//! Headless e2e: worldgen → map → mesh → stats artifact.

use craft_core::mesh::mesh_chunk;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let p: i32 = args.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let q: i32 = args.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let out = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(format!("artifacts/mesh_{p}_{q}.stats")));

    let (data, stats) = mesh_chunk(p, q);
    if let Some(parent) = out.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let body = format!(
        "p={}\nq={}\nblocks={}\nfaces={}\nfloats={}\nminy={}\nmaxy={}\nao_sum={:.6}\nlight_sum={:.6}\nbytes={}\n",
        stats.p,
        stats.q,
        stats.blocks,
        stats.faces,
        stats.floats,
        stats.miny,
        stats.maxy,
        stats.ao_sum,
        stats.light_sum,
        data.len() * 4
    );
    if let Err(e) = fs::write(&out, &body) {
        eprintln!("write {}: {e}", out.display());
        return ExitCode::FAILURE;
    }
    print!("{body}");
    if stats.faces == 0 || stats.floats == 0 {
        eprintln!("FAIL: empty mesh");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
