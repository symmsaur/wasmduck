extern crate cfg_if;

use std::mem;
use std::os::raw::c_char;
use std::os::raw::c_void;
mod utils;
mod sph;

#[no_mangle]
pub extern "C" fn alloc(size: usize) -> *mut c_void {
    let mut buf = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    mem::forget(buf);
    return ptr as *mut c_void;
}

#[no_mangle]
pub extern "C" fn update(buffer: *mut c_char, width: usize, height: usize) {
    let byte_size = width * height * 4;
    let safe_buffer = unsafe { std::slice::from_raw_parts_mut(buffer as *mut u8, byte_size) };
    for y in 0..height {
        for x in 0..width {
            let density = sph::density(x as f64 / width as f64, y as f64 / height as f64);
            let mut norm_density = (255. * (density - 0.6) / (0.7 - 0.6)).round() as i32;
            if norm_density > 255 {
                norm_density = 255;
            }
            if norm_density < 0 {
                norm_density = 0;
            }
            let index = (y * width + x) * 4;
            safe_buffer[index + 0] = 255;
            safe_buffer[index + 3] = norm_density as u8;
        }
    }
}
