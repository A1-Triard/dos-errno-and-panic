#![feature(extern_types)]

#![windows_subsystem="console"]
#![no_std]
#![no_main]

extern crate dos_errno_and_panic;
extern crate pc_atomics;

extern {
    type PEB;
}

#[allow(non_snake_case)]
#[no_mangle]
extern "stdcall" fn mainCRTStartup(_: *const PEB) -> u64 {
    0
}
