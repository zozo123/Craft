//! Shared helpers for golden-fixture parity tests.
//!
//! Each integration test compiles its own copy of this module and uses only
//! part of it, so unused-item warnings here are expected.
#![allow(dead_code)]

use std::path::{Path, PathBuf};

/// Directory holding the golden `.tsv` files produced by the C oracle.
///
/// Override with `CRAFT_GOLDEN_DIR`; otherwise defaults to the repo's
/// `fixtures/golden`, which `make -C oracle golden` writes to.
pub fn golden_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("CRAFT_GOLDEN_DIR") {
        return PathBuf::from(dir);
    }
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../fixtures/golden")
}

pub fn read_golden(name: &str) -> String {
    let path = golden_dir().join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "cannot read golden {}: {e}\n\
             Generate goldens first:  make -C oracle golden\n\
             (or set CRAFT_GOLDEN_DIR to a directory that contains them)",
            path.display()
        )
    })
}

/// Parses a C `printf(\"%a\")` hex float, e.g. `-0x1.dfbfc4p-2`, `0x1p+1`,
/// `0x0p+0`. Computed in f64 then narrowed; since `%a` of an f32 is exact,
/// this round-trips the original f32 bit-for-bit.
pub fn parse_hex_f32(s: &str) -> f32 {
    let s = s.trim();
    let (neg, rest) = match s.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, s),
    };
    let rest = rest
        .strip_prefix("0x")
        .or_else(|| rest.strip_prefix("0X"))
        .unwrap_or_else(|| panic!("not a hex float: {s}"));

    let (mantissa, exp_str) = rest
        .split_once(['p', 'P'])
        .unwrap_or_else(|| panic!("hex float missing exponent: {s}"));
    let exp: i32 = exp_str
        .parse()
        .unwrap_or_else(|_| panic!("bad exponent: {s}"));

    let (int_part, frac_part) = match mantissa.split_once('.') {
        Some((i, f)) => (i, f),
        None => (mantissa, ""),
    };

    let mut value = 0.0f64;
    if !int_part.is_empty() {
        value = i64::from_str_radix(int_part, 16).unwrap_or_else(|_| panic!("bad int part: {s}"))
            as f64;
    }
    let mut scale = 1.0f64 / 16.0;
    for c in frac_part.chars() {
        let d = c
            .to_digit(16)
            .unwrap_or_else(|| panic!("bad frac digit: {s}")) as f64;
        value += d * scale;
        scale /= 16.0;
    }

    value *= 2f64.powi(exp);
    if neg {
        value = -value;
    }
    value as f32
}

/// Non-comment, non-empty lines split into tab-separated fields.
pub fn rows(contents: &str) -> Vec<Vec<&str>> {
    contents
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| l.split('\t').collect())
        .collect()
}
