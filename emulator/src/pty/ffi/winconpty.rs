use lazy_static::lazy_static;
use libc::{c_char, c_int};
use log::warn;
use tmui::tipc::parking_lot::RwLock;
use widestring::WideCString;

type RustCallback = unsafe extern "C" fn(data: *const c_char, len: c_int);
type Callback = dyn Fn(&str) + Send + Sync + 'static;
lazy_static! {
    static ref CALLBACK_HOLDER: RwLock<Option<Box<Callback>>> = RwLock::new(None);
}

#[link(name = "winconpty", kind = "static")]
extern "C" {
    fn openConPty(columns: c_int, lines: c_int) -> c_int;
    fn setUTF8Mode(enabled: bool);
    fn closeConPty(fd: c_int);
    fn resizeConPty(fd: c_int, columns: c_int, lines: c_int);
    fn startSubProcess(fd: c_int, cmd: *const u16) -> bool;
    fn startReadListenerBridge(fd: c_int, callback: RustCallback);
    fn writeData(fd: c_int, data: *const c_char);
}

#[inline]
pub(crate) fn open_conpty(columns: i32, lines: i32) -> i32 {
    unsafe { openConPty(columns, lines) }
}

#[inline]
pub(crate) fn set_utf8_mode(enabled: bool) {
    unsafe {
        setUTF8Mode(enabled);
    }
}

#[inline]
pub(crate) fn close_conpty(fd: i32) {
    unsafe {
        closeConPty(fd);
    }
}

#[inline]
pub(crate) fn resize_conpty(fd: i32, columns: i32, lines: i32) {
    unsafe {
        resizeConPty(fd, columns, lines);
    }
}

#[inline]
pub(crate) fn start_sub_process(fd: i32, cmd: &str) -> bool {
    let wstr = WideCString::from_str(cmd).unwrap();

    unsafe { startSubProcess(fd, wstr.as_ptr()) }
}

#[no_mangle]
extern "C" fn c_callback(data: *const c_char, len: c_int) {
    unsafe {
        let slice = std::slice::from_raw_parts(data as *const u8, len as usize);
        if let Ok(s) = std::str::from_utf8(slice) {
            if let Some(cb) = CALLBACK_HOLDER.read().as_ref() {
                cb(s);
            } else {
                warn!("Get callback from holder is None.")
            }
        } else {
            warn!("Convert to utf8 failed.")
        }
    }
}

/// `callback` will be executed in another thread.
#[inline]
pub(crate) fn start_read_listener<F>(fd: c_int, callback: F)
where
    F: Fn(&str) + Send + Sync + 'static,
{
    let cb: Box<Callback> = Box::new(callback);
    *CALLBACK_HOLDER.write() = Some(cb);

    unsafe {
        startReadListenerBridge(fd, c_callback);
    }
}

#[inline]
pub(crate) fn write_data(fd: i32, data: &str) {
    let wstr = WideCString::from_str(data).unwrap();
    unsafe {
        writeData(fd, std::mem::transmute(wstr.as_ptr()));
    }
}
