use std::path::Path;

use anyhow::Result;

use crate::{Index, Observation, ScanConfig};

pub fn run(root: &Path, config: &ScanConfig) -> Result<Index> {
    let mut index = Index::new(root);
    crate::walk(root, config, &mut |observation: Observation| {
        index.apply(observation)
    })?;
    Ok(index)
}

pub fn refresh(root: &Path, previous: &mut Index, config: &ScanConfig) -> Result<()> {
    crate::reconcile(root, config, &mut |observation| {
        previous.apply(observation)
    })?;
    Ok(())
}

pub fn save(index: &Index, destination: &Path) -> Result<()> {
    let temporary = destination.with_extension("tmp");
    index.write_to(&temporary)?;
    std::fs::rename(temporary, destination)?;
    Ok(())
}
