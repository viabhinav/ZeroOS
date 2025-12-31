//! ioctl command encoding and decoding helpers.
//!
//! Follows the standard Linux encoding:
//! - Bits 0-7:   Sequence Number (NR)
//! - Bits 8-15:  Type/Magic (TYPE)
//! - Bits 16-29: Size (SIZE) - 14 bits
//! - Bits 30-31: Direction (DIR)

/// Direction of the ioctl data transfer.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoctlDir {
    None = 0,
    Read = 1,      // _IOC_READ
    Write = 2,     // _IOC_WRITE
    ReadWrite = 3, // _IOC_READ | _IOC_WRITE
}

impl IoctlDir {
    pub const fn from_u8(val: u8) -> Self {
        match val & 0b11 {
            0 => IoctlDir::None,
            1 => IoctlDir::Read,
            2 => IoctlDir::Write,
            3 => IoctlDir::ReadWrite,
            _ => unsafe { core::hint::unreachable_unchecked() },
        }
    }
}

/// Represents a decoded ioctl command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IoctlCommand {
    pub dir: IoctlDir,
    pub size: u16,
    pub magic: u8,
    pub nr: u8,
}

impl IoctlCommand {
    // Constants for shifting and masking (Linux Generic/RISC-V)
    pub const NRBITS: usize = 8;
    pub const TYPEBITS: usize = 8;
    pub const SIZEBITS: usize = 14;
    pub const DIRBITS: usize = 2;

    pub const NRSHIFT: usize = 0;
    pub const TYPESHIFT: usize = Self::NRSHIFT + Self::NRBITS;
    pub const SIZESHIFT: usize = Self::TYPESHIFT + Self::TYPEBITS;
    pub const DIRSHIFT: usize = Self::SIZESHIFT + Self::SIZEBITS;

    pub const IOC_NONE: usize = 0;
    pub const IOC_WRITE: usize = 1;
    pub const IOC_READ: usize = 2;
    // Note: Linux defines _IOC_READ as 2 and _IOC_WRITE as 1 in widely used generic headers,
    // but the IoctlDir values passed into the struct use abstract 1=Read, 2=Write for simplicity.
    // The to_raw method maps them to the correct ABI bits.

    /// Create a new command.
    ///
    /// # Panics
    /// Panics if size exceeds 16383 bytes (14 bits).
    pub const fn new(dir: IoctlDir, magic: u8, nr: u8, size: usize) -> Self {
        if size > ((1 << Self::SIZEBITS) - 1) {
            panic!("ioctl size too large");
        }
        Self {
            dir,
            magic,
            nr,
            size: size as u16,
        }
    }

    /// Decode a raw `usize` command into components.
    pub const fn from_raw(raw: usize) -> Self {
        let nr = (raw >> Self::NRSHIFT) as u8; // auto-masks to 8 bits because u8
        let magic = (raw >> Self::TYPESHIFT) as u8;
        let size = ((raw >> Self::SIZESHIFT) & ((1 << Self::SIZEBITS) - 1)) as u16;
        let dir_val = (raw >> Self::DIRSHIFT) & ((1 << Self::DIRBITS) - 1);

        // Map ABI bits to Enum
        // Generic ABI: 00=None, 01=Write, 10=Read, 11=ReadWrite
        /*
        Source: RISC-V: Read=2, Write=1.
        */

        let dir = match dir_val {
            0 => IoctlDir::None,
            1 => IoctlDir::Write,
            2 => IoctlDir::Read,
            3 => IoctlDir::ReadWrite,
            _ => IoctlDir::None, // Should be unreachable with mask
        };

        Self {
            dir,
            magic,
            nr,
            size,
        }
    }

    /// Encode into a raw usize.
    pub const fn to_raw(&self) -> usize {
        let dir_bits = match self.dir {
            IoctlDir::None => 0,
            IoctlDir::Write => 1,
            IoctlDir::Read => 2,
            IoctlDir::ReadWrite => 3,
        };

        (self.nr as usize) << Self::NRSHIFT
            | (self.magic as usize) << Self::TYPESHIFT
            | (self.size as usize) << Self::SIZESHIFT
            | dir_bits << Self::DIRSHIFT
    }
}

/// Macro for defining IO command (no data).
#[macro_export]
macro_rules! io {
    ($magic:expr, $nr:expr) => {
        $crate::ioctl::IoctlCommand::new($crate::ioctl::IoctlDir::None, $magic, $nr, 0).to_raw()
    };
}

/// Macro for defining IOR command (read from driver).
#[macro_export]
macro_rules! ior {
    ($magic:expr, $nr:expr, $ty:ty) => {
        $crate::ioctl::IoctlCommand::new(
            $crate::ioctl::IoctlDir::Read,
            $magic,
            $nr,
            core::mem::size_of::<$ty>(),
        )
        .to_raw()
    };
}

/// Macro for defining IOW command (write to driver).
#[macro_export]
macro_rules! iow {
    ($magic:expr, $nr:expr, $ty:ty) => {
        $crate::ioctl::IoctlCommand::new(
            $crate::ioctl::IoctlDir::Write,
            $magic,
            $nr,
            core::mem::size_of::<$ty>(),
        )
        .to_raw()
    };
}

/// Macro for defining IOWR command (read/write).
#[macro_export]
macro_rules! iowr {
    ($magic:expr, $nr:expr, $ty:ty) => {
        $crate::ioctl::IoctlCommand::new(
            $crate::ioctl::IoctlDir::ReadWrite,
            $magic,
            $nr,
            core::mem::size_of::<$ty>(),
        )
        .to_raw()
    };
}
