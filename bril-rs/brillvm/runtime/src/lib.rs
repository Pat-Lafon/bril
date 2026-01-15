#![no_std]

use core::ffi::{CStr, c_char};
use libc::{strtoll, strtod};

#[unsafe(no_mangle)]
pub extern "C" fn _bril_print_int(i: i64) {
    unsafe {
        libc::printf(c"%lld".as_ptr().cast(), i);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn _bril_print_bool(b: bool) {
    let c_str = if b { c"true" } else { c"false" };
    unsafe {
        libc::printf(c_str.as_ptr().cast());
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn _bril_print_float(f: f64) {
    if f.is_infinite() {
        if f.is_sign_negative() {
            unsafe {
                libc::printf(c"-Infinity".as_ptr().cast());
            }
        } else {
            unsafe {
                libc::printf(c"Infinity".as_ptr().cast());
            }
        }
    } else if f.is_nan() {
        unsafe {
            libc::printf(c"NaN".as_ptr().cast());
        }
    } else if f != 0.0 && (f.abs() >= 1E10 || f.abs() <= 1E-10) {
        unsafe {
            libc::printf(c"%.17e".as_ptr().cast(), f);
        }
    } else {
        unsafe {
            libc::printf(c"%.17lf".as_ptr().cast(), f);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn _bril_print_sep() {
    unsafe {
        libc::printf(c" ".as_ptr().cast());
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn _bril_print_end() {
    unsafe {
        libc::printf(c"\n".as_ptr().cast());
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn _bril_parse_int(arg: *const c_char) -> i64 {
    const DECIMAL_BASE: i32 = 10;
    unsafe { strtoll(arg, core::ptr::null_mut(), DECIMAL_BASE) }
}

#[unsafe(no_mangle)]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn _bril_parse_bool(arg: *const c_char) -> bool {
    let c_str = unsafe { CStr::from_ptr(arg) };
    c_str.to_bytes() == b"true"
}

#[unsafe(no_mangle)]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn _bril_parse_float(arg: *const c_char) -> f64 {
    unsafe { strtod(arg, core::ptr::null_mut()) }
}
