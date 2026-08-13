use std::path::PathBuf;

pub type EntryId = u32;

pub struct Entry {
    pub parent: EntryId,
    pub name: PathBuf,
    pub bytes: u64,
    pub allocated: u64,
}

impl Entry {
    pub fn fingerprint(&self) -> u64 {
        self.bytes.rotate_left(7) ^ self.allocated.rotate_right(3) ^ u64::from(self.parent)
    }
}

#[derive(Clone, Copy, Default)]
pub struct RollUp {
    pub files: u64,
    pub directories: u64,
    pub bytes: u64,
    pub allocated: u64,
}

impl RollUp {
    pub fn add(&mut self, other: Self) {
        self.files += other.files;
        self.directories += other.directories;
        self.bytes += other.bytes;
        self.allocated += other.allocated;
    }
}
