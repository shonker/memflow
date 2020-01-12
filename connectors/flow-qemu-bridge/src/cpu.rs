use libc_print::*;
use std::ffi::CString;

use std::io::Result;

use crate::native::*;

#[allow(dead_code)]
pub fn state() -> Result<()> {
    // TODO:
    libc_eprintln!("read_registers()");

    let file_cstr = CString::new("cpu.rs").unwrap();
    QEMU_MUTEX_LOCK_IOTHREAD_IMPL.unwrap()(file_cstr.as_ptr(), 15);

    // TODO: this will crash if the vm is not running
    // TODO2: add a check...
    //let env = MON_GET_CPU_ENV.unwrap()();

    QEMU_MUTEX_UNLOCK_IOTHREAD.unwrap()();

    Ok(())
}
