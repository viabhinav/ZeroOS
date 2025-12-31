use crate::{DeviceFactory, Fd, FdEntry, VfsResult};
use foundation::ioctl::IoctlCommand;
use foundation::user_ptr::{UserPtr, UserVoidPtr};
use foundation::utils::GlobalCell;

use alloc::boxed::Box;

const MAX_FDS: usize = 256;

// We need a way to store factories dynamically

pub struct Vfs {
    fd_table: [Option<FdEntry>; MAX_FDS],
    next_fd: Fd,
    // We use a fixed size array of boxed factories.
    // Initialization is tricky with non-Copy types (Option<Box<...>>).
    // usage of array_init or similar would be needed, or just Vec if we had it.
    // For simplicity/no-deps, we will use a Vec which comes with alloc.
    devices: alloc::vec::Vec<(&'static str, Box<dyn DeviceFactory>)>,
}

impl Default for Vfs {
    fn default() -> Self {
        Self::new()
    }
}

impl Vfs {
    /// Create a new VFS instance
    pub const fn new() -> Self {
        Self {
            fd_table: [const { None }; MAX_FDS],
            next_fd: 3,
            devices: alloc::vec::Vec::new(),
        }
    }

    pub fn register_fd(&mut self, fd: Fd, entry: FdEntry) -> VfsResult<()> {
        if fd < 0 || fd as usize >= MAX_FDS {
            return Err(-(libc::EINVAL as isize));
        }
        self.fd_table[fd as usize] = Some(entry);
        Ok(())
    }

    pub fn register_device(
        &mut self,
        path: &'static str,
        factory: Box<dyn DeviceFactory>,
    ) -> VfsResult<()> {
        self.devices.push((path, factory));
        Ok(())
    }

    pub fn open(&mut self, path: &str, _flags: i32, _mode: u32) -> VfsResult<Fd> {
        // Find factory
        // We need to iterate and find match.
        // Since we changed to Vec, we can iterate easily.
        let factory = self
            .devices
            .iter()
            .find(|(p, _)| *p == path)
            .map(|(_, f)| f)
            .ok_or(-(libc::ENOENT as isize))?;

        let mut found: Option<Fd> = None;
        let start = self.next_fd.max(3) as usize;
        for idx in start..MAX_FDS {
            if self.fd_table[idx].is_none() {
                found = Some(idx as Fd);
                break;
            }
        }
        if found.is_none() {
            for idx in 3..start.min(MAX_FDS) {
                if self.fd_table[idx].is_none() {
                    found = Some(idx as Fd);
                    break;
                }
            }
        }
        let fd = match found {
            Some(fd) => fd,
            None => return Err(-(libc::EMFILE as isize)),
        };
        self.next_fd = if (fd as usize) + 1 < MAX_FDS {
            fd + 1
        } else {
            3
        };

        let device = factory.create();
        let entry = FdEntry { device };
        self.fd_table[fd as usize] = Some(entry);

        Ok(fd)
    }

    pub fn read(&mut self, fd: Fd, buf: *mut u8, count: usize) -> isize {
        if fd < 0 || fd as usize >= MAX_FDS {
            return -(libc::EBADF as isize);
        }
        if count != 0 && buf.is_null() {
            return -(libc::EFAULT as isize);
        }

        match self.fd_table[fd as usize].as_mut() {
            Some(entry) => {
                let user_buf = UserVoidPtr::new(buf as usize);
                entry.device.read(user_buf, count)
            }
            None => -(libc::EBADF as isize),
        }
    }

    pub fn write(&mut self, fd: Fd, buf: *const u8, count: usize) -> isize {
        if fd < 0 || fd as usize >= MAX_FDS {
            return -(libc::EBADF as isize);
        }
        if count != 0 && buf.is_null() {
            return -(libc::EFAULT as isize);
        }

        match self.fd_table[fd as usize].as_mut() {
            Some(entry) => {
                let user_buf = UserVoidPtr::new(buf as usize);
                entry.device.write(user_buf, count)
            }
            None => -(libc::EBADF as isize),
        }
    }

