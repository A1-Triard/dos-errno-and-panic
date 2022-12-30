#![feature(extern_types)]

#![windows_subsystem="console"]
#![no_std]
#![no_main]

extern crate dos_errno_and_panic;
extern crate pc_atomics;
extern crate rlibc;

mod no_std {
    #[no_mangle]
    extern "C" fn _aulldiv() -> ! { panic!("10") }
    #[no_mangle]
    extern "C" fn _aullrem() -> ! { panic!("11") }
    #[no_mangle]
    extern "C" fn _chkstk() { }
    #[no_mangle]
    extern "C" fn _fltused() -> ! { panic!("13") }
    #[no_mangle]
    extern "C" fn strlen() -> ! { panic!("14") }
}

extern {
    type PEB;
}

#[allow(non_snake_case)]
#[no_mangle]
extern "stdcall" fn mainCRTStartup(_: *const PEB) -> u64 {
    panic!("Hello! I am panicking.");
}
