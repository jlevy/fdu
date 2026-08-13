use std::io::{Read, Write};

use anyhow::{Context, Result};

use crate::index::Entry;

pub fn write_entries(entries: &[Entry], mut destination: impl Write) -> Result<()> {
    for entry in entries {
        destination.write_all(&entry.parent.to_le_bytes())?;
        destination.write_all(&entry.bytes.to_le_bytes())?;
        destination.write_all(&entry.allocated.to_le_bytes())?;
    }
    Ok(())
}

pub fn read_entries(mut source: impl Read) -> Result<Vec<Entry>> {
    let mut bytes = Vec::new();
    source.read_to_end(&mut bytes).context("read entry table")?;
    crate::index::decode_entries(&bytes).context("decode entry table")
}
