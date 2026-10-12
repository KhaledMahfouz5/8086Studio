//! `studio-fmt` — file writers for the output formats 8086Studio supports.
//!
//! v0 (vertical slice): just `.com`. `.com` is the simplest format: a raw
//! byte dump that DOS loads at `CS:IP = 0:0100h` and jumps to.

#![forbid(unsafe_code)]

use std::io::Write;
use std::path::Path;

/// Write a `.com` file: the raw bytes, no header.
pub fn write_com(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut f = std::fs::File::create(path)?;
    f.write_all(bytes)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_com_round_trips_bytes() {
        let dir = std::env::temp_dir().join("studio-fmt-tests");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("hello.com");
        let bytes = vec![0xB8, 0x05, 0x00, 0xC3];
        write_com(&path, &bytes).unwrap();
        let read = std::fs::read(&path).unwrap();
        assert_eq!(read, bytes);
    }
}
