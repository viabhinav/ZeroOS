#![no_std]

extern crate alloc;

pub use foundation::ops::VfsOps;

pub use libc::{
    S_IFBLK, S_IFCHR, S_IFDIR, S_IFIFO, S_IFLNK, S_IFMT, S_IFREG, S_IFSOCK, S_IRGRP, S_IROTH,
    S_IRUSR, S_IRWXG, S_IRWXO, S_IRWXU, S_IWGRP, S_IWOTH, S_IWUSR, S_IXGRP, S_IXOTH, S_IXUSR,
};

pub mod device;
mod vfs;

pub use device::*;
pub use vfs::*;

use alloc::boxed::Box;

pub type Fd = i32;

pub type VfsResult<T> = Result<T, isize>;

/// File Descriptor Entry.
///
/// Holds the stateful device object.
pub struct FdEntry {
    pub device: Box<dyn Device>,
}

/// A factory that produces a new device instance.
pub trait DeviceFactory: Send + Sync {
    fn create(&self) -> Box<dyn Device>;
}
