#![feature(wasm_import_memory)]
#![wasm_import_memory]
extern crate cfg_if;

use std::os::raw::c_char;
mod utils;

#[no_mangle]
pub extern "C" fn greet(buffer: *mut c_char, size: i32) -> i32 {
   //assert!(buffer != std::ptr::null());
   let safe_buffer = unsafe {std::slice::from_raw_parts(buffer as *mut u8, size as usize)};
   let mut sum = 0;
   for color in safe_buffer {
       sum += *color as i32;
   }
   return sum;
}

#[no_mangle]
pub extern "C" fn list_int(buffer: *mut i32, size: i32) -> i32 {
   //assert!(buffer != std::ptr::null());
   let safe_buffer = unsafe {std::slice::from_raw_parts(buffer as *mut i32, size as usize)};
   let mut sum = 0;
   for color in safe_buffer {
       sum += *color as i32;
   }
   return sum;
}