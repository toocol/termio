#![allow(dead_code)]
use std::ffi::c_int;
use libc::c_void;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};
use wchar::wchar_t;
use widestring::WideString;

pub const SEEK_CUR: i32 = 1;
pub const SEEK_END: i32 = 2;
pub const SEEK_SET: i32 = 0;

pub const PROT_NONE: i32 = 0;
pub const PROT_READ: i32 = 1;
pub const PROT_WRITE: i32 = 2;
pub const PROT_EXEC: i32 = 4;

pub const MAP_FILE: i32 = 0;
pub const MAP_SHARED: i32 = 1;
pub const MAP_PRIVATE: i32 = 2;
pub const MAP_TYPE: i32 = 0xf;
pub const MAP_FIXED: i32 = 0x10;
pub const MAP_ANONYMOUS: i32 = 0x20;
pub const MAP_ANON: i32 = MAP_ANONYMOUS;

pub const MAP_FAILED: *const c_void = &-1 as *const i32 as *const c_void;

#[cfg(target_os = "windows")]
#[link(name = "native-system", kind = "static")]
extern "C" {
    fn mmap_ffi(
        addr: *const u8,
        len: usize,
        prot: c_int,
        flags: c_int,
        fildes: c_int,
        offset_type: i64,
    ) -> *const u8;
    fn munmap_ffi(addr: *const u8, len: usize) -> c_int;
    fn chsize_ffi(file_handle: c_int, size: c_int) -> c_int;
}

#[inline]
pub fn mmap(
    addr: *const u8,
    len: usize,
    prot: i32,
    flags: i32,
    fildes: i32,
    offset_type: i64,
) -> *const u8 {
    #[cfg(not(target_os = "windows"))]
    unsafe { libc::mmap(addr as *mut c_void, len, prot, flags, fildes, offset_type) as *const u8 }
    #[cfg(target_os = "windows")]
    unsafe { mmap_ffi(addr, len, prot, flags, fildes, offset_type) }
}

#[inline]
pub fn munmap(addr: *const u8, len: usize) -> i32 {
    #[cfg(not(target_os = "windows"))]
    unsafe { libc::munmap(addr as *mut c_void, len) }
    #[cfg(target_os = "windows")]
    unsafe { munmap_ffi(addr, len) }
}

#[inline]
pub fn chsize(file_handle: i32, size: i32) -> i32 {
    #[cfg(not(target_os = "windows"))]
    unsafe { libc::ftruncate(file_handle, size as i64) }
    #[cfg(target_os = "windows")]
    unsafe { chsize_ffi(file_handle, size) }
}

#[inline]
pub fn wcwidth(ucs: wchar_t) -> c_int {
    let char = char::from_u32(ucs as u32).unwrap();
    char.width().unwrap() as c_int
}

#[inline]
pub fn string_width(wstr: &WideString) -> c_int {
    let str = wstr.to_string().unwrap();
    str.width() as c_int
}

#[cfg(test)]
mod tests {
    use std::ptr::null;
    use libc::{close, dup, fileno, tmpfile};
    use wchar::wch;
    use widestring::WideString;
    use super::*;

    #[test]
    fn test_mmap() {
        let tmp = unsafe { tmpfile() };
        let ion = unsafe { dup(fileno(tmp)) };
        println!("Process `tempfile()`, handle = {}", ion);
        let ptr = mmap(null(), 1024, PROT_READ, MAP_PRIVATE, ion, 0);
        println!("Process `mmap`");
        munmap(ptr, 1024);
        println!("Process `munmap`");
        unsafe { close(ion) };
        println!("Process `close`");
    }

    #[test]
    fn test_wcwidth() {
        let wc = wch!('你');
        println!("{}", wcwidth(wc));

        let wcstring = WideString::from_str("你好RUST");
        println!("{}", string_width(&wcstring));

        let string = "Hello World\0";
        let u16string = WideString::from_str(string);
        println!("{}", string_width(&u16string));
    }
}
