//! Filesystem capacity on Windows: free space and size from `GetDiskFreeSpaceExW`, and the
//! filesystem's name from `GetVolumeInformationW` on the volume that holds the path.

use std::io;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use windows_sys::Win32::Storage::FileSystem::{
    GetDiskFreeSpaceExW, GetVolumeInformationW, GetVolumePathNameW,
};

use super::VolumeCapacity;

/// Long enough for any volume root and filesystem name; `MAX_PATH` plus its NUL.
const BUFFER: usize = 261;

fn wide(path: &Path) -> Vec<u16> {
    path.as_os_str().encode_wide().chain(std::iter::once(0)).collect()
}

fn text(buffer: &[u16]) -> String {
    let end = buffer.iter().position(|unit| *unit == 0).unwrap_or(buffer.len());
    String::from_utf16_lossy(&buffer[..end])
}

pub(super) fn observe(path: &Path) -> io::Result<VolumeCapacity> {
    let path = wide(path);
    let (mut available, mut capacity, mut free) = (0_u64, 0_u64, 0_u64);
    // SAFETY: `path` is NUL-terminated and outlives the call, and the three outputs are
    // valid, writable `u64`s.
    if unsafe {
        GetDiskFreeSpaceExW(path.as_ptr(), &raw mut available, &raw mut capacity, &raw mut free)
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    Ok(VolumeCapacity {
        capacity_bytes: capacity,
        available_bytes: available,
        filesystem: filesystem_name(&path),
    })
}

/// The name of the filesystem on the volume that holds `path`, or `None` where Windows
/// does not say; capacity stands without it.
fn filesystem_name(path: &[u16]) -> Option<String> {
    let mut root = [0_u16; BUFFER];
    let mut name = [0_u16; BUFFER];
    let length = u32::try_from(BUFFER).ok()?;
    // SAFETY: `path` is NUL-terminated, and each buffer is writable for the length passed;
    // every pointer argument left null is one the API documents as optional.
    let named = unsafe {
        GetVolumePathNameW(path.as_ptr(), root.as_mut_ptr(), length) != 0
            && GetVolumeInformationW(
                root.as_ptr(),
                std::ptr::null_mut(),
                0,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                name.as_mut_ptr(),
                length,
            ) != 0
    };
    named.then(|| text(&name)).filter(|name| !name.is_empty())
}
