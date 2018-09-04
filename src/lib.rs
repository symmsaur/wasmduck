extern crate cfg_if;

use std::os::raw::c_char;
use std::os::raw::c_void;
use std::mem;
mod utils;

#[no_mangle]
pub extern "C" fn alloc(size: usize) -> *mut c_void {
    let mut buf = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    mem::forget(buf);
    return ptr as *mut c_void;
}

#[no_mangle]
pub extern "C" fn sum(buffer: *mut c_char, size: i32) -> i32 {
   //assert!(buffer != std::ptr::null());
   let safe_buffer = unsafe {std::slice::from_raw_parts(buffer as *mut u8, size as usize)};
   let mut sum = 0;
   for color in safe_buffer {
       sum += *color as i32;
   }
   return sum;
}