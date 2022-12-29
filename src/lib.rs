#![feature(panic_info_message)]

#![deny(warnings)]

#![no_std]

use core::fmt::{self};
#[cfg(dos_app)]
use core::fmt::Write as fmt_Write;
use core::mem::{MaybeUninit, forget, transmute};
use core::slice::{self};
use exit_no_std::exit;
use pc_ints::*;

#[cfg(dos_app)]
mod link {
    use core::fmt::{self, Formatter};

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
}

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

#[cfg(dos_app)]
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

struct RmAlloc {
    selector: u16,
}

impl Drop for RmAlloc {
    fn drop(&mut self) {
       let _ = int_31h_ax_0101h_rm_free(self.selector);
    }
}

fn assert_dos_3_3() -> Result<(), &'static str> {
    let dos_ver = int_21h_ah_30h_dos_ver();
    if dos_ver.al_major < 3 || dos_ver.al_major == 3 && dos_ver.ah_minor < 30 {
        Err("DOS >= 3.3 required")
    } else {
        Ok(())
    }
}

pub fn load_code_page() -> Result<&'static CodePage, &'static str> {
    assert_dos_3_3()?;
    let code_page_memory = int_31h_ax_0100h_rm_alloc(8)
        .map_err(|_| "cannot allocate real-mode memory for code page tables")?;
    let code_page_selector = RmAlloc { selector: code_page_memory.dx_selector };
    let code_page_memory = unsafe { slice::from_raw_parts_mut(
        ((code_page_memory.ax_segment as u32) << 4) as *mut u8,
        512
    ) };
    let code_page_n = int_21h_ax_6601h_code_page()
        .map_err(|_| "cannot request selected DOS code page")?
        .bx_active;
    if code_page_n > 999 {
        return Err("unsupported code page");
    }
    let mut code_page: [MaybeUninit<u8>; 13] = unsafe { MaybeUninit::uninit().assume_init() };
    code_page[.. 9].copy_from_slice(unsafe { transmute(&b"CODEPAGE\\"[..]) });
    code_page[9].write(b'0' + (code_page_n / 100) as u8);
    code_page[10].write(b'0' + ((code_page_n % 100) / 10) as u8);
    code_page[11].write(b'0' + (code_page_n % 10) as u8);
    code_page[12].write(0);
    let code_page: [u8; 13] = unsafe { transmute(code_page) };
    let code_page = int_21h_ah_3Dh_open(code_page.as_ptr(), 0x00)
        .map_err(|_| "cannot open code page file")?
        .ax_handle;
    let mut code_page_buf: &mut [MaybeUninit<u8>] = unsafe { transmute(&mut code_page_memory[..]) };
    loop {
        if code_page_buf.is_empty() {
            let mut byte: MaybeUninit<u8> = MaybeUninit::uninit();
            let read = int_21h_ah_3Fh_read(code_page, slice::from_mut(&mut byte))
                .map_err(|_| "cannot read code page file")?
                .ax_read;
            if read != 0 {
                return Err("invalid code page file: too big");
            }
            break;
        }
        let read = int_21h_ah_3Fh_read(code_page, code_page_buf)
            .map_err(|_| "cannot read code page file")?
            .ax_read;
        if read == 0 { break; }
        code_page_buf = &mut code_page_buf[read as usize ..];
    }
    if !code_page_buf.is_empty() {
        return Err("invalid code page file: too small");
    }
    let code_page = unsafe { &*(code_page_memory.as_ptr() as *const CodePage) };
    forget(code_page_selector);
    Ok(code_page)
}
