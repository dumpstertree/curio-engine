use std::ffi::CStr;

use rusty_spine::c::{c_char, c_int};

pub fn main() {}

pub mod asset {
    pub mod model_asset_animated;
}
pub mod habit {
    pub mod habit_update;
}
pub mod facet {
    pub mod renderer_dynamic;
}

// pub mod assets {
//     pub mod model_asset_animated;
// }

// Called by spine runtime to read a file from disk.
// Must return a heap-allocated buffer that spine will free.
#[unsafe(no_mangle)]
pub extern "C" fn _spUtil_readFile(path: *const c_char, length: *mut c_int) -> *mut c_char {
    unsafe {
        let path_str = CStr::from_ptr(path).to_str().unwrap_or("");
        match std::fs::read(path_str) {
            Ok(bytes) => {
                *length = bytes.len() as c_int;
                // allocate with malloc so spine can free it with free()
                let ptr = libc::malloc(bytes.len()) as *mut c_char;
                std::ptr::copy_nonoverlapping(bytes.as_ptr() as *const c_char, ptr, bytes.len());
                ptr
            }
            Err(_) => {
                *length = 0;
                std::ptr::null_mut()
            }
        }
    }
}
