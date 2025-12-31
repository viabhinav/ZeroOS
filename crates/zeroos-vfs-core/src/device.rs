use foundation::ioctl::IoctlCommand;
use foundation::user_ptr::{UserPtr, UserVoidPtr};

/// A trait for device drivers.
///
/// This trait replaces the old function-pointer based `FileOps` table.
/// It enables stateful drivers using `Box<dyn Device>` and safer ioctl handling.
pub trait Device: Send + Sync {
    /// Read data from the device.
    fn read(&mut self, _buf: UserVoidPtr, _count: usize) -> isize {
        -(libc::EBADF as isize)
    }

    /// Write data to the device.
    fn write(&mut self, _buf: UserVoidPtr, _count: usize) -> isize {
        -(libc::EBADF as isize)
    }

    /// Seek to an offset.
    fn seek(&mut self, _offset: isize, _whence: i32) -> isize {
        -(libc::ESPIPE as isize)
    }

    /// Handle an ioctl command.
    fn ioctl(&mut self, _cmd: IoctlCommand, _arg: UserPtr<usize>) -> isize {
        -(libc::ENOTTY as isize)
    }

    /// Release resources when the file descriptor is closed.
    fn release(&mut self) -> isize {
        0
    }
}
