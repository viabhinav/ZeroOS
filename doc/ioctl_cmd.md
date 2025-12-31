# ioctl Command Encoding in ZeroOS

ZeroOS adopts the standard Linux-style bit-packing for `ioctl` commands to ensure compatibility and robustness. This encoding embeds the command's direction, size, magic number (type), and sequence number into a single 32-bit integer.

## Bit Layout

The 32-bit command (`u32` / `usize`) is structured as follows:

| Bits      | Label  | Width | Description                                                            |
| :-------- | :----- | :---- | :--------------------------------------------------------------------- |
| **0-7**   | `NR`   | 8     | **Number**: Implementing sequence number (0-255).                      |
| **8-15**  | `TYPE` | 8     | **Type (Magic)**: Unique ASCII character for the driver (e.g., `'k'`). |
| **16-29** | `SIZE` | 14    | **Size**: Size of the data argument in bytes (max 16KB).               |
| **30-31** | `DIR`  | 2     | **Direction**: Data transfer direction (None, Read, Write, ReadWrite). |

## Directions (`DIR`)

| Value | Macro   | Direction | Description                                                  |
| :---- | :------ | :-------- | :----------------------------------------------------------- |
| `00`  | `_IO`   | None      | No data transfer. Argument is ignored or treated as a value. |
| `10`  | `_IOW`  | Write     | Userspace writes data to the kernel/device.                  |
| `01`  | `_IOR`  | Read      | Userspace reads data from the kernel/device.                 |
| `11`  | `_IOWR` | ReadWrite | Bidirectional transfer (Read-Modify-Write).                  |

> **Note**: This differs slightly from some architectures where Read/Write bits might be swapped, but follows the generic Linux convention used on RISC-V.

## Usage in Rust

ZeroOS provides the `IoctlCommand` struct to parse and encode these values type-safely.

```rust
use zeroos_foundation::ioctl::{IoctlCommand, IoctlDir};

let cmd = IoctlCommand::new(IoctlDir::Read, b'X', 1, 4); // Creating a command to READ a 4-byte integer from driver 'X', sequence 1

let raw = cmd.to_raw(); 
```
