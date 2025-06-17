#![no_std]

use num_enum::FromPrimitive;

pub mod macros;

#[repr(usize)]
#[derive(Clone, Debug, FromPrimitive)]
pub enum Syscall {
    Read = 0,
    Write = 1,
    Open = 2,
    Close = 3,

    Brk = 12,

    GetPid = 39,
    
    Fork = 58,
    Spawn = 59,
    Exit = 60,
    WaitPid = 61,

    Sem = 66,

    ListDir = 65531,
    Stat = 65532,
    Allocate = 65533,
    Deallocate = 65534,

    #[num_enum(default)]
    Unknown = 65535,
}
