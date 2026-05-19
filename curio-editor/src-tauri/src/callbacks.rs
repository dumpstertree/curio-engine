#[unsafe(no_mangle)]
pub extern "C" fn set_resolution(_x: i32, _y: i32) {}

#[unsafe(no_mangle)]
pub extern "C" fn set_fullscreen(_x: bool) {}

#[unsafe(no_mangle)]
pub extern "C" fn set_cursor_visible(_x: bool) {}
