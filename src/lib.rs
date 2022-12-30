#![feature(panic_info_message)]

#![deny(warnings)]

#![no_std]

#[cfg(dos_errno_and_panic)]
mod link {
    use core::fmt::{self, Formatter};
    use core::fmt::Write as fmt_Write;
    use exit_no_std::exit;
    use pc_ints::*;

    static mut ERRNO: i32 = 0;

    #[no_mangle]
    extern "Rust" fn rust_errno() -> i32 {
        unsafe { ERRNO }
    }

    #[no_mangle]
    extern "Rust" fn rust_set_errno(e: i32) {
        unsafe { ERRNO = e; }
    }

    #[no_mangle]
    extern "Rust" fn rust_errno_fmt(e: i32, f: &mut Formatter) -> fmt::Result {
        write!(f, "Error {}", e)
    }

    #[cfg(all(windows, target_env="gnu"))]
    #[no_mangle]
    extern "C" fn rust_eh_register_frames () { }

    #[cfg(all(windows, target_env="gnu"))]
    #[no_mangle]
    extern "C" fn rust_eh_unregister_frames () { }

    struct DosLastChanceWriter;

    impl fmt::Write for DosLastChanceWriter {
        fn write_char(&mut self, c: char) -> fmt::Result {
            let c = c as u32;
            let c = if c > 0x7F || c == '\r' as u32 {
                b'?'
            } else {
                c as u8
            };
            if c == b'\n' {
                int_21h_ah_02h_out_ch(b'\r');
            }
            int_21h_ah_02h_out_ch(c);
            Ok(())
        }

        fn write_str(&mut self, s: &str) -> fmt::Result {
            for c in s.chars() {
                self.write_char(c)?;
            }
            Ok(())
        }
    }

    #[panic_handler]
    fn panic_handler(info: &core::panic::PanicInfo) -> ! {
        let _ = DosLastChanceWriter.write_str("panic");
        if let Some(&message) = info.message() {
            let _ = DosLastChanceWriter.write_str(": ");
            let _ = DosLastChanceWriter.write_fmt(message);
        } else if let Some(message) = info.payload().downcast_ref::<&str>() {
            let _ = DosLastChanceWriter.write_str(": ");
            let _ = DosLastChanceWriter.write_str(message);
        } else {
            let _ = DosLastChanceWriter.write_str("!");
        }
        if let Some(location) = info.location() {
            let _ = writeln!(DosLastChanceWriter, " ({})", location);
        } else {
            let _ = writeln!(DosLastChanceWriter);
        }
        exit(b'P')
    }
}
