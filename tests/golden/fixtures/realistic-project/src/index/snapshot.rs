use std::io::{Read, Write};

use anyhow::{Context, Result};

use super::Index;

pub struct Header {
    pub version: u32,
    pub checksum: u64,
    pub entries: u64,
}

pub fn save(index: &Index, mut destination: impl Write) -> Result<()> {
    let header = Header {
        version: 1,
        checksum: checksum(index),
        entries: index.entries().len() as u64,
    };
    destination.write_all(&header.version.to_le_bytes())?;
    destination.write_all(&header.checksum.to_le_bytes())?;
    destination.write_all(&header.entries.to_le_bytes())?;
    Ok(())
}

pub fn load(mut source: impl Read) -> Result<Index> {
    let mut bytes = Vec::new();
    source.read_to_end(&mut bytes).context("read snapshot")?;
    crate::decode(&bytes).context("decode snapshot")
}

fn checksum(index: &Index) -> u64 {
    index.entries().iter().fold(0, |sum, entry| sum ^ entry.fingerprint())
}
