extern crate cfg_if;

mod utils;

#[no_mangle]
pub extern "C" fn greet() -> i32 {
    return 42;
}