    pub fn lseek(&mut self, fd: Fd, offset: isize, whence: i32) -> isize {
        if fd < 0 || fd as usize >= MAX_FDS {
            return -(libc::EBADF as isize);
        }

        match self.fd_table[fd as usize].as_mut() {
            Some(entry) => entry.device.seek(offset, whence),
            None => -(libc::EBADF as isize),
        }
    }

    pub fn ioctl(&mut self, fd: Fd, request: usize, raw_arg: usize) -> isize {
        if fd < 0 || fd as usize >= MAX_FDS {
            return -(libc::EBADF as isize);
        }

        match self.fd_table[fd as usize].as_mut() {
            Some(entry) => {
                let cmd = IoctlCommand::from_raw(request);
                let arg = UserPtr::new(raw_arg);
                entry.device.ioctl(cmd, arg)
            }
            None => -(libc::EBADF as isize),
        }
    }

    pub fn close(&mut self, fd: Fd) -> isize {
        if fd < 0 || fd as usize >= MAX_FDS {
            return -(libc::EBADF as isize);
        }

        match self.fd_table[fd as usize].take() {
            Some(mut entry) => entry.device.release(),
            None => -(libc::EBADF as isize),
        }
    }

    pub fn fstat(&self, fd: Fd, statbuf: *mut libc::stat) -> isize {
        if fd < 0 || fd as usize >= MAX_FDS {
            return -(libc::EBADF as isize);
        }

        if statbuf.is_null() {
            return -(libc::EFAULT as isize);
        }

        -(libc::ENOSYS as isize)
    }
}

// Global VFS instance
// Note: We use GlobalCell (which behaves like a Mutex/RefCell) so we have interior mutability.
static VFS: GlobalCell<Vfs> = GlobalCell::new(Vfs::new());

pub fn register_fd(fd: Fd, entry: FdEntry) -> VfsResult<()> {
    VFS.with_mut(|vfs| vfs.register_fd(fd, entry))
}

pub fn register_device(path: &'static str, factory: Box<dyn DeviceFactory>) -> VfsResult<()> {
    VFS.with_mut(|vfs| vfs.register_device(path, factory))
}

pub fn read(fd: Fd, buf: *mut u8, count: usize) -> isize {
    VFS.with_mut(|vfs| vfs.read(fd, buf, count))
}

pub fn write(fd: Fd, buf: *const u8, count: usize) -> isize {
    VFS.with_mut(|vfs| vfs.write(fd, buf, count))
}

pub fn lseek(fd: Fd, offset: isize, whence: i32) -> isize {
    VFS.with_mut(|vfs| vfs.lseek(fd, offset, whence))
}

pub fn ioctl(fd: Fd, request: usize, arg: usize) -> isize {
    VFS.with_mut(|vfs| vfs.ioctl(fd, request, arg))
}

pub fn close(fd: Fd) -> isize {
    VFS.with_mut(|vfs| vfs.close(fd))
}

pub fn fstat(fd: Fd, statbuf: *mut libc::stat) -> isize {
    VFS.with(|vfs| vfs.fstat(fd, statbuf))
}

pub(crate) fn fstat_raw(fd: Fd, statbuf: *mut u8) -> isize {
    fstat(fd, statbuf as *mut libc::stat)
}

pub const VFS_OPS: crate::VfsOps = crate::VfsOps {
    init: || {},
    read,
    write,
    open: open_cstr,
    close,
    lseek,
    ioctl,
    fstat: fstat_raw,
};

/// # Safety
/// `path` must be a valid NUL-terminated string.
pub unsafe fn open_cstr(path: *const u8, flags: i32, mode: u32) -> isize {
    if path.is_null() {
        return -(libc::EFAULT as isize);
    }

    let mut len = 0;
    while *path.add(len) != 0 {
        len += 1;
        if len > 4096 {
            return -(libc::ENAMETOOLONG as isize);
        }
    }
    let slice = core::slice::from_raw_parts(path, len);
    match core::str::from_utf8(slice) {
        Ok(s) => VFS.with_mut(|vfs| match vfs.open(s, flags, mode) {
            Ok(fd) => fd as isize,
            Err(e) => e,
        }),
        Err(_) => -(libc::EINVAL as isize),
    }
}
