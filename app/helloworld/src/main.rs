#![no_std]
#![no_main]

use core::ptr::NonNull;

use alloc::string::ToString;
use sparreal_rt::kernel;

extern crate alloc;
extern crate sparreal_rt;

#[sparreal_rt::entry]
fn main() {
    unsafe {
        let s = "hello world".to_string();

        // let a = *(dtb_addr as *const u8);
        // let b = a + 1;

        assert_eq!(&s, "hello world");
    }
}
